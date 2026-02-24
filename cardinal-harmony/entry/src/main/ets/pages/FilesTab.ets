import { AppStateManager, AppState } from '../types/AppStateManager'
import { NodeInfo, NodeFileType } from 'libharmony_bindings.so';
import fileUri from '@ohos.file.fileuri';
import uniformTypeDescriptor from '@ohos.data.uniformTypeDescriptor';
import filePreview from '@hms.filemanagement.filepreview';

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

      SymbolGlyph(getItemIcon(ri.item))
        .fontSize(24)
        .renderingStrategy(SymbolRenderingStrategy.SINGLE)
        .fontColor([Color.Black, Color.Green, Color.White])
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
          .layoutWeight(4)
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
          .layoutWeight(1)

        Text(formatTimestamp(ri.item.metadata?.ctime))
          .fontSize(12)
          .fontColor($r('app.color.text_secondary_color'))
          .alignSelf(ItemAlign.End)
          .layoutWeight(1)

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
    .bindMenu(
      [
      {
        value: '打开',
        action: () => {
          let uri = fileUri.getUriFromPath(ri.item.path)
          let uiContext = this.getUIContext().getHostContext() as Context;
          filePreview.canPreview(uiContext, uri).then((result) => {    // 传入支持的文件类型且文件存在时会返回true
            console.info(`Succeeded in obtaining the result of whether it can be previewed. result = ${result}`);
          }).catch((err: BusinessError) => {
            console.error(`Failed to preview file ${ri.item.path}`)
            console.error(`Failed to obtain the result of whether it can be previewed, err.code = ${err.code}, err.message = ${err.message}`);
          });
          let displayInfo: filePreview.DisplayInfo = {
            x: 100,
            y: 100,
            width: 800,
            height: 800
          };

          let typeId = uniformTypeDescriptor.getUniformDataTypeByFilenameExtension(ri.item.path.substring(ri.item.path.lastIndexOf('.')));
          // 根据UTD数据类型查询对应的MIMEType列表。
          let typeObj = uniformTypeDescriptor.getTypeDescriptor(typeId);
          let mimeTypes = typeObj.mimeTypes;

          let fileInfo: filePreview.PreviewInfo = {
            title: ri.item.path.substring(ri.item.path.lastIndexOf('/')+1),
            uri: uri,
            mimeType: mimeTypes[0]
          };
          filePreview.openPreview(uiContext, fileInfo, displayInfo).then(() => {
            console.info('Succeeded in opening preview');
          }).catch((err: BusinessError) => {
            console.error(`Failed to open preview, err.code = ${err.code}, err.message = ${err.message}`);
          });
        }
      },
      {
        value: '打开文件所在文件夹',
        action: () => {
        }
      },
    ]
    )
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

function getItemIcon(item: NodeInfo): Resource {
  switch (item.metadata?.type) {
    case NodeFileType.File:
      return $r('sys.symbol.doc')
    case NodeFileType.Dir:
      return $r('sys.symbol.folder')
    case NodeFileType.Symlink:
      return $r('sys.symbol.link_slash')
    default:
      return $r('sys.symbol.questionmark_circle')
  }
}