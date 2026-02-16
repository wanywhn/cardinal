/**
 * FFI接口定义 - 鸿蒙版Cardinal
 * 此文件定义了将来要实现的原生功能接口
 */
// 搜索参数接口
export interface SearchOptions {
  query: string;
  caseSensitive?: boolean;
  maxResults?: number;
}

// 搜索结果接口
interface SearchResult {
  id: string;
  name: string;
  path: string;
  size: string;
  modified: string;
  created?: string;
}

// 文件系统事件接口
interface FileSystemEvent {
  id: string;
  type: 'CREATE' | 'MODIFY' | 'DELETE' | 'MOVE' | 'ACCESS';
  name: string;
  path: string;
  time: string;
}

// 搜索服务接口
interface SearchService {
  /**
   * 执行文件搜索
   * @param options 搜索选项
   * @returns 搜索结果数组
   */
  search(options: SearchOptions): Promise<SearchResult[]>;
  /**
   * 获取文件详细信息
   * @param path 文件路径
   * @returns 文件详细信息
   */
  getFileDetails(path: string): Promise<FileSystemEvent>;

  /**
   * 监听文件系统事件
   * @param callback 事件回调函数
   */
  watchFileSystemEvents(callback: (event: FileSystemEvent) => void): void;

  /**
   * 停止监听文件系统事件
   */
  stopWatchingFileSystemEvents(): void;
}

/**
 * 搜索服务实现（打桩）
 * 实际实现将在原生层通过FFI提供
 */
class MockSearchService implements SearchService {
  async search(options: SearchOptions): Promise<SearchResult[]> {
    // 打桩实现
    console.info(`[FFI MOCK] Searching for: ${options.query}`);
    
    return new Promise((resolve) => {
      setTimeout(() => {
        if (options.query.trim() === "") {
          resolve([
            { id: "1", name: "README.md", path: "/project/README.md", size: "1.2 KB", modified: "2024-09-15" },
            { id: "2", name: "package.json", path: "/project/package.json", size: "2.5 KB", modified: "2024-09-10" },
            { id: "3", name: "src", path: "/project/src", size: "-", modified: "2024-09-12" },
            { id: "4", name: "dist", path: "/project/dist", size: "-", modified: "2024-09-14" },
            { id: "5", name: "index.html", path: "/project/index.html", size: "3.1 KB", modified: "2024-09-11" }
          ]);
        } else {
          resolve([
            { id: "1", name: `result_${options.query}_1.txt`, path: `/mock/path/result_${options.query}_1.txt`, size: "1.2 KB", modified: "2024-09-15" },
            { id: "2", name: `result_${options.query}_2.js`, path: `/mock/path/result_${options.query}_2.js`, size: "2.5 KB", modified: "2024-09-10" },
            { id: "3", name: `result_${options.query}_3.json`, path: `/mock/path/result_${options.query}_3.json`, size: "0.8 KB", modified: "2024-09-12" }
          ]);
        }
      }, 500);
    });
  }
  async getFileDetails(path: string): Promise<FileSystemEvent> {
    // 打桩实现
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
    // 打桩实现 - 模拟事件
    console.info("[FFI MOCK] Starting to watch file system events");
    
    // 模拟定期产生事件
    setInterval(() => {
      const mockEvents: FileSystemEvent[] = [
        { id: Date.now().toString(), type: 'CREATE', name: `mock_file_${Date.now()}.txt`, path: `/mock/path/mock_file_${Date.now()}.txt`, time: new Date().toISOString() },
        { id: (Date.now()+1).toString(), type: 'MODIFY', name: `existing_file.js`, path: `/project/existing_file.js`, time: new Date().toLocaleTimeString() }
      ];
      
      mockEvents.forEach(event => {
        callback(event);
      });
    }, 50000); // 每5秒发送一次模拟事件
  }

  stopWatchingFileSystemEvents(): void {
    // 打桩实现
    console.info("[FFI MOCK] Stopping to watch file system events");
  }
}

// 导出搜索服务实例
export const searchService: SearchService = new MockSearchService();