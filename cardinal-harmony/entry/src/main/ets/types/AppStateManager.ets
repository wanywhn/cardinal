import { searchService, SearchOptions } from './ffiInterface'

/**
 * 应用级状态管理器
 * 使用ArkTS状态管理V2实现
 */
// 定义搜索结果类型
export interface SearchResult {
  id: string;
  name: string;
  path: string;
  size: string;
  modified: string;
}

// 定义文件系统事件类型
export interface FileSystemEvent {
  id: string;
  type: string; // CREATE, MODIFY, DELETE, MOVE, ACCESS
  name: string;
  path: string;
  time: string;
}
// 定义应用状态类型
export interface AppState {
  searchQuery: string;
  caseSensitive: boolean;
  isLoading: boolean;
  searchResults: SearchResult[];
  fileSystemEvents: FileSystemEvent[];
  activeTab: number; // 0 for files, 1 for events
}
/**
 * 应用状态管理类
 */
export class AppStateManager {
  private static instance: AppStateManager;
  private state: AppState = {
    searchQuery: "",
    caseSensitive: false,
    isLoading: false,
    searchResults: [],
    fileSystemEvents: [],
    activeTab: 0
  };

  // 观察者列表
  private observers: Array<(state: AppState) => void> = [];

  // 私有构造函数，实现单例模式
  private constructor() {}

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
    return {
      searchQuery: this.state.searchQuery,
      caseSensitive: this.state.caseSensitive,
      isLoading: this.state.isLoading,
      searchResults: [...this.state.searchResults],
      fileSystemEvents: [...this.state.fileSystemEvents],
      activeTab: this.state.activeTab
    }; // 返回副本以防止直接修改
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
   * 执行搜索操作
   * 通过FFI调用原生搜索功能
   */
  public async performSearch(query: string): Promise<void> {
    this.setLoading(true);

    try {
      // 通过FFI调用原生搜索功能
      const searchOptions: SearchOptions = {
        query: query,
        caseSensitive: this.state.caseSensitive,
        maxResults: 100
      };
      
      const results = await searchService.search(searchOptions);
      this.setSearchResults(results);
    } catch (error) {
      console.error("Search failed:", error);
      // 设置错误状态或默认结果
      this.setSearchResults([]);
    }
  }

  /**
   * 加载文件系统事件
   * 通过FFI调用原生事件监听功能
   */
  public startFileSystemEventListening(): void {
    // 通过FFI调用原生事件监听功能
    searchService.watchFileSystemEvents((event: FileSystemEvent) => {
      // 将新事件添加到现有事件列表中，保持固定长度以优化Repeat组件性能
      const updatedEvents = [event, ...this.state.fileSystemEvents];
      // 限制事件数量以优化性能
      if (updatedEvents.length > 1000) {
        updatedEvents.length = 1000; // 只保留最新的1000个事件
      }
      this.state.fileSystemEvents = [...updatedEvents];
      this.notify();
    });
  }
  
  /**
   * 停止文件系统事件监听
   */
  public stopFileSystemEventListening(): void {
    searchService.stopWatchingFileSystemEvents();
  }
}