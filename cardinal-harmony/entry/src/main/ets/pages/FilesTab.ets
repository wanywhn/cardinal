import { AppStateManager, AppState } from '../types/AppStateManager'
import { NodeInfo, NodeFileType } from 'libharmony_bindings.so';

/**
 * 文件标签页组件
 */
@ComponentV2
export struct FilesTab {
  private appState: AppStateManager = AppStateManager.getInstance()
  currentState: AppState = this.appState.getState()

  aboutToAppear(): void {
    // 订阅状态变化
    this.appState.subscribe(this.updateUI.bind(this))
    // 初始化状态
    this.currentState = this.appState.getState()
  }
  updateUI(state: AppState): void {
    // 当状态变化时更新UI
    this.currentState = state
  }

  @Builder
  ResultRow(ri:RepeatItem<NodeInfo>) {
    Row({space: 1}) {
      // 图标占位
      // Image($r('app.media.icon_file'))
      //   .width(24)
      //   .height(24)
      //   .margin({ right: 10 })
      Text(ri.item.metadata?.type.toString())
        .margin({right: 10})

      // 文件信息列
      // Column({ space: 2 }) {
        // 文件名
        Text(ri.item.path.substring(ri.item.path.lastIndexOf("/")+1))
          .fontSize(16)
          .fontColor($r('app.color.text_secondary_color'))
          .fontWeight(FontWeight.Medium)
          .maxLines(1)
          .textOverflow({ overflow: TextOverflow.Ellipsis })
          .layoutWeight(1)
        // 文件路径
        Text(ri.item.path)
          .fontSize(12)
          .fontColor($r('app.color.text_secondary_color'))
          .maxLines(1)
          .textOverflow({ overflow: TextOverflow.Ellipsis })
          .layoutWeight(2)
      // }
      // .layoutWeight(1)

      // 文件大小和修改时间
      // Column({ space: 2 }) {
        Text(formatFileSize(ri.item))
          .fontSize(14)
          .fontColor($r('app.color.text_secondary_color'))
          .alignSelf(ItemAlign.End)
          .layoutWeight(1)

        Text(formatTimestamp(ri.item.metadata?.mtime))
          .fontSize(12)
          .fontColor($r('app.color.text_secondary_color'))
          .alignSelf(ItemAlign.End)

        Text(formatTimestamp(ri.item.metadata?.ctime))
          .fontSize(12)
          .fontColor($r('app.color.text_secondary_color'))
          .alignSelf(ItemAlign.End)
      // }
    }
    .padding({ top: 8, bottom: 8, left: 16, right: 16 })
    .width('100%')
    .height(60) // 固定高度以便Repeat组件计算
    // .backgroundColor(this.isSelected ? $r('app.color.selected_row_background_color') : $r('app.color.row_background_color'))
    .onClick(() => {
      // this.isSelected = !this.isSelected
      // TODO: 实现文件打开功能
      // this.openFile(this.item.path)
    })
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
          Repeat<NodeInfo>(this.currentState.searchResults)
            .each((ri) => {
              ListItem() {
                Column({ space: 2 }) {
                  this.ResultRow(ri)
                }
              }.border({ width: 1 })
            }).virtualScroll()
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

// 辅助函数：格式化文件大小
function formatFileSize(node: NodeInfo): string {
  const bytes = node.metadata?.size
  if (node.metadata?.type != NodeFileType.File) {
    return '-'
  }
  if (bytes === undefined) return '-'
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  if (bytes < 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  return `${(bytes / (1024 * 1024 * 1024)).toFixed(1)} GB`;
}

// 辅助函数：格式化时间戳
function formatTimestamp(timestamp: number | undefined): string {
  if (timestamp === undefined) return '-'
  const date = new Date(timestamp * 1000); // Rust后端使用秒级时间戳
  return date.toLocaleDateString() + ' ' + date.toLocaleTimeString();
}
