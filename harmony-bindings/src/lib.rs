use base64::{engine::general_purpose, Engine as _};
use fs_icon;
use napi_derive_ohos::napi;
use napi_ohos::{Error, Result};
use ohos_fileuri_binding::get_path_from_uri;
use ohos_hilog_binding::hilog_debug;
use once_cell::sync::{Lazy, OnceCell};
use search_cache::{SearchCache, SearchOptions, SearchResultNode, SlabNodeMetadataCompact};
use search_cancel::CancellationToken;
use serde::Serialize;
use std::{
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, RwLock,
    },
    time::Duration,
};

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

// 后端状态
struct BackendState {
    lifecycle_state: u8,
    search_cache: Option<Arc<RwLock<SearchCache>>>,
    root_path: Option<PathBuf>,
}

impl BackendState {
    fn new() -> Self {
        Self {
            lifecycle_state: STATE_UNINITIALIZED,
            search_cache: None,
            root_path: None,
        }
    }
}

// 生命周期状态常量
pub const STATE_UNINITIALIZED: u8 = 0;
pub const STATE_INITIALIZING: u8 = 1;
pub const STATE_INDEXING: u8 = 2;
pub const STATE_READY: u8 = 3;
pub const STATE_UPDATING: u8 = 4;
pub const STATE_ERROR: u8 = 5;

// 鸿蒙后端初始化主函数
#[napi]
pub async fn initialize_harmony_backend(
    watch_root: String,
    ignore_paths: Vec<String>,
) -> Result<u8> {
    hilog_debug!("Backend: Starting HarmonyOS backend initialization");
    update_lifecycle_state(STATE_INITIALIZING);

    // 初始化数据库路径
    let db_path = PathBuf::from("com.wanywhn.anything/data/storage/el2/cardinal.db");

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
    // 在异步任务中运行逻辑线程
    tokio::task::spawn_blocking(move || {
        if let Err(e) = run_logic_thread(root_path, ignore_paths) {
            hilog_debug!("Backend: Logic thread failed: {}", e);
            update_lifecycle_state(STATE_ERROR);
        }
    });

    // 立即返回索引中状态
    update_lifecycle_state(STATE_INDEXING);
    Ok(STATE_INDEXING)
}

// 获取应用状态
#[napi]
pub fn get_app_status() -> u8 {
    BACKEND_STATE.read().unwrap().lifecycle_state
}

// 更新生命周期状态
fn update_lifecycle_state(new_state: u8) {
    let mut state = BACKEND_STATE.write().unwrap();
    state.lifecycle_state = new_state;
    println!("Lifecycle state changed to: {}", state_to_string(new_state));
}

// 状态到字符串转换
fn state_to_string(state: u8) -> &'static str {
    match state {
        STATE_UNINITIALIZED => "UNINITIALIZED",
        STATE_INITIALIZING => "INITIALIZING",
        STATE_INDEXING => "INDEXING",
        STATE_READY => "READY",
        STATE_UPDATING => "UPDATING",
        STATE_ERROR => "ERROR",
        _ => "UNKNOWN",
    }
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

    // 创建搜索缓存
    let search_cache = SearchCache::walk_fs(&watch_path);
    hilog_debug!(
        "Backend: Search cache built successfully. Total files: {}",
        search_cache.get_total_files()
    );

    // 更新后端状态
    {
        let mut state = BACKEND_STATE.write().unwrap();
        state.search_cache = Some(Arc::new(RwLock::new(search_cache)));
        state.root_path = Some(watch_path);
        state.lifecycle_state = STATE_READY;
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

    if state.lifecycle_state != STATE_READY {
        return Err(Error::from_reason(format!(
            "Backend not ready. Current state: {}",
            state_to_string(state.lifecycle_state)
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

    if state.lifecycle_state != STATE_READY {
        return Err(Error::from_reason(format!(
            "Backend not ready. Current state: {}",
            state_to_string(state.lifecycle_state)
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
    update_lifecycle_state(STATE_INDEXING);

    // 延迟模拟重建索引过程
    tokio::time::sleep(Duration::from_secs(2)).await;

    update_lifecycle_state(STATE_READY);
    println!("Rescan completed");
    Ok(())
}

// 清理后端
#[napi]
pub async fn cleanup_backend() -> Result<()> {
    println!("Cleaning up HarmonyOS backend");
    APP_QUIT.store(true, Ordering::Relaxed);

    update_lifecycle_state(STATE_UNINITIALIZED);
    println!("Backend cleanup completed");
    Ok(())
}
