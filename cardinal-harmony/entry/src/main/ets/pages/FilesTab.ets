import { AppStateManager, AppState } from '../types/AppStateManager'
import { NodeInfo, NodeFileType } from 'libharmony_bindings.so';
import fileUri from '@ohos.file.fileuri';
import uniformTypeDescriptor from '@ohos.data.uniformTypeDescriptor';
import { filePreview } from '@kit.PreviewKit';


/**
 * 文件标签页组件
 */
@ComponentV2
export struct FilesTab {
  private appState: AppStateManager = AppStateManager.getInstance()
  currentState: AppState = this.appState.getState()
  @Local colNameWidth2: Length = '25%'
  @Local colPathWidth2: Length = '50%'
  @Local colSizeWidth2: Length = '10%'
  @Local colMtimeWidth2: Length = '15%'

  aboutToAppear(): void {
    // 初始化状态
    this.currentState = this.appState.getState()
  }

  @Builder
  RightClickMenuBuilder(ri: RepeatItem<NodeInfo>) {
    Menu() {
      MenuItem({ content: '打开文件', labelInfo: '' })
        .onClick(() => {
          this.openFile(ri.item.path, ri.item);
        })
      MenuItem({ content: '打开所在文件夹', labelInfo: '' })
        .onClick(() => {
          this.openFileFolder(ri.item.path);
        })
    }
    .width(180)
    .height(80)
  }

  @Builder
  ResultRow(ri:RepeatItem<NodeInfo>) {
    Row({space: 1}) {
      Row() {
        SymbolGlyph(getUTDIcon(ri.item))
          .fontSize(24)
          .renderingStrategy(SymbolRenderingStrategy.SINGLE)
          .fontColor([Color.Black, Color.Green, Color.White])
          .margin({right: 10})

        Text(ri.item.path.substring(ri.item.path.lastIndexOf("/")+1))
          .fontSize(16)
          .fontColor($r('app.color.text_secondary_color'))
          .fontWeight(FontWeight.Medium)
          .maxLines(1)
          .textOverflow({ overflow: TextOverflow.Ellipsis })
      }
      .width(this.colNameWidth2)
      // .layoutWeight(1)

        // 文件路径
        Text(ri.item.path)
          .fontSize(12)
          .fontColor($r('app.color.text_secondary_color'))
          .maxLines(1)
          .textOverflow({ overflow: TextOverflow.Ellipsis })
          // .layoutWeight(4)
          .width(this.colPathWidth2)
      // }
      // .layoutWeight(1)

      // 文件大小和修改时间
      // Column({ space: 2 }) {
        Text(formatFileSize(ri.item))
          .fontSize(14)
          .fontColor($r('app.color.text_secondary_color'))
          .alignSelf(ItemAlign.End)
          // .layoutWeight(1)
          .width(this.colSizeWidth2)

        Text(formatTimestamp(ri.item.metadata?.mtime))
          .fontSize(12)
          .fontColor($r('app.color.text_secondary_color'))
          .alignSelf(ItemAlign.End)
          // .layoutWeight(1)
          .width(this.colMtimeWidth2)
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
    .bindContextMenu(this.RightClickMenuBuilder(ri), ResponseType.RightClick)
    .bindMenu(
      [
      {
        value: '打开',
        action: () => {
          this.openFile(ri.item.path, ri.item);
        }
      },
      {
        value: '打开文件所在文件夹',
        action: () => {
          this.openFileFolder(ri.item.path);
        }
      },
    ]
    )
  }

  build() {
    Column() {
      if (this.currentState.searchResults.length === 0) {
        Text("No results found")
          .fontSize(16)
          .margin({ top: 20 })
          .textAlign(TextAlign.Center)
          .width('100%')
      } else {
        Column() {
            Row() {
                Text('名称')
                .width(this.colNameWidth2)
                // .layoutWeight(1)
                .height(40)
                .backgroundColor('#f0f0f0')
                .textAlign(TextAlign.Center)
                .fontSize(16)
                .fontWeight(FontWeight.Bold)
                .border({ width: 1, color: '#ccc' })

              Text('路径')
                .width(this.colPathWidth2)
                // .layoutWeight(3)
                .height(40)
                .backgroundColor('#f0f0f0')
                .textAlign(TextAlign.Center)
                .fontSize(16)
                .fontWeight(FontWeight.Bold)
                .border({ width: 1, color: '#ccc' })

              Text('大小')
                .width(this.colSizeWidth2)
                // .layoutWeight(1)
                .height(40)
                .backgroundColor('#f0f0f0')
                .textAlign(TextAlign.Center)
                .fontSize(16)
                .fontWeight(FontWeight.Bold)
                .border({ width: 1, color: '#ccc' })

              Text('修改时间')
                .width(this.colMtimeWidth2)
                // .layoutWeight(1)
                .height(40)
                .backgroundColor('#f0f0f0')
                .textAlign(TextAlign.Center)
                .fontSize(16)
                .fontWeight(FontWeight.Bold)
                .border({ width: 1, color: '#ccc' })

            }
            .width('100%')
            .height(40)

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
          .height('100%')
        }.width('100%')
      }
    }
    .layoutWeight(1)
    .width('100%')
    .padding({ top: 10 })
  }
  
  /**
   * 打开文件
   */
  private openFile(filePath: string, nodeInfo?: NodeInfo): void {
    let uri = fileUri.getUriFromPath(filePath);
    let uiContext = this.getUIContext().getHostContext() as Context;
    
    filePreview.canPreview(uiContext, uri).then((result) => {
      console.info(`Succeeded in obtaining the result of whether it can be previewed. result = ${result}`);
    }).catch((err: BusinessError) => {
      console.error(`Failed to preview file ${filePath}`);
      console.error(`Failed to obtain the result of whether it can be previewed, err.code = ${err.code}, err.message = ${err.message}`);
    });
    
    let displayInfo: filePreview.DisplayInfo = {
      x: 100,
      y: 100,
      width: 800,
      height: 800
    };

    let fileInfo: filePreview.PreviewInfo = {
      uri: uri,
      mimeType: ""
    };
    
    filePreview.openPreview(uiContext, fileInfo, displayInfo).then(() => {
      console.info('Succeeded in opening preview');
    }).catch((err: BusinessError) => {
      console.error(`Failed to open preview, err.code = ${err.code}, err.message = ${err.message}`);
    });
  }

  /**
   * 打开文件所在文件夹
   */
  private openFileFolder(filePath: string): void {
    // TODO: 实现打开文件所在文件夹功能
    console.info(`Opening folder containing: ${filePath}`);
    // 这里需要实现打开文件夹的逻辑
    // 可能需要使用文件管理器或系统API来打开文件夹
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

// 辅助函数：根据文件扩展名获取图标资源
function getIconByExtension(extension: string): Resource {
  // 使用简单的switch语句来避免ArkTS的类型限制
  switch (extension) {
    // 文档文件
    case '.pdf':
    case '.doc':
    case '.docx':
    case '.txt':
    case '.rtf':
      return $r('sys.symbol.doc')
    
    // 表格文件
    case '.xls':
    case '.xlsx':
    case '.csv':
      return $r('sys.symbol.doc') // 使用文档图标作为表格文件图标
    
    // 演示文稿
    case '.ppt':
    case '.pptx':
      return $r('sys.symbol.doc') // 使用文档图标作为演示文稿图标
    
    // 图片文件
    case '.jpg':
    case '.jpeg':
    case '.png':
    case '.gif':
    case '.bmp':
    case '.svg':
      return $r('sys.symbol.rectangle_stack') // 使用堆叠矩形作为图片图标
    
    // 音频文件
    case '.mp3':
    case '.wav':
    case '.flac':
    case '.aac':
      return $r('sys.symbol.waveform') // 使用波形图标作为音频图标
    
    // 视频文件
    case '.mp4':
    case '.avi':
    case '.mkv':
    case '.mov':
      return $r('sys.symbol.play') // 使用播放图标作为视频图标
    
    // 压缩文件
    case '.zip':
    case '.rar':
    case '.7z':
    case '.tar':
    case '.gz':
      return $r('sys.symbol.archivebox')
    
    // 代码文件
    case '.js':
    case '.ts':
    case '.py':
    case '.java':
    case '.cpp':
    case '.c':
    case '.html':
    case '.css':
    case '.json':
    case '.xml':
      return $r('sys.symbol.doc')
    
    // 可执行文件
    case '.exe':
    case '.app':
    case '.bin':
      return $r('sys.symbol.gearshape')
    
    default:
      return $r('sys.symbol.doc')
  }
}

// 辅助函数：根据文件路径获取UTD图标
function getUTDIcon(item: NodeInfo): Resource {
  // 如果是文件夹，使用文件夹图标
  if (item.metadata?.type === NodeFileType.Dir) {
    return $r('sys.symbol.folder')
  }
  
  // 如果是符号链接，使用链接图标
  if (item.metadata?.type === NodeFileType.Symlink) {
    return $r('sys.symbol.link_slash')
  }
  
  // 对于文件，根据文件扩展名获取UTD类型并映射到图标
  let filePath = item.path
  let extension = filePath.substring(filePath.lastIndexOf('.')).toLowerCase()
  
  if (extension === '') {
    return $r('sys.symbol.questionmark_circle')
  }
  
  try {
    // 获取文件扩展名对应的UTD类型ID
    let typeId = uniformTypeDescriptor.getUniformDataTypeByFilenameExtension(extension)
    
    // 获取类型描述符
    let typeObj = uniformTypeDescriptor.getTypeDescriptor(typeId)

    // 根据UTD类型信息选择合适的图标
    // 这里可以根据typeObj的mimeTypes或description来进一步细化图标选择
    if (typeObj.mimeTypes && typeObj.mimeTypes.length > 0) {
      let mimeType = typeObj.mimeTypes[0]
      
      // 根据MIME类型大类选择图标
      if (mimeType.startsWith('image/')) {
        return $r('sys.symbol.rectangle_stack') // 图片文件使用堆叠矩形图标
      } else if (mimeType.startsWith('audio/')) {
        return $r('sys.symbol.waveform') // 音频文件使用波形图标
      } else if (mimeType.startsWith('video/')) {
        return $r('sys.symbol.play') // 视频文件使用播放图标
      } else if (mimeType.startsWith('text/')) {
        return $r('sys.symbol.doc') // 文本文件使用文档图标
      } else if (mimeType.includes('pdf')) {
        return $r('sys.symbol.doc') // PDF文件使用文档图标
      }
    }
    
    // 使用扩展名映射作为备用方案
    return getIconByExtension(extension)
    
  } catch (error) {
    console.error(`Failed to get UTD icon for file ${filePath}: ${error}`)
    // 使用扩展名映射作为回退方案
    return getIconByExtension(extension)
  }
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