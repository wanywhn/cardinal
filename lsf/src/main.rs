use bincode::{Decode, Encode};
use clap::Parser;
use fswalk::{Node, WalkData, walk_it};
use namepool::NamePool;
use serde::{Deserialize, Serialize};
use slab::Slab;
use std::{
    collections::BTreeMap,
    fs::{self, File, Metadata},
    io::{BufWriter, Write},
    path::PathBuf,
    time::{Instant, UNIX_EPOCH},
};

#[derive(Serialize, Deserialize, Encode, Decode)]
struct SlabNode {
    parent: Option<usize>,
    children: Vec<usize>,
    name: String,
}

impl SlabNode {
    /// Get the path of the node in the slab.
    pub fn path(&self, slab: &Slab<SlabNode>) -> String {
        let mut segments = vec![self.name.clone()];
        // Write code like this to avoid the root node, which has no node name and shouldn't be put into semgents.
        if let Some(mut parent) = self.parent {
            while let Some(new_parent) = slab[parent].parent {
                segments.push(slab[parent].name.clone());
                parent = new_parent
            }
        }
        let mut result = String::new();
        for segment in segments.into_iter().rev() {
            result.push('/');
            result.push_str(&segment);
        }
        result
    }
}

pub struct SlabNodeData {
    pub name: String,
    pub ctime: Option<u64>,
    pub mtime: Option<u64>,
}

impl SlabNodeData {
    pub fn new(name: String, metadata: &Option<Metadata>) -> Self {
        let (ctime, mtime) = match metadata {
            Some(metadata) => ctime_mtime_from_metadata(metadata),
            None => (None, None),
        };
        Self { name, ctime, mtime }
    }
}

fn ctime_mtime_from_metadata(metadata: &Metadata) -> (Option<u64>, Option<u64>) {
    // TODO(ldm0): is this fast enough?
    let ctime = metadata
        .created()
        .ok()
        .and_then(|x| x.duration_since(UNIX_EPOCH).ok())
        .map(|x| x.as_secs());
    let mtime = metadata
        .modified()
        .ok()
        .and_then(|x| x.duration_since(UNIX_EPOCH).ok())
        .map(|x| x.as_secs());
    (ctime, mtime)
}

fn construct_node_slab(parent: Option<usize>, node: &Node, slab: &mut Slab<SlabNode>) -> usize {
    let slab_node = SlabNode {
        parent,
        children: vec![],
        name: node.name.clone(),
    };
    let index = slab.insert(slab_node);
    slab[index].children = node
        .children
        .iter()
        .map(|node| construct_node_slab(Some(index), node, slab))
        .collect();
    index
}

/// Combine the construction routine of NamePool and BTreeMap since we can deduplicate node name for free.
// TODO(ldm0): Memory optimization can be done by letting name index reference the name in the pool(gc need to be considered though)
fn construct_name_index_and_namepool(
    slab: &Slab<SlabNode>,
    name_index: &mut BTreeMap<String, Vec<usize>>,
    name_pool: &mut NamePool,
) {
    for (i, node) in slab.iter() {
        if let Some(nodes) = name_index.get_mut(&node.name) {
            nodes.push(i);
        } else {
            name_pool.push(&node.name);
            name_index.insert(node.name.clone(), vec![i]);
        };
    }
}

#[derive(Parser)]
struct Cli {
    #[clap(short, long, default_value = "false")]
    /// Open enabled, cache was ignored and filesystem will be rewalked.
    refresh: bool,
}

fn walkfs_to_slab() -> (usize, Slab<SlabNode>) {
    // 先多线程构建树形文件名列表(不能直接创建 slab 因为 slab 无法多线程构建)
    let walk_data = WalkData::default();
    let visit_time = Instant::now();
    let node = walk_it(PathBuf::from("/"), &walk_data).expect("failed to walk");
    dbg!(walk_data);
    dbg!(visit_time.elapsed());

    // 然后创建 slab
    let slab_time = Instant::now();
    let mut slab = Slab::new();
    let slab_root = construct_node_slab(None, &node, &mut slab);
    dbg!(slab_time.elapsed());
    dbg!(slab_root);
    dbg!(slab.len());

    (slab_root, slab)
}

fn name_index_and_pool(slab: &Slab<SlabNode>) -> (BTreeMap<String, Vec<usize>>, NamePool) {
    let name_index_time = Instant::now();
    let mut name_index = BTreeMap::default();
    let mut name_pool = NamePool::new();
    construct_name_index_and_namepool(&slab, &mut name_index, &mut name_pool);
    dbg!(name_index_time.elapsed());
    println!("name index len: {}", name_index.len());
    println!("name pool size: {}MB", name_pool.len() / 1024 / 1024);
    (name_index, name_pool)
}

#[derive(Encode, Decode)]
struct PersistentStorage {
    slab_root: usize,
    slab: Slab<SlabNode>,
    name_index: BTreeMap<String, Vec<usize>>,
    name_pool: NamePool,
}

fn main() {
    let cli = Cli::parse();
    let (slab_root, slab) = walkfs_to_slab();
    let (name_index, name_pool) = name_index_and_pool(&slab);

    let stdin = std::io::stdin();
    let mut stdout = std::io::stdout();
    loop {
        print!(">");
        stdout.flush().unwrap();
        let mut line = String::new();
        stdin.read_line(&mut line).unwrap();
        let line = line.trim();
        if line.is_empty() {
            continue;
        } else if line == "/bye" {
            break;
        }
        {
            // Search out all leafs that contain the substring
            // e.g. "foo": ["/System/foo", "/System/Library/aaafoo"]
            // "/System/Library/aaafool/heck" won't be presented
            let search_time = Instant::now();
            for (i, name) in name_pool.search_substr(line).enumerate() {
                // TODO(ldm0): this can be parallelized
                if let Some(nodes) = name_index.get(name) {
                    for &node in nodes {
                        println!("[{}] {}", i, slab[node].path(&slab));
                    }
                }
            }
            dbg!(search_time.elapsed());
        }
    }

    {
        let bincode_time = Instant::now();
        let output = File::create("target/tree.bin").unwrap();
        let mut output = BufWriter::new(output);
        bincode::encode_into_std_write(&slab, &mut output, bincode::config::standard()).unwrap();
        dbg!(bincode_time.elapsed());
        dbg!(fs::metadata("target/tree.bin").unwrap().len() / 1024 / 1024);
    }

    {
        let zstd_bincode_time = Instant::now();
        let output = File::create("target/tree.bin.zstd").unwrap();
        let mut output = zstd::Encoder::new(output, 3).unwrap();
        bincode::encode_into_std_write(&slab, &mut output, bincode::config::standard()).unwrap();
        dbg!(zstd_bincode_time.elapsed());
        dbg!(fs::metadata("target/tree.bin.zstd").unwrap().len() / 1024 / 1024);
    }
}
