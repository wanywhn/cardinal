import { AppStateManager, AppState } from '../types/AppStateManager'
import { SearchBar } from './SearchBar'
import { FilesTab } from './FilesTab'
import { EventsTab } from './EventsTab'

import { abilityAccessCtrl, common, Permissions, bundleManager } from '@kit.AbilityKit';
import { BusinessError } from '@kit.BasicServicesKit';
import { appRecovery } from '@kit.AbilityKit';


const permissions: Permissions = 'ohos.permission.ACCESS_USER_FULL_DISK';

function reqPermissionsFromUser(permissions: Array<Permissions>, context: common.UIAbilityContext): boolean {
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
        atManager.requestPermissionOnSetting(context, permissions).then((data: Array<abilityAccessCtrl.GrantStatus>) => {
          console.info(`requestPermissionOnSetting success, result: ${data}`);
        }).catch((err: BusinessError) => {
          console.error(`requestPermissionOnSetting fail, code: ${err.code}, message: ${err.message}`);
          return false
        });
      }
    }
    // 授权成功
  }).catch((err: BusinessError) => {
    console.error(`Failed to request permissions from user, code: ${err.code}, message: ${err.message}`);
    return false;
  })
  return true
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

    // 检查权限状态
    this.checkPermissions()
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

  /**
   * 检查权限状态
   */
  private async checkPermissions(): Promise<void> {
    try {
      const context: common.UIAbilityContext = this.getUIContext().getHostContext() as common.UIAbilityContext;
      const atManager: abilityAccessCtrl.AtManager = abilityAccessCtrl.createAtManager();

      // 获取应用tokenId
      const bundleInfo = await bundleManager.getBundleInfoForSelf(
        bundleManager.BundleFlag.GET_BUNDLE_INFO_WITH_APPLICATION
      );
      const tokenId = bundleInfo.appInfo.accessTokenId;


      // 检查权限状态
      const checkAndHandlePermission = async () => {
        let grantStatus = await atManager.checkAccessToken(tokenId, permissions);
        if (grantStatus === abilityAccessCtrl.GrantStatus.PERMISSION_GRANTED) {
          console.info(`checkPermissions success, result: ${grantStatus}`);
          this.appState.init();
          return;
        }

        this.getUIContext().showAlertDialog({
          title: '权限缺失',
          message: "需要获取'允许访问全盘文件'以建立相关索引，将引导打开权限界面，请授予相关权限后关闭窗口。",
          autoCancel: true,
          alignment: DialogAlignment.Center,
          buttons: [
            {
              value: '知道了',
              action: () => {
                const context: common.UIAbilityContext =
                  this.getUIContext().getHostContext() as common.UIAbilityContext;
                atManager.openPermissionOnSetting(context, permissions)
                  .then((data: abilityAccessCtrl.SelectedResult) => {
                    console.info(`openPermissionOnSetting success, result: ${data}`);
                    // 设置完成后再次检查权限
                    checkAndHandlePermission();
                  })
                  .catch((err: BusinessError) => {
                    console.error(`openPermissionOnSetting fail, code: ${err.code}, message: ${err.message}`);
                  });
              }
            }
          ]
        });
      }


      // 初始检查
      await checkAndHandlePermission();

    } catch (error) {
      console.error(`Failed to check permissions: ${error}`);
    }
  }

  build() {
    Column() {
        this.MainInterface()
    }
    .height('80%')
    .backgroundColor($r('app.color.main_background_color'))
  }

  /**
   * 主功能界面构建器
   */
  @Builder
  MainInterface() {
    Column() {
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
      .scrollable(false)
      .cachedMaxCount(0, TabsCacheMode.CACHE_LATEST_SWITCHED)
      .onChange((index: number) => {
        this.appState.setActiveTab(index)
      })
      
      Row() {
        this.objInfoBar()
        // Blank()
        // 后端状态栏
        this.StatusBar()
      }
      .justifyContent(FlexAlign.SpaceBetween)
      .width('100%')
    }
  }

  @Builder
  objInfoBar() {
    Row() {
      Text(`${this.currentState.searchResults.length}个对象`)
    }
  }

  /**
   * 状态栏构建器
   */
  @Builder
  StatusBar() {
    Row() {
      Text(`后端状态: ${this.appState.getBackendStatusText()}`)
        .fontSize(12)

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
    .padding({ left: 16, right: 16, top: 8, bottom: 8 })
  }

  /**
   * 处理搜索请求
   */
  private handleSearch(query: string): void {
    console.info(`Searching for: ${query}`)
    this.appState.performSearch(query)
  }
}