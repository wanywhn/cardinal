import { searchService, SearchOptions, SearchService } from './ffiInterface'
import { hilog } from '@kit.PerformanceAnalysisKit';
import App from '@system.app';
import { abilityAccessCtrl, bundleManager, Permissions } from '@kit.AbilityKit';
import { Environment, fileUri } from '@kit.CoreFileKit';

/**
 * 应用级状态管理器
 * 使用ArkTS状态管理V2实现
 */
// 定义搜索结果类型（适配ArkUI渲染）
@ObservedV2
export class SearchResult {
  @Trace id: string = "1";
  @Trace name: string = "2";
  @Trace path: string = "3";
  @Trace size: string = "4";
  @Trace modified: string = "5";
}

// 定义文件系统事件类型
export interface FileSystemEvent {
  id: string;
  type: string; // CREATE, MODIFY, DELETE, MOVE, ACCESS
  name: string;
  path: string;
  time: string;
}

// 应用生命周期状态常量
const STATE_UNINITIALIZED: number = 0;
const STATE_INITIALIZING: number = 1;
const STATE_INDEXING: number = 2;
const STATE_READY: number = 3;
const STATE_UPDATING: number = 4;
const STATE_ERROR: number = 5;

// 后端初始化配置
const DEFAULT_WATCH_ROOT: string = "/data/app/el2/100/base/com.wanywhn.anything";
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
  @Trace public isLoading: boolean = false;
  @Trace public backendInitialized: boolean = false;
  @Trace public backendStatus: string = STATE_UNINITIALIZED.toString();
  @Trace public searchResults: SearchResult[] = [];
  @Trace public fileSystemEvents: FileSystemEvent[] = [];
  @Trace public activeTab: number = 0; // 0 for files, 1 for events
}

/**
 * 应用状态管理类
 */
export class AppStateManager {
  private static instance: AppStateManager;
  private state: AppState = new AppState()

  // 观察者列表
  private observers: Array<(state: AppState) => void> = [];

  // 后端初始化状态跟踪
  private backendInitialized: boolean = false;

  // 私有构造函数，实现单例模式
  private constructor() {
    // 应用启动时延迟初始化后端
    this.delayedInitializeBackend();
  }

  // 获取单例实例
  public static getInstance(): AppStateManager {
    if (!AppStateManager.instance) {
      AppStateManager.instance = new AppStateManager();
    }
    return AppStateManager.instance;
  }

  // 订阅状态变化
  public subscribe(observer: (state: AppState) => void): void {
    this.observers.push(observer);
  }

  // 取消订阅
  public unsubscribe(observer: (state: AppState) => void): void {
    const index = this.observers.indexOf(observer);
    if (index > -1) {
      this.observers.splice(index, 1);
    }
  }

  // 通知所有观察者状态变化
  private notify(): void {
    this.observers.forEach(observer => observer(this.state));
  }

  // 获取当前状态
  public getState(): AppState {
    return this.state;
  }

  // 更新搜索查询
  public setSearchQuery(query: string): void {
    this.state.searchQuery = query;
    this.notify();
  }

  // 切换大小写敏感
  public toggleCaseSensitive(): void {
    this.state.caseSensitive = !this.state.caseSensitive;
    this.notify();
  }

  // 设置加载状态
  public setLoading(loading: boolean): void {
    this.state.isLoading = loading;
    this.notify();
  }

  // 更新后端状态
  public setBackendStatus(status: number): void {
    this.state.backendStatus = status.toString();
    this.state.backendInitialized = (status === STATE_READY);
    this.notify();
  }

  // 更新搜索结果
  public setSearchResults(results: SearchResult[]): void {
    this.state.searchResults = [...results]; // 创建新数组以触发更新
    this.state.isLoading = false;
    this.notify();
  }

  // 更新文件系统事件
  public setFileSystemEvents(events: FileSystemEvent[]): void {
    this.state.fileSystemEvents = [...events]; // 创建新数组以触发更新
    this.notify();
  }

  // 切换活动标签页
  public setActiveTab(tabIndex: number): void {
    this.state.activeTab = tabIndex;
    this.notify();
  }

  /**
   * 延迟初始化后端 - 在应用启动后异步执行
   */
  private async delayedInitializeBackend(): Promise<void> {
    // 等待UI初始化完成
    setTimeout(async () => {
      await this.initializeBackend();
    }, 1000); // 延迟1秒开始初始化
  }

  /**
   * 初始化后端服务
   */
  public async initializeBackend(): Promise<void> {
    if (this.backendInitialized) {
      console.info("Backend already initialized");
      return;
    }

    try {
      this.setBackendStatus(STATE_INITIALIZING);
      console.info("Initializing backend...");

      if (!canIUse('SystemCapability.FileManagement.File.Environment.FolderObtain')) {
        console.error('this api is not supported on this device');
        // return;
      }

      await checkPermissions()
      const docuri = Environment.getUserDocumentDir();
      console.info(`docuri is ${docuri}`)
      const status = await searchService.initializeBackend(
        // DEFAULT_WATCH_ROOT,
        // docuri,
        "/storage/Users/currentUser",
        DEFAULT_IGNORE_PATHS
      );
      
      this.setBackendStatus(status);

      this.backendInitialized = true;
      console.info("Backend initialized successfully");

      if (status === STATE_INDEXING) {

        
        // 后端初始化完成后，执行空的搜索以测试连接
        // this.performSearch("");
      } else {
        console.info(`Backend initialization completed with status: ${status}`);
      }
    } catch (error) {
      console.error(`Backend initialization failed:`, error);
      this.setBackendStatus(STATE_ERROR);
    }
  }

  /**
   * 获取当前后端状态文本描述
   */
  public getBackendStatusText(): string {
    switch (parseInt(this.state.backendStatus)) {
      case STATE_UNINITIALIZED:
        return "未初始化";
      case STATE_INITIALIZING:
        return "初始化中...";
      case STATE_INDEXING:
        return "索引构建中...";
      case STATE_READY:
        return "就绪";
      case STATE_UPDATING:
        return "更新中...";
      case STATE_ERROR:
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
    if (!this.backendInitialized) {
      console.info("Backend not ready for search, ignoring request");
      return;
    }

    this.setLoading(true);

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
      this.setBackendStatus(STATE_INDEXING);
      await searchService.triggerRescan();
      this.setBackendStatus(STATE_READY);
      console.info("Rescan completed");
    } catch (error) {
      console.error("Rescan failed:", error);
      this.setBackendStatus(STATE_ERROR);
    }
  }

  /**
   * 加载文件系统事件
   * 通过FFI调用原生事件监听功能
   */
  public startFileSystemEventListening(): void {
    if (!this.backendInitialized) {
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
      this.backendInitialized = false;
      this.setBackendStatus(STATE_UNINITIALIZED);
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

async function checkPermissions(): Promise<void> {
  let grantStatus: boolean = await checkPermissionGrant('ohos.permission.READ_WRITE_DOCUMENTS_DIRECTORY') === abilityAccessCtrl.GrantStatus.PERMISSION_GRANTED;// 获取拦截键盘输入事件权限状态。
  if (grantStatus) {
    // 已经授权，可以继续访问目标操作
    console.info(`permission is granted.`);
  } else {
    // 未授权，引导用户跳转到系统应用“设置”中进行授权
    console.info(`permission is not granted.`);
  }
}