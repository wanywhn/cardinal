
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
  private currentState: AppState = this.appState.getState()
  
  aboutToAppear(): void {
    // 订阅状态变化
    this.appState.subscribe(this.updateUI.bind(this))
    // 初始化状态
    this.currentState = this.appState.getState()
  }
  updateUI(state: AppState): void {
    // 当状态变化时更新UI
    this.currentState = state
    console.info("State updated, refreshing UI")
  }
  build() {
    Column() {
      // 搜索栏
      SearchBar({
        queryChangeCallback: (query: string) => {
          this.appState.setSearchQuery(query)
          // TODO: 实现搜索功能
          this.handleSearch(query)
        },
        toggleCaseSensitiveCallback: (value: boolean) => {
          if (value !== this.currentState.caseSensitive) {
            this.appState.toggleCaseSensitive()
          }
        }
      }).height(50)
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
    }
    .height('95%')
    .backgroundColor($r('app.color.main_background_color'))
  }
  
  /**
   * 处理搜索请求
   */
  private handleSearch(query: string): void {
    // 打桩：实际的搜索功能将通过FFI调用实现
    console.info(`Searching for: ${query}`)
    this.appState.performSearch(query)
  }
}