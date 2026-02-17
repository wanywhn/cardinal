
import { AppStateManager, AppState } from '../types/AppStateManager'
import { SearchBar } from './SearchBar'
import { FilesTab } from './FilesTab'
import { EventsTab } from './EventsTab'

/**
 * 主页面组件
 */
@Entry
@ComponentV2
struct Index {
  private appState: AppStateManager = AppStateManager.getInstance()
  @Local currentState: AppState = this.appState.getState()
  
  aboutToAppear(): void {
    // 订阅状态变化
    this.appState.subscribe(this.updateUI.bind(this))
    // 初始化状态
    this.currentState = this.appState.getState()
  }

  aboutToDisappear(): void {
    // 清理资源
    this.appState.cleanup()
  }

  updateUI(state: AppState): void {
    // 当状态变化时更新UI
    this.currentState = state
    console.info("State updated, refreshing UI")
  }

  build() {
    Column() {
      // 后端状态栏
      this.StatusBar()

      // 搜索栏
      SearchBar({
        queryChangeCallback: (query: string) => {
          this.handleSearch(query)
          this.appState.setSearchQuery(query)
        },
        toggleCaseSensitiveCallback: (value: boolean) => {
          if (value !== this.currentState.caseSensitive) {
            this.appState.toggleCaseSensitive()
          }
        }
      }).height(50)

      // 搜索结果统计
      if (this.currentState.searchQuery || this.currentState.isLoading) {
        this.ResultsInfo()
      }

      // 标签页切换
      Tabs({ barPosition: BarPosition.Start }) {
        TabContent() {
          FilesTab()
        }
        .tabBar("Files")
        TabContent() {
          EventsTab()
        }
        .tabBar("Events")
      }
      .barPosition(BarPosition.End)
      .vertical(false)
      .barMode(BarMode.Fixed)
      .onChange((index: number) => {
        this.appState.setActiveTab(index)
      })

      // 控制按钮
      this.ControlButtons()
    }
    .height('80%')
    .backgroundColor($r('app.color.main_background_color'))
  }

  /**
   * 状态栏构建器
   */
  @Builder
  StatusBar() {
    Row() {
      Text(`后端状态: ${this.appState.getBackendStatusText()}`)
        .fontSize(12)

      Blank()
      
      // 重新扫描按钮
      if (this.currentState.backendInitialized) {
        Button("重新扫描").onClick(()=> {
          this.appState.triggerRescan()
        })
        .fontSize(10)
        .height(24)
        .backgroundColor(Color.Blue)
        .fontColor(Color.White)
      }
    }
    .width('100%')
    .padding({ left: 16, right: 16, top: 8, bottom: 8 })
  }

  /**
   * 搜索结果信息构建器
   */
  @Builder
  ResultsInfo() {
    Row() {
      if (this.currentState.isLoading) {
        LoadingProgress()
          .width(16)
          .height(16)
        
        Text("搜索中...")
          .fontSize(12)
          .margin({ left: 8 })
      } else {
        Text(`找到 ${this.currentState.searchResults.length} 个结果`)
          .fontSize(12)
          .fontColor($r('app.color.text_secondary_color'))
      }
      
      Blank()
      
      if (this.currentState.searchQuery) {
        Text(`搜索词: "${this.currentState.searchQuery}"`)
          .fontSize(10)
      }
    }
    .width('100%')
    .padding({ left: 16, right: 16, top: 4, bottom: 4 })
  }

  /**
   * 控制按钮构建器
   */
  @Builder
  ControlButtons() {
    Row({ space: 10 }) {
      Button("开始监听事件").onClick(()=>{
        this.appState.startFileSystemEventListening()
      })
      .enabled(this.currentState.backendInitialized)
      
      Button("停止监听事件").onClick(()=>{
        this.appState.stopFileSystemEventListening()
      })
      .enabled(this.currentState.backendInitialized)
    }
    .width('100%')
    .padding(16)
    .justifyContent(FlexAlign.Center)
  }
  
  /**
   * 处理搜索请求
   */
  private handleSearch(query: string): void {
    console.info(`Searching for: ${query}`)
    this.appState.performSearch(query)
  }
}