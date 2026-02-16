
import { AppStateManager, AppState, SearchResult } from '../types/AppStateManager'
import { ResultRow } from './ResultRow'
/**
 * 文件标签页组件
 */
@ComponentV2
export struct FilesTab {
  // 接收父组件传入的参数
  @Param paramSearchQuery: string = ""
  @Param paramCaseSensitive: boolean = false

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
    console.info("FilesTab state updated")
  }

  build() {
    Column() {
      if (this.currentState.isLoading) {
        LoadingProgress()
          .width('100%')
          .height(50)
      } else if (this.currentState.searchResults.length === 0) {
        Text("No results found")
          .fontSize(16)
          .margin({ top: 20 })
          .textAlign(TextAlign.Center)
          .width('100%')
      } else {
        // 使用ForEach组件显示搜索结果
        List({ space: 1 }) {
          ForEach(this.currentState.searchResults, (item: SearchResult, index?: number) => {
            ListItem() {
              ResultRow({
                item:item,
              })
            }
          }, (item: SearchResult, index?: number) => index?.toString());
        }
        .cachedCount(10)
        .edgeEffect(EdgeEffect.Spring)
        .scrollBar(BarState.Auto)
        .layoutWeight(1)
        .width('100%')
      }
    }
    .layoutWeight(1)
    .width('100%')
    .padding({ top: 10 })
  }
  
  /**
   * 加载指定范围的数据
   */
  private loadRange(start: number, end: number): void {
    // 根据可见范围加载数据
    console.info(`Loading range: ${start} to ${end}`)
  }
}