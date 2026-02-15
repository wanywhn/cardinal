use base64::{engine::general_purpose, Engine as _};
use fs_icon;
use napi_derive_ohos::napi;
use napi_ohos::threadsafe_function::{ThreadsafeFunction, ThreadsafeFunctionCallMode};
use napi_ohos::{Error, Result};
use ohos_fileuri_binding::get_path_from_uri;
use ohos_hilog_binding::{hilog_debug, hilog_info};
use once_cell::sync::{Lazy, OnceCell};
use search_cache::{SearchCache, SearchOptions, SearchResultNode, SlabNodeMetadataCompact, WalkData};
use search_cancel::CancellationToken;
use std::{
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, RwLock,
    },
    time::Duration,
};
use std::sync::Once;

// 全局状态
static APP_QUIT: AtomicBool = AtomicBool::new(false);
static DB_PATH: OnceCell<PathBuf> = OnceCell::new();
static BACKEND_STATE: Lazy<RwLock<BackendState>> = Lazy::new(|| RwLock::new(BackendState::new()));

// NodeInfo 结构体 - 与 Tauri 版本保持一致
#[napi(object)]
pub struct NodeInfo {
    pub path: String,
    pub metadata: Option<NodeInfoMetadata>,
    pub icon: Option<String>,
}

#[napi(object)]
pub struct NodeInfoMetadata {
    pub r#type: u8,
    pub size: i64,
    pub ctime: u32,
    pub mtime: u32,
}

#[napi]
#[repr(u8)]
pub enum NodeFileType {
    // File occurs a lot, assign it to 0 for better compression ratio(I guess... maybe useful).
    File = 0,
    Dir = 1,
    Symlink = 2,
    Unknown = 3,
}

impl NodeInfoMetadata {
    fn from_metadata(metadata: &SlabNodeMetadataCompact) -> Self {
        match metadata.as_ref() {
            Some(metadata_ref) => Self {
                r#type: metadata_ref.r#type() as u8,
                size: metadata_ref.size(),
                ctime: metadata_ref.ctime().map(|x| x.get()).unwrap_or_default(),
                mtime: metadata_ref.mtime().map(|x| x.get()).unwrap_or_default(),
            },
            None => Self {
                r#type: 0,
                size: -1,
                ctime: 0,
                mtime: 0,
            },
        }
    }
}

// 生命周期状态
#[napi]
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LifecycleState {
    Uninitialized = 0,
    Initializing = 1,
    Indexing = 2,
    Ready = 3,
    Updating = 4,
    Error = 5,
}

impl LifecycleState {
    fn as_u8(self) -> u8 {
        self as u8
    }

    fn from_u8(value: u8) -> Self {
        match value {
            0 => LifecycleState::Uninitialized,
            1 => LifecycleState::Initializing,
            2 => LifecycleState::Indexing,
            3 => LifecycleState::Ready,
            4 => LifecycleState::Updating,
            5 => LifecycleState::Error,
            _ => LifecycleState::Uninitialized,
        }
    }

    fn to_str(self) -> &'static str {
        match self {
            LifecycleState::Uninitialized => "UNINITIALIZED",
            LifecycleState::Initializing => "INITIALIZING",
            LifecycleState::Indexing => "INDEXING",
            LifecycleState::Ready => "READY",
            LifecycleState::Updating => "UPDATING",
            LifecycleState::Error => "ERROR",
        }
    }
}

// 后端状态
struct BackendState {
    lifecycle_state: LifecycleState,
    search_cache: Option<Arc<RwLock<SearchCache>>>,
    root_path: Option<PathBuf>,
    func_set_state: Option<ThreadsafeFunction<LifecycleState, ()>>
}

impl BackendState {
    fn new() -> Self {
        Self {
            lifecycle_state: LifecycleState::Uninitialized,
            search_cache: None,
            root_path: None,
            func_set_state: None,
        }
    }

    pub fn set_lifecycle_state(&mut self, lifecycle_state: LifecycleState) {
        self.lifecycle_state = lifecycle_state;
        if let Some(func_mtd) = &self.func_set_state {
            func_mtd.call_with_return_value(
                Ok(self.lifecycle_state),
                ThreadsafeFunctionCallMode::NonBlocking,
                |_result, _env| {
                    Ok(())
                }
            );
        }
    }

    pub fn set_func_set_state(&mut self, func_set_state: Option<ThreadsafeFunction<LifecycleState, ()>>) {
        self.func_set_state = func_set_state;
    }
}

// 鸿蒙后端初始化主函数
#[napi]
pub async fn initialize_harmony_backend(
    watch_root: String,
    ignore_paths: Vec<String>,
    db_uri: String,
    func_set_state: ThreadsafeFunction<LifecycleState, ()>
) -> Result<LifecycleState> {
    hilog_debug!("Backend: Starting HarmonyOS backend initialization");
    BACKEND_STATE.write().unwrap().set_func_set_state(Some(func_set_state));
    update_lifecycle_state(LifecycleState::Initializing);

    // 初始化数据库路径
    hilog_debug!("Backend: db_uri : {:?}", db_uri);
    let db_path = PathBuf::from(get_path_from_uri(&db_uri).unwrap());
    hilog_debug!("Backend: db_path : {:?}", db_path);

    DB_PATH
        .set(db_path)
        .map_err(|_| Error::from_reason("Failed to set DB path"))?;

    hilog_debug!(
        "Backend: Starting backend with watch_root: {}, ignore_paths: {}",
        watch_root,
        ignore_paths.len()
    );
    let root_path = get_path_from_uri(&watch_root).unwrap();
    hilog_debug!("Backend: Root path: {:?}", root_path);

    // 立即返回索引中状态
    update_lifecycle_state(LifecycleState::Indexing);

    // 在异步任务中运行逻辑线程
    tokio::task::spawn_blocking(move || {
        if let Err(e) = run_logic_thread(root_path, ignore_paths) {
            hilog_debug!("Backend: Logic thread failed: {}", e);
            update_lifecycle_state(LifecycleState::Error);
        }
    });

    Ok(LifecycleState::Indexing)
}

// 更新生命周期状态
fn update_lifecycle_state(new_state: LifecycleState) {
    let mut state = BACKEND_STATE.write().unwrap();
    state.set_lifecycle_state(new_state);
    println!("Lifecycle state changed to: {}", new_state.to_str());
}

pub(crate) fn build_search_cache(
    watch_root: &str,
    ignore_paths: &[PathBuf],
) -> Option<SearchCache> {
    let path = PathBuf::from(watch_root);
    let walk_data = WalkData::new(
        &path,
        ignore_paths,
        false,
        Some(&APP_QUIT),
    );
    let walking_done = AtomicBool::new(false);

    std::thread::scope(|s| {
        s.spawn(|| {
            while !walking_done.load(Ordering::Relaxed) {
                let dirs = walk_data.num_dirs.load(Ordering::Relaxed);
                let files = walk_data.num_files.load(Ordering::Relaxed);
                let _total = dirs + files;
                //TODO update cache info to UI
                std::thread::sleep(Duration::from_millis(100));
            }
        });
        let cache =
            SearchCache::walk_fs_with_walk_data(&walk_data, Some(&APP_QUIT));
        walking_done.store(true, Ordering::Relaxed);
        cache
    })
}

// 运行逻辑线程
fn run_logic_thread(watch_root: String, ignore_paths: Vec<String>) -> Result<()> {
    // 检查数据库路径
    let db_path = DB_PATH
        .get()
        .ok_or_else(|| Error::from_reason("DB path not initialized"))?;

    hilog_debug!(
        "Backend: Attempting to initialize backend with watch_root: {}",
        watch_root
    );
    hilog_debug!("Backend: Database path: {:?}", db_path);
    hilog_debug!("Backend: Ignore paths: {:?}", ignore_paths);

    // 构建搜索缓存
    let watch_path = PathBuf::from(&watch_root);
    let ignore_paths: Vec<PathBuf> = ignore_paths.iter().map(PathBuf::from).collect();

    hilog_debug!("Backend :Building search cache for path: {:?}", watch_path);

    let cache = match SearchCache::try_read_persistent_cache(
        &watch_path,
        db_path,
        &ignore_paths,
        Some(&APP_QUIT),
    ) {
        Ok(cached) => {
            hilog_info!("Loaded existing cache, Total files: {}", cached.get_total_files());
            //TODO update cache info to UI
            cached
        }
        Err(e) => {
            hilog_info!("Walking filesystem: {:?}", e);
            let Some(cache) = build_search_cache(&watch_root, &ignore_paths) else {
                hilog_info!("Walk filesystem cancelled, app quitting");
                return Ok(());
            };
            //TODO update cache info to UI
            hilog_info!("build_search_cache ok");
            cache
        }
    };

    hilog_debug!(
        "Backend: Search cache built successfully. Total files: {}",
        cache.get_total_files()
    );

    // 更新后端状态
    {
        let mut state = BACKEND_STATE.write().unwrap();
        state.search_cache = Some(Arc::new(RwLock::new(cache)));
        state.root_path = Some(watch_path);
        state.set_lifecycle_state(LifecycleState::Ready);
    }

    hilog_debug!("Backend: HarmonyOS backend is ready");
    Ok(())
}

// 执行搜索 - 完整实现
#[napi]
pub async fn search(
    query: String,
    case_insensitive: Option<bool>,
    max_results: Option<u32>,
) -> Result<Vec<u32>> {
    let state = BACKEND_STATE.read().unwrap();

    if state.lifecycle_state != LifecycleState::Ready {
        return Err(Error::from_reason(format!(
            "Backend not ready. Current state: {}",
            state.lifecycle_state.to_str()
        )));
    }

    hilog_debug!("Backend:Searching for: '{}'", query);
    hilog_debug!("Backend:Case insensitive: {:?}", case_insensitive);
    hilog_debug!("Backend:Max results: {:?}", max_results);

    // 获取搜索缓存
    let search_cache_ref = match &state.search_cache {
        Some(cache) => cache.clone(),
        None => return Err(Error::from_reason("Search cache not initialized")),
    };

    // 配置搜索选项
    let options = SearchOptions {
        case_insensitive: case_insensitive.unwrap_or(false),
    };

    // 执行搜索
    let cancellation_token = CancellationToken::noop();

    // 提前获取写锁并执行搜索
    let search_result = {
        let mut cache_write = search_cache_ref.write().unwrap();
        cache_write.search_with_options(&query, options, cancellation_token)
    };

    match search_result {
        Ok(outcome) => {
            let results: Vec<u32> = outcome
                .nodes
                .unwrap_or_default()
                .into_iter()
                .map(|idx| idx.get() as u32)
                .collect();

            hilog_debug!("Backend:Search returned {} results", results.len());
            Ok(results)
        }
        Err(e) => {
            hilog_debug!("Backend:Search error: {}", e);
            Err(Error::from_reason(format!("Search failed: {}", e)))
        }
    }
}

// 获取节点信息 - 完整实现
#[napi]
pub async fn get_nodes_info(
    slab_indices: Vec<u32>,
    include_icons: Option<bool>,
) -> Result<Vec<NodeInfo>> {
    if slab_indices.is_empty() {
        hilog_debug!("Backend: get_nodes_info for empty idx");
        return Ok(Vec::new());
    }

    let include_icons = include_icons.unwrap_or(true);
    let state = BACKEND_STATE.read().unwrap();

    if state.lifecycle_state != LifecycleState::Ready {
        return Err(Error::from_reason(format!(
            "Backend not ready. Current state: {}",
            state.lifecycle_state.to_str()
        )));
    }

    // 获取搜索缓存
    let search_cache_ref = match &state.search_cache {
        Some(cache) => cache.clone(),
        None => return Err(Error::from_reason("Search cache not initialized")),
    };

    // 转换索引类型
    let slab_indices: Vec<search_cache::SlabIndex> = slab_indices
        .into_iter()
        .map(|idx| search_cache::SlabIndex::new(idx as usize))
        .collect();

    // 从缓存中获取节点信息 - 需要可变引用调用expand_file_nodes
    let nodes = {
        let mut cache = search_cache_ref.write().unwrap();
        cache.expand_file_nodes(&slab_indices)
    };

    let node_infos: Vec<NodeInfo> = nodes
        .into_iter()
        .map(|SearchResultNode { path, metadata }| {
            let path_str = path.to_string_lossy().into_owned();

            // 计算图标（如果需要）
            let icon = if include_icons {
                // 鸿蒙平台使用 fs-icon 库获取图标
                match fs_icon::icon_of_path(&path_str) {
                    Some(data) => Some(format!(
                        "data:image/png;base64,{}",
                        general_purpose::STANDARD.encode(&data)
                    )),
                    None => None,
                }
            } else {
                None
            };

            NodeInfo {
                path: path_str,
                icon,
                metadata: Some(NodeInfoMetadata::from_metadata(&metadata)),
            }
        })
        .collect();

    hilog_debug!(
        "Backend: get_nodes_info returned {} items",
        node_infos.len()
    );
    Ok(node_infos)
}

// 触发重新扫描 - 桩实现
#[napi]
pub async fn trigger_rescan() -> Result<()> {
    println!("Triggering rescan");
    update_lifecycle_state(LifecycleState::Indexing);

    // 延迟模拟重建索引过程
    tokio::time::sleep(Duration::from_secs(2)).await;

    update_lifecycle_state(LifecycleState::Ready);
    println!("Rescan completed");
    Ok(())
}

// 清理后端
#[napi]
pub async fn cleanup_backend() -> Result<()> {
    hilog_debug!("Backend: Cleaning up HarmonyOS backend");
    APP_QUIT.store(true, Ordering::Relaxed);

    static FLUSH_ONCE: Once = Once::new();
    FLUSH_ONCE.call_once(|| {
        let mut state = BACKEND_STATE.write().unwrap();
        let cache_arc = state.search_cache.take().unwrap();
        // 尝试获取唯一所有权
        match Arc::try_unwrap(cache_arc) {
            Ok(cache_lock) => {
                hilog_debug!("Backend: Flush to file 1");
                let cache = cache_lock.into_inner().unwrap();
                hilog_debug!("Backend: Flush to file 2");
                cache.flush_to_file(DB_PATH.get().unwrap()).unwrap();
                hilog_debug!("Backend: Flush to file 3");
            }
            Err(arc_cache) => {
                // 如果还有其他引用，尝试获取写锁并克隆
                hilog_debug!("Backend: Cache still has multiple references, skipping flush");
                // 重新放回去
                state.search_cache = Some(arc_cache);
            }
        }
    });

    update_lifecycle_state(LifecycleState::Uninitialized);
    hilog_debug!("Backend cleanup completed");
    Ok(())
}
