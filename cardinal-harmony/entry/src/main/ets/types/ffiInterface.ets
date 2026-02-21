/**
 * FFI接口定义 - 鸿蒙版Cardinal
 * 适配Rust后端接口，支持mock和实际后端切换
 */
// Rust后端生成的原生接口（通过ohos-rs生成）
import nativeBackend, { LifecycleState, NodeInfo } from 'libharmony_bindings.so';

// 搜索参数接口
export interface SearchOptions {
  query: string;
  caseSensitive?: boolean;
  maxResults?: number;
}

// 适配前端显示的结果接口
export interface SearchResult {
  id: string;
  name: string;
  path: string;
  size: string;
  modified: string;
  created?: string;
}

// 文件系统事件接口
export interface FileSystemEvent {
  id: string;
  type: 'CREATE' | 'MODIFY' | 'DELETE' | 'MOVE' | 'ACCESS';
  name: string;
  path: string;
  time: string;
}

// 搜索服务接口
export interface SearchService {
  /**
   * 执行文件搜索
   * @param options 搜索选项
   * @returns 搜索结果数组
   */
  search(options: SearchOptions): Promise<NodeInfo[]>;
  
  /**
   * 获取节点详细信息
   * @param slabIndices 节点索引数组
   * @param includeIcons 是否包含图标
   * @returns 节点信息数组
   */
  getNodesInfo(slabIndices: number[], includeIcons?: boolean): Promise<NodeInfo[]>;
  
  /**
   * 初始化后端
   * @param watchRoot 监听根目录
   * @param ignorePaths 忽略路径数组
   * @returns 初始化状态代码
   */
  initializeBackend(watchRoot: string, ignorePaths: string[], funcSetState: ((err: Error | null, arg: LifecycleState) => void)): Promise<LifecycleState>;

  /**
   * 触发重新扫描
   */
  triggerRescan(): Promise<void>;
  
  /**
   * 清理后端
   */
  cleanupBackend(): Promise<void>;
}

/**
 * 实际Rust后端搜索服务实现
 */
class RustSearchService implements SearchService {
  async search(options: SearchOptions): Promise<NodeInfo[]> {
    console.info(`[Rust Backend] Searching for: ${options.query}`);
    
    try {
      // 调用Rust后端的搜索接口
      const caseInsensitive = !options.caseSensitive;
      const maxResults = options.maxResults || 100;
      
      const slabIndices : Array<number> = await nativeBackend.search(options.query, caseInsensitive, maxResults);
      console.info(`[Rust Backend] Search returned ${slabIndices.length} results`);
      
      // 获取节点详细信息
      if (slabIndices.length > 0) {
        const nodeInfos = await this.getNodesInfo(slabIndices, true);
        return nodeInfos
      }
      
      return [];
    } catch (error) {
      console.error(`[Rust Backend] Search error:`, error);
      return [];
    }
  }
  
  async getNodesInfo(slabIndices: number[], includeIcons: boolean = true): Promise<NodeInfo[]> {
    console.info(`[Rust Backend] Getting node info for ${slabIndices.length} indices`);
    
    try {
      const nodeInfos : Array<NodeInfo> = await nativeBackend.getNodesInfo(slabIndices, includeIcons);
      console.info(`[Rust Backend] Got ${nodeInfos.length} nodes info`);
      return nodeInfos;
    } catch (error) {
      console.error(`[Rust Backend] Get nodes info error:`, error);
      return [];
    }
  }
  
  async initializeBackend(watchRoot: string, ignorePaths: string[], funcSetState: ((err: Error | null, arg: LifecycleState) => void)): Promise<LifecycleState> {
    console.info(`[Rust Backend] Initializing backend with watch root: ${watchRoot}`);
    return await nativeBackend.initializeHarmonyBackend(watchRoot, ignorePaths, funcSetState);
  }

  async triggerRescan(): Promise<void> {
    console.info(`[Rust Backend] Triggering rescan`);
    
    try {
      await nativeBackend.triggerRescan();
    } catch (error) {
      console.error(`[Rust Backend] Trigger rescan error:`, error);
    }
  }
  
  async cleanupBackend(): Promise<void> {
    console.info(`[Rust Backend] Cleaning up backend`);
    
    try {
      await nativeBackend.cleanupBackend();
    } catch (error) {
      console.error(`[Rust Backend] Cleanup backend error:`, error);
    }
  }
  
  // 向下兼容的方法实现
  async getFileDetails(path: string): Promise<FileSystemEvent> {
    console.info(`[Rust Backend] Get file details mock implementation for: ${path}`);
    const fileEvent: FileSystemEvent = {
      id: Date.now().toString(),
      type: 'ACCESS',
      name: path.split('/').pop() || 'file',
      path,
      time: new Date().toISOString()
    };
    return Promise.resolve(fileEvent);
  }
  watchFileSystemEvents(callback: (event: FileSystemEvent) => void): void {
    console.info("[Rust Backend] File system events monitoring mock implementation");
    
    // Mock实现 - 异步模拟事件
    setTimeout(() => {
      const fileEvent: FileSystemEvent = {
        id: Date.now().toString(),
        type: 'CREATE',
        name: `file_${Date.now()}.txt`,
        path: `/data/user/0/example/file_${Date.now()}.txt`,
        time: new Date().toISOString()
      };
      callback(fileEvent);
    }, 1000);
  }
  
  stopWatchingFileSystemEvents(): void {
    console.info("[Rust Backend] Stop watching file system events mock implementation");
  }
}

/**
 * 模拟搜索服务实现（保留用于测试和后端切换）
 */
class MockSearchService implements SearchService {
  async search(options: SearchOptions): Promise<NodeInfo[]> {
    // 打桩实现
    console.info(`[FFI MOCK] Searching for: ${options.query}`);
    
    return new Promise((resolve) => {
      setTimeout(() => {
        if (options.query.trim() === "") {
          const results: SearchResult[] = [
            { id: "1", name: "README.md", path: "/project/README.md", size: "1.2 KB", modified: "2024-09-15" },
            { id: "2", name: "package.json", path: "/project/package.json", size: "2.5 KB", modified: "2024-09-10" },
            { id: "3", name: "src", path: "/project/src/source", size: "-", modified: "2024-09-12" },
            { id: "4", name: "dist", path: "/project/dist", size: "-", modified: "2024-09-14" },
            { id: "5", name: "index.html", path: "/project/index.html", size: "3.1 KB", modified: "2024-09-11" }
          ];
          resolve(results);
        } else {
          const results: SearchResult[] = [
            { id: "1", name: `result_${options.query}_1.txt`, path: `/mock/path/result_${options.query}_1.txt`, size: "1.2 KB", modified: "2024-09-15" },
            { id: "2", name: `result_${options.query}_2.js`, path: `/mock/path/result_${options.query}_2.js`, size: "2.5 KB", modified: "2024-09-10" },
            { id: "3", name: `result_${options.query}_3.json`, path: `/mock/path/result_${options.query}_3.json`, size: "0.8 KB", modified: "2024-09-12" }
          ];
          resolve(results);
        }
      }, 500);
    });
  }
  async getNodesInfo(slabIndices: number[], includeIcons?: boolean): Promise<NodeInfo[]> {
    // Mock实现 - 返回模拟节点信息
    console.info(`[FFI MOCK] Getting node info for ${slabIndices.length} indices`);
    
    const nodeInfos: NodeInfo[] = slabIndices.map((index) => {
      const nodeInfo: NodeInfo = {
        path: `/mock/path/file_${index}.txt`,
        metadata: {
          type: 1, // 文件类型
          size: Math.random() * 10000,
          ctime: Math.floor(Date.now() / 1000) - Math.random() * 1000000,
          mtime: Math.floor(Date.now() / 1000) - Math.random() * 100000
        }
      };
      return nodeInfo;
    });
    return nodeInfos;
  }
  
  async initializeBackend(watchRoot: string, ignorePaths: string[], funcSetState: ((err: Error | null, arg: LifecycleState) => void)): Promise<LifecycleState> {
    console.info(`[FFI MOCK] Initializing mock backend`);
    return Promise.resolve(LifecycleState.Ready); // READY状态
  }

  async triggerRescan(): Promise<void> {
    console.info("[FFI MOCK] Triggering mock rescan");
    return Promise.resolve();
  }
  
  async cleanupBackend(): Promise<void> {
    console.info("[FFI MOCK] Cleaning up mock backend");
    return Promise.resolve();
  }
  
  // 向下兼容的方法实现
  async getFileDetails(path: string): Promise<FileSystemEvent> {
    console.info(`[FFI MOCK] Getting details for: ${path}`);
    
    return new Promise((resolve) => {
      setTimeout(() => {
        resolve({
          id: Date.now().toString(),
          type: 'ACCESS',
          name: path.split('/').pop() || 'file',
          path,
          time: new Date().toISOString()
        });
      }, 200);
    });
  }

  watchFileSystemEvents(callback: (event: FileSystemEvent) => void): void {
    console.info("[FFI MOCK] Starting to watch file system events");
    
    setInterval(() => {
      const mockEvents: FileSystemEvent[] = [
        { id: Date.now().toString(), type: 'CREATE', name: `mock_file_${Date.now()}.txt`, path: `/mock/path/mock_file_${Date.now()}.txt`, time: new Date().toISOString() },
        { id: (Date.now()+1).toString(), type: 'MODIFY', name: `existing_file.js`, path: `/project/existing_file.js`, time: new Date().toLocaleTimeString() }
      ];
      
      mockEvents.forEach(event => callback(event));
    }, 5000);
  }

  stopWatchingFileSystemEvents(): void {
    console.info("[FFI MOCK] Stopping to watch file system events");
  }
}

// 导出搜索服务实例
// 通过注释/解注释下面两行来切换后端实现
export const searchService: SearchService = new RustSearchService();
// export const searchService: SearchService = new MockSearchService();