/**
 * 搜索状态管理类
 */
class SearchState {
  private static instance: SearchState;
  private _searchQuery: string = "";
  private _caseSensitive: boolean = false;
  private _isLoading: boolean = false;
  private _searchResults: Array<any> = [];
  private _events: Array<any> = [];

  // 单例模式
  public static getInstance(): SearchState {
    if (!SearchState.instance) {
      SearchState.instance = new SearchState();
    }
    return SearchState.instance;
  }

  // Getter 和 Setter 方法
  get searchQuery(): string {
    return this._searchQuery;
  }

  set searchQuery(value: string) {
    this._searchQuery = value;
    // 触发状态更新通知
    this.notifyObservers();
  }

  get caseSensitive(): boolean {
    return this._caseSensitive;
  }

  set caseSensitive(value: boolean) {
    this._caseSensitive = value;
    this.notifyObservers();
  }

  get isLoading(): boolean {
    return this._isLoading;
  }

  set isLoading(value: boolean) {
    this._isLoading = value;
    this.notifyObservers();
  }

  get searchResults(): Array<any> {
    return this._searchResults;
  }

  set searchResults(value: Array<any>) {
    this._searchResults = value;
    this.notifyObservers();
  }

  get events(): Array<any> {
    return this._events;
  }

  set events(value: Array<any>) {
    this._events = value;
    this.notifyObservers();
  }

  // 添加观察者模式支持
  private observers: Array<() => void> = [];

  addObserver(observer: () => void): void {
    this.observers.push(observer);
  }

  removeObserver(observer: () => void): void {
    const index = this.observers.indexOf(observer);
    if (index > -1) {
      this.observers.splice(index, 1);
    }
  }

  private notifyObservers(): void {
    this.observers.forEach(observer => observer());
  }

  /**
   * 执行搜索操作
   */
  async performSearch(query: string): Promise<void> {
    this.isLoading = true;
    
    // 打桩：实际的搜索功能将通过FFI调用实现
    console.info(`Performing search for: ${query}`);
    
    // 模拟异步搜索
    setTimeout(() => {
      if (query.trim() === "") {
        this.searchResults = this.getDefaultResults();
      } else {
        this.searchResults = this.getMockSearchResults(query);
      }
      this.isLoading = false;
    }, 500);
  }

  /**
   * 获取默认结果
   */
  private getDefaultResults(): Array<any> {
    return [
      { id: "1", name: "README.md", path: "/project/README.md", size: "1.2 KB", modified: "2024-09-15" },
      { id: "2", name: "package.json", path: "/project/package.json", size: "2.5 KB", modified: "2024-09-10" },
      { id: "3", name: "src", path: "/project/src", size: "-", modified: "2024-09-12" },
      { id: "4", name: "dist", path: "/project/dist", size: "-", modified: "2024-09-14" },
      { id: "5", name: "index.html", path: "/project/index.html", size: "3.1 KB", modified: "2024-09-11" }
    ];
  }

  /**
   * 获取模拟搜索结果
   */
  private getMockSearchResults(query: string): Array<any> {
    return [
      { id: "1", name: `result_${query}_1.txt`, path: `/mock/path/result_${query}_1.txt`, size: "1.2 KB", modified: "2024-09-15" },
      { id: "2", name: `result_${query}_2.js`, path: `/mock/path/result_${query}_2.js`, size: "2.5 KB", modified: "2024-09-10" },
      { id: "3", name: `result_${query}_3.json`, path: `/mock/path/result_${query}_3.json`, size: "0.8 KB", modified: "2024-09-12" }
    ];
  }

  /**
   * 加载事件数据
   */
  async loadEvents(): Promise<void> {
    // 打桩：实际的事件加载功能将通过FFI调用实现
    console.info("Loading events...");
    
    // 模拟异步加载
    setTimeout(() => {
      this.events = [
        { id: "1", type: "CREATE", name: "new_file.txt", path: "/documents/new_file.txt", time: "10:30:15" },
        { id: "2", type: "MODIFY", name: "existing_file.js", path: "/project/existing_file.js", time: "10:29:42" },
        { id: "3", type: "DELETE", name: "temp.log", path: "/logs/temp.log", time: "10:28:33" },
        { id: "4", type: "MOVE", name: "moved_file.txt", path: "/new_location/moved_file.txt", time: "10:27:51" },
        { id: "5", type: "ACCESS", name: "readme.md", path: "/project/readme.md", time: "10:26:17" }
      ];
    }, 500);
  }
}