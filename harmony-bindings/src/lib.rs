use napi_derive_ohos::napi;
use napi_ohos::{Error, Result};
use once_cell::sync::{Lazy, OnceCell};
use search_cache::{SearchCache, SearchOptions};
use search_cancel::CancellationToken;
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
    println!("Starting HarmonyOS backend initialization");
    update_lifecycle_state(STATE_INITIALIZING);

    // 初始化数据库路径
    let db_path = PathBuf::from("/data/storage/el1/bundle/cardinal.db");
    dbg!(&db_path);

    DB_PATH
        .set(db_path)
        .map_err(|_| Error::from_reason("Failed to set DB path"))?;

    println!(
        "Starting backend with watch_root: {}, ignore_paths: {}",
        watch_root,
        ignore_paths.len()
    );

    // 在异步任务中运行逻辑线程
    tokio::task::spawn_blocking(move || {
        if let Err(e) = run_logic_thread(watch_root, ignore_paths) {
            println!("Logic thread failed: {}", e);
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

    println!(
        "Attempting to initialize backend with watch_root: {}",
        watch_root
    );
    println!("Database path: {:?}", db_path);
    println!("Ignore paths: {:?}", ignore_paths);

    // 构建搜索缓存
    let watch_path = PathBuf::from(&watch_root);
    let ignore_paths: Vec<PathBuf> = ignore_paths.iter().map(PathBuf::from).collect();

    println!("Building search cache for path: {:?}", watch_path);

    // 创建搜索缓存
    let search_cache = SearchCache::walk_fs(&watch_path);
    println!(
        "Search cache built successfully. Total files: {}",
        search_cache.get_total_files()
    );

    // 更新后端状态
    {
        let mut state = BACKEND_STATE.write().unwrap();
        state.search_cache = Some(Arc::new(RwLock::new(search_cache)));
        state.root_path = Some(watch_path);
        state.lifecycle_state = STATE_READY;
    }

    println!("HarmonyOS backend is ready");
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

    println!("Searching for: '{}'", query);
    println!("Case insensitive: {:?}", case_insensitive);
    println!("Max results: {:?}", max_results);

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

            println!("Search returned {} results", results.len());
            Ok(results)
        }
        Err(e) => {
            println!("Search error: {}", e);
            Err(Error::from_reason(format!("Search failed: {}", e)))
        }
    }
}

// 获取节点信息 - 桩实现
#[napi]
pub async fn get_nodes_info(slab_indices: Vec<u32>) -> Result<Vec<String>> {
    println!("Getting info for {} nodes", slab_indices.len());

    // 桩实现返回模拟信息
    let info: Vec<String> = slab_indices
        .iter()
        .map(|&idx| format!("Node {} info (stub)", idx))
        .collect();

    Ok(info)
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
