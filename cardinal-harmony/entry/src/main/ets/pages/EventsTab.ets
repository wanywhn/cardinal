
import { AppStateManager, AppState, FileSystemEvent } from '../types/AppStateManager'
import { EventRow } from './EventRow'
/**
 * 事件标签页组件
 */
@ComponentV2
export struct EventsTab {
  private appState: AppStateManager = AppStateManager.getInstance()
  private currentState: AppState = this.appState.getState()

  aboutToAppear(): void {
    // 开始监听文件系统事件
    this.appState.startFileSystemEventListening()
    // 初始化状态
    this.currentState = this.appState.getState()
  }

  build() {
    Column() {
      if (this.currentState.fileSystemEvents.length === 0) {
        Text("No events recorded")
          .fontSize(16)
          .margin({ top: 20 })
          .textAlign(TextAlign.Center)
          .width('100%')
      } else {
        // 使用ForEach组件显示事件
        List({ space: 1 }) {
          ForEach(this.currentState.fileSystemEvents, (event: FileSystemEvent, index?: number) => {
            ListItem() {
              EventRow({
                event: event
              })
            }
          }, (event: FileSystemEvent, index?: number) => index?.toString());
        }
        .cachedCount(10)
        .edgeEffect(EdgeEffect.Spring)
        .scrollBar(BarState.Auto)
      }
    }
    .layoutWeight(1)
    .width('100%')
    .padding({ top: 10 })
  }
}