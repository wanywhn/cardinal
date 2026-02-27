import { searchService, SearchOptions, SearchService } from './ffiInterface'
import { hilog } from '@kit.PerformanceAnalysisKit';
import App from '@system.app';
import { abilityAccessCtrl, bundleManager, Permissions } from '@kit.AbilityKit';
import { Environment, fileUri } from '@kit.CoreFileKit';
import { LifecycleState, NodeInfo } from 'libharmony_bindings.so';

// 定义文件系统事件类型
export interface FileSystemEvent {
  id: string;
  type: string; // CREATE, MODIFY, DELETE, MOVE, ACCESS
  name: string;
  path: string;
  time: string;
}

// 后端初始化配置
const DEFAULT_WATCH_ROOT: string = "/data/app/el2/100/base/com.wanywhn.anything/";
const DEFAULT_IGNORE_PATHS: string[] = [
  "node_modules",
  ".git",
  ".idea",
  ".vscode",
  "target",
  "build",
  "dist"
];

// 定义应用状态类型
@ObservedV2
export class AppState {
  @Trace public searchQuery: string = "";
  @Trace public caseSensitive: boolean = false;
  @Trace public backendInitialized: boolean = false;
  @Trace public backendStatus: number = LifecycleState.Uninitialized;
  @Trace searchResults: NodeInfo[] = [];
  @Trace public fileSystemEvents: FileSystemEvent[] = [];
}

/**
 * 应用状态管理类
 */
export class AppStateManager {
  private static instance: AppStateManager;
  private state: AppState = new AppState()
  private dbPath?: string;
  private isInitialized = false;

  // 私有构造函数，实现单例模式
  private constructor() {
    // 不在构造函数中自动初始化，由外部显式调用 init()
  }

  /**
   * 设置应用文件目录 - 在 Ability 中调用
   * @param dir 应用文件目录路径
   */
  public setFilesDir(dir: string): void {
    this.dbPath = dir;
    console.info(`FilesDir set to: ${dir}`);
  }

  /**
   * 初始化后端 - 在设置完 dbPath 后调用
   */
  public init(): void {
    if (this.isInitialized) {
      console.info('AppStateManager already initialized');
      return;
    }
    this.isInitialized = true;
    this.initializeBackend();
  }

  // 获取单例实例
  public static getInstance(): AppStateManager {
    if (!AppStateManager.instance) {
      AppStateManager.instance = new AppStateManager();
    }
    return AppStateManager.instance;
  }

  // 获取当前状态
  public getState(): AppState {
    return this.state;
  }

  // 更新搜索查询
  public setSearchQuery(query: string): void {
    this.state.searchQuery = query;
  }

  // 切换大小写敏感
  public toggleCaseSensitive(): void {
    this.state.caseSensitive = !this.state.caseSensitive;
  }

  // 更新后端状态
  public setBackendStatus(status: LifecycleState): void {
    this.state.backendStatus = status;
    this.state.backendInitialized = (status === LifecycleState.Ready);
    if (status == LifecycleState.Ready) {
      this.performSearch("")

    }
  }

  // 更新搜索结果
  public setSearchResults(results: NodeInfo[]): void {
    this.state.searchResults = results;
  }

  // 更新文件系统事件
  public setFileSystemEvents(events: FileSystemEvent[]): void {
    this.state.fileSystemEvents = [...events]; // 创建新数组以触发更新
  }

  /**
   * 初始化后端服务
   */
  public async initializeBackend(): Promise<void> {
    if (this.state.backendInitialized) {
      console.info("Backend already initialized");
      return;
    }

    try {
      this.setBackendStatus(LifecycleState.Initializing);
      console.info("Initializing backend...");

      if (!canIUse('SystemCapability.FileManagement.File.Environment.FolderObtain')) {
        console.error('this api is not supported on this device');
        // return;
      }
      // const docuri = Environment.getUserDocumentDir();
      // console.info(`docuri is ${docuri}`)

      const bind_func = (err: Error | null, arg: LifecycleState): void => this.setBackendStatus(arg);

      const status = await searchService.initializeBackend(
        "/storage/Users/currentUser",
        DEFAULT_IGNORE_PATHS,
        this.dbPath + "/cardianl.db",
        bind_func
      );
    } catch (error) {
      console.error(`Backend initialization failed:`, error);
      this.setBackendStatus(LifecycleState.Error);
    }
  }

  /**
   * 获取当前后端状态文本描述
   */
  public getBackendStatusText(): string {
    switch (this.state.backendStatus) {
      case LifecycleState.Uninitialized:
        return "未初始化";
      case LifecycleState.Initializing:
        return "初始化中...";
      case LifecycleState.Indexing:
        return "索引构建中...";
      case LifecycleState.Ready:
        return "就绪";
      case LifecycleState.Updating:
        return "更新中...";
      case LifecycleState.Error:
        return "错误";
      default:
        return "未知状态";
    }
  }

  /**
   * 执行搜索操作
   * 通过FFI调用原生搜索功能
   */
  public async performSearch(query: string): Promise<void> {
    if (!this.state.backendInitialized) {
      console.info("Backend not ready for search, ignoring request");
      return;
    }

    try {
      const searchOptions: SearchOptions = {
        query: query,
        caseSensitive: this.state.caseSensitive,
        maxResults: 100
      };
      
      console.info(`Starting search for: "${query}"`);
      const results = await searchService.search(searchOptions);
      console.info(`Search completed, found ${results.length} results`);
      
      this.setSearchResults(results);
    } catch (error) {
      console.error("Search failed:", error);
      this.setSearchResults([]);
    }
  }

  /**
   * 触发重新扫描
   */
  public async triggerRescan(): Promise<void> {
    try {
      console.info("Triggering backend rescan...");
      this.setBackendStatus(LifecycleState.Indexing);
      await searchService.triggerRescan();
      this.setBackendStatus(LifecycleState.Ready);
      console.info("Rescan completed");
    } catch (error) {
      console.error("Rescan failed:", error);
      this.setBackendStatus(LifecycleState.Error);
    }
  }

  /**
   * 加载文件系统事件
   * 通过FFI调用原生事件监听功能
   */
  public startFileSystemEventListening(): void {
    if (!this.state.backendInitialized) {
      console.info("Backend not ready for event listening");
      return;
    }

  }
  
  /**
   * 停止文件系统事件监听
   */
  public stopFileSystemEventListening(): void {
    try {
      // searchService.stopWatchingFileSystemEvents();
      console.info("File system event listening stopped");
    } catch (error) {
      console.error("Failed to stop file system event listening:", error);
    }
  }

  /**
   * 清理应用状态和资源
   */
  public async cleanup(): Promise<void> {
    try {
      this.stopFileSystemEventListening();
      await searchService.cleanupBackend();
      // this.state.backendInitialized = false;
      // this.setBackendStatus(LifecycleState.Uninitialized);
      console.info("Application cleanup completed");
    } catch (error) {
      console.error("Cleanup failed:", error);
    }
  }
}

async function checkPermissionGrant(permission: Permissions): Promise<abilityAccessCtrl.GrantStatus> {
  let atManager: abilityAccessCtrl.AtManager = abilityAccessCtrl.createAtManager();
  let grantStatus: abilityAccessCtrl.GrantStatus = abilityAccessCtrl.GrantStatus.PERMISSION_DENIED;

  // 获取应用程序的accessTokenID
  let tokenId: number = 0;
  try {
    let bundleInfo: bundleManager.BundleInfo =
      await bundleManager.getBundleInfoForSelf(bundleManager.BundleFlag.GET_BUNDLE_INFO_WITH_APPLICATION);
    let appInfo: bundleManager.ApplicationInfo = bundleInfo.appInfo;
    tokenId = appInfo.accessTokenId;
  } catch (error) {
    const err: BusinessError = error as BusinessError;
    console.error(`Failed to get bundle info for self, code: ${err.code}, message: ${err.message}`);
  }

  // 校验应用是否被授予权限
  try {
    grantStatus = await atManager.checkAccessToken(tokenId, permission);
  } catch (error) {
    const err: BusinessError = error as BusinessError;
    console.error(`Failed to check access token, code: ${err.code}, message: ${err.message}`);
  }

  return grantStatus;
}
