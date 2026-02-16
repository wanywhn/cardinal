
import { SearchResult } from '../types/AppStateManager'

/**
 * 结果行组件
 */
@ComponentV2
export struct ResultRow {
  @Param item: SearchResult = {
    id: '',
    name: '',
    path: '',
    size: '',
    modified: ''
  }
  @Param caseSensitive: boolean = false
  @Local isSelected: boolean = false

  build() {
    Row() {
      // 图标占位
      Image($r('app.media.icon_file'))
        .width(24)
        .height(24)
        .margin({ right: 10 })
      
      // 文件信息列
      Column({ space: 2 }) {
        // 文件名
        Text(this.item.name)
          .fontSize(16)
          .fontWeight(FontWeight.Medium)
          .maxLines(1)
          .textOverflow({ overflow: TextOverflow.Ellipsis })
        
        // 文件路径
        Text(this.item.path)
          .fontSize(12)
          .fontColor($r('app.color.text_secondary_color'))
          .maxLines(1)
          .textOverflow({ overflow: TextOverflow.Ellipsis })
      }
      .layoutWeight(1)
      
      // 文件大小和修改时间
      Column({ space: 2 }) {
        Text(this.item.size)
          .fontSize(14)
          .alignSelf(ItemAlign.End)
        
        Text(this.item.modified)
          .fontSize(12)
          .fontColor($r('app.color.text_secondary_color'))
          .alignSelf(ItemAlign.End)
      }
    }
    .padding({ top: 8, bottom: 8, left: 16, right: 16 })
    .width('100%')
    .height(60) // 固定高度以便Repeat组件计算
    .backgroundColor(this.isSelected ? $r('app.color.selected_row_background_color') : $r('app.color.row_background_color'))
    .onClick(() => {
      this.isSelected = !this.isSelected
      // TODO: 实现文件打开功能
      this.openFile(this.item.path)
    })
  }

  /**
   * 打开文件
   */
  private openFile(filePath: string): void {
    // 打桩：实际的文件打开功能将通过FFI调用实现
    console.info(`Opening file: ${filePath}`)
  }
}