
import { AppStateManager, AppState } from '../types/AppStateManager'
import { SearchBar } from './SearchBar'
import { FilesTab } from './FilesTab'
import { EventsTab } from './EventsTab'

import { abilityAccessCtrl, common, Permissions } from '@kit.AbilityKit';
import { BusinessError } from '@kit.BasicServicesKit';
const permissions: Permissions[] = ['ohos.permission.LOCATION', 'ohos.permission.READ_WRITE_DOCUMENTS_DIRECTORY'];

function reqPermissionsFromUser(permissions: Array<Permissions>, context: common.UIAbilityContext): void {
  let atManager: abilityAccessCtrl.AtManager = abilityAccessCtrl.createAtManager();
  // requestPermissionsFromUser会判断权限的授权状态来决定是否唤起弹窗
  atManager.requestPermissionsFromUser(context, permissions).then((data) => {
    let grantStatus: number[] = data.authResults;
    let length: number = grantStatus.length;
    for (let i = 0; i < length; i++) {
      if (grantStatus[i] === 0) {
        // 用户授权，可以继续访问目标操作
        console.info(`${permissions[i]} is granted by user.`);
      } else {
        // 用户拒绝授权，提示用户必须授权才能访问当前页面的功能，并引导用户到系统设置中打开相应的权限
        return;
      }
    }
    // 授权成功
  }).catch((err: BusinessError) => {
    console.error(`Failed to request permissions from user, code: ${err.code}, message: ${err.message}`);
  })
}

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

    const context: common.UIAbilityContext = this.getUIContext().getHostContext() as common.UIAbilityContext;
    reqPermissionsFromUser(permissions, context);
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