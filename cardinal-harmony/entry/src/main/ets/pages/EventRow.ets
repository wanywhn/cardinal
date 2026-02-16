
import { FileSystemEvent } from '../types/AppStateManager'

/**
 * 事件行组件
 */
@ComponentV2
export struct EventRow {
  @Param event: FileSystemEvent = {
    id: '',
    type: '',
    name: '',
    path: '',
    time: ''
  }

  build() {
    Row() {
      // 事件类型标签
      Text(this.event.type)
        .fontSize(12)
        .fontColor('#FFFFFF')
        .padding({ top: 4, bottom: 4, left: 8, right: 8 })
        .borderRadius(4)
        .backgroundColor(this.getEventTypeColor(this.event.type))
        .margin({ right: 10 })
      
      // 事件详情
      Column({ space: 2 }) {
        // 文件名
        Text(this.event.name)
          .fontSize(16)
          .fontWeight(FontWeight.Medium)
          .maxLines(1)
          .textOverflow({ overflow: TextOverflow.Ellipsis })
        
        // 文件路径
        Text(this.event.path)
          .fontSize(12)
          .fontColor($r('app.color.text_secondary_color'))
          .maxLines(1)
          .textOverflow({ overflow: TextOverflow.Ellipsis })
      }
      .layoutWeight(1)
      
      // 时间
      Text(this.event.time)
        .fontSize(14)
        .fontColor($r('app.color.text_secondary_color'))
        .alignSelf(ItemAlign.Center)
    }
    .padding({ top: 8, bottom: 8, left: 16, right: 16 })
    .width('100%')
    .height(60) // 固定高度以便Repeat组件计算
    .backgroundColor($r('app.color.row_background_color'))
  }

  /**
   * 根据事件类型获取颜色
   */
  private getEventTypeColor(eventType: string): ResourceColor {
    switch (eventType) {
      case 'CREATE':
        return $r('app.color.event_create_color')
      case 'MODIFY':
        return $r('app.color.event_modify_color')
      case 'DELETE':
        return $r('app.color.event_delete_color')
      case 'MOVE':
        return $r('app.color.event_move_color')
      case 'ACCESS':
        return $r('app.color.event_access_color')
      default:
        return $r('app.color.event_default_color')
    }
  }
}