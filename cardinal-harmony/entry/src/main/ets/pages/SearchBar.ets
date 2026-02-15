
/**
 * 搜索栏组件
 */
@ComponentV2
export struct SearchBar {
  @Param @Require queryChangeCallback: (query: string) => void
  @Param @Require toggleCaseSensitiveCallback: (value: boolean) => void
  private searchQuery: string = ""
  private caseSensitive: boolean = false
  private placeholder: string = "Search files..."

  // 初始化时设置回调函数
  aboutToAppear(): void {
    // 如果需要，可以在这里初始化一些内容
  }

  build() {
    Flex({ direction: FlexDirection.Column, alignItems: ItemAlign.Start, justifyContent: FlexAlign.Start }) {
      Row({ space: 5 }) {
        // 搜索输入框
        TextInput({
          placeholder: this.placeholder,
          text: this.searchQuery
        })
        .placeholderColor($r('app.color.text_placeholder_color'))
        .fontColor($r('app.color.text_primary_color'))
        .fontSize(16)
        .height(40)
        .width('80%')
        .borderRadius(8)
        .backgroundColor($r('app.color.input_background_color'))
        .onChange((value: string) => {
          this.searchQuery = value
          this.queryChangeCallback(value)
        })
        // 大小写敏感选项
        Toggle({
          type: ToggleType.Switch,
          isOn: this.caseSensitive
        })
        .size({ width: 45, height: 24 })
        .onChange((value: boolean) => {
          this.caseSensitive = value
          this.toggleCaseSensitiveCallback(value)
        })

        Text("Aa")
        .fontSize(12)
        .margin({ left: 5 })
      }
      .width('100%')
      .padding({ top: 10, bottom: 10, left: 16, right: 16 })
    }
    .width('100%')
    .backgroundColor($r('app.color.top_bar_background_color'))
  }
}