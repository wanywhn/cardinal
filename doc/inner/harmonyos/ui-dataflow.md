# UI Dataflow - HarmonyOS Edition

This chapter maps the ArkUI-side components and lifecycle events to the Rust FFI layer for HarmonyOS Cardinal.

---

## Search Execution Flow

```
Search input change
  -> debounce (unless immediate) -> rust_search(query, ?, ?)
  -> await Promise<SearchResultsPayload>
  -> store results in @Local/@Track and pass to <List>/<Grid>
```

- ArkTS uses async/await instead of manual cancellation tokens
- Backend automatically handles cancellation through Promise rejection
- HarmonyOS has stricter battery and performance constraints on search operations
- Debouncing logic must respect HarmonyOS UI thread constraints

ArkTS Implementation Pattern:

```typescript
private handleSearchInput(query: string): void {
    // Built-in HarmonyOS debouncing through TaskPool
    this.taskPool.debounce(async () => {
        const results = await this.nativeService.search(query);
        this.searchResults = results;
    });

    // Immediate search bypasses debounce (Enter key)
    if (immediate) {
        this.taskPool.immediate(async () => {
            // Direct search execution
        });
    }
}
```

---

## Row Hydration Pipeline Using Repeat Component

```
@EntryV2
@ComponentV2
struct FileList {
  @Local items: Array<NodeInfo> = []

  build() {
    Column() {
      // 使用Repeat组件渲染大数据列表
      Repeat(this.items, (item: NodeInfo, index?: number) => {
        FileRow({
          path: item.path,
          metadata: item.metadata,
          icon: item.icon
        })
      })
      .onAppearIndex((start: number, end: number) => {
        // 可视区域变化时触发数据加载
        this.loadRange(start, end)
      })
    }
  }

  loadRange(start: number, end: number) {
    // 调用Rust FFI获取节点信息
    rust_get_nodes_info(this.items.slice(start, end))
      .then((nodes) => {
        // 更新对应位置的数据
        this.items.splice(start, nodes.length, ...nodes)
      })
  }
}
```

### Repeat组件核心优势：

1. **高性能渲染**：自动复用列表项组件，减少内存占用
2. **可视区域优化**：只渲染可视区域内的元素，提升滚动性能
3. **动态加载**：通过onAppearIndex回调实现按需加载
4. **平滑滚动**：内置动画优化，保证列表滚动流畅性

### 性能优化实践：

1. **组件复用**：确保FileRow组件使用@Reusable装饰器
2. **状态分离**：将频繁变化的状态与静态内容分离
3. **图片懒加载**：结合LazyForEach实现图片按需加载
4. **内存管理**：使用@Observed和@ObjectLink优化大数据量场景

---

## Icon Loading Pipeline

```
<List> useIconViewport(start,end)
  -> rust_update_icon_viewport(id, viewport: slab indices)
  -> backend loads icons for visible paths
  -> FFI stream emits [{ slabIndex, icon }]
  -> arkui_list listens to icon_update events
  -> maps slabIndex to row index; patches cache with override icon
```

HarmonyOS Specific Constraints:

- Icon loading must be batched and prioritized by viewport visibility
- Use HarmonyOS File API for secure file access
- Implement proper memory management for bitmap icons
- Consider HarmonyOS performance optimization guidelines

---

## HarmonyOS Quick Launch Integration

```
Global shortcut (Power button + Volume Down combo)
  -> rust_toggle_main_window()
  -> backend activates window and emits launch_trigger
  -> ArkUI focuses search input upon quick_launch event
```

Key Differences from Desktop:

- No global shortcuts due to HarmonyOS security policy restrictions
- Use HarmonyOS Window Manager APIs for window management
- Focus management handled through ArkUI lifecycle events
- Permission-based access controls for system integration

---

## HarmonyOS Specific Dataflow Considerations

### Platform Constraints

1. **Security Model**: All file accesses must use HarmonyOS File API
2. **Memory Constraints**: Implement progressive loading and efficient caching
3. **Performance Considerations**: Optimize IPC payload sizes for mobile devices

### ArkUI Lifecycle Integration

```typescript
@ComponentV2
struct FileSearchComponent {
    private dataLoader: DataLoader = new DataLoader();

    // V2生命周期方法
    @Local
    private onShow: () => void = () => {...};

    @Local
    private onHide: () => void = () => {};

    @Local
    private onBackPressInternal: () => boolean = () => {...};
```

### ArkUI Component Communication

```
SearchBar (ArkUI)
  ├─ handleSearchQuery(query: string)
  │     -> TaskPool.debounce -> rust_search(query, options)
  │           -> returns Promise<SearchResponse>
  └─ Repeat component renders search results
        ↓
  useDataLoader.ensureRangeLoaded(range)
        -> rust_get_nodes_info(slice: SlabIndex[])
        -> caches rows with optimistic updates
```

---

## Adding New Flows for HarmonyOS

### Development Process Guidelines

1. **FFI Command Definition**

   ```
   Step 1: Define Rust async functions with #[napi] annotation
   Step 2: ohos-rs automatically generates TypeScript bindings
   Step 3: Import and use generated Promise interfaces in ArkTS
   ```

2. **State Management Strategy**

   ```
   ArkUI @Local/@Track variables (V2)
   -> @BuilderParam for UI composition
   -> ohos-rs Promise-based API calls
   -> Updates UI through ArkUI V2 reactive framework
   ```

3. **Performance Optimization Guidelines**
   - **Memory Management**: Implement row recycling and object pooling
   - **Battery Optimization**: Use HarmonyOS TaskPool for background operations
   - **API Constraints**: All FFI operations must respect HarmonyOS security policies

### File Operations Example

```typescript
// HarmonyOS file operation patterns - State Management V2
@ComponentV2
struct HarmonyFileSystem {
  @Local openFile: (path: string) => Promise<void> = async (path: string): Promise<void> => {
    // Use HarmonyOS File API…
    const fileAccess = await HarmoryFile.open(path);
  }

  @Local saveFile: (content: Uint8Array) => Promise<string> = async (content: Uint8Array): Promise<string> => {
    // Must adhere to HarmonyOS security policies…
    const savedPath = await HarmoryFile.save(content);
    return savedPath;
  }
}
```

---

## HarmonyOS Architecture Integration

### Search Query Pattern

```
ArkUI Search Component (@ComponentV2)
  ├─ Decorators for data binding (@Local/@Track)…
  ├─ State management with @Local variables (V2)…
  ├─ Event handling through @BuilderParam composition…
  ├─ FFI communication through ohos-rs Task layer…
  └─ Performance optimized for mobile devices…
```

### Component Architecture Overview

```
ArkUI Interface Layer
  ├─ Views: List, Grid, Stack navigation…
  ├─ State: @Local/@Track reactive variables (V2)…
  ├─ Events: User interactions, lifecycle events…
  └─ Services: FFI communication handlers…
```

### Key Technical Considerations

1. **Build System Integration**

   ```
   root/Cargo.toml
   ├─ harmony feature flag enables ArkUI compilation
   ├─ ohos-rs generates HarmonyOS specific binaries
   └─ DevEco Studio handles ArkUI bundle assembly…
   ```

2. **Testing Strategy**
   ```
   // Unit tests…
   ├─ ArkUI component tests with HarmonyOS testing framework
   ├─ Rust backend tests through ohos-rs simulation layer…
   └─ Manual testing on HarmonyOS emulator/mobile devices…
   ```

### Performance Monitoring Framework

```
Monitoring:
  ├─ Memory usage patterns per device type…
  ├─ Battery consumption during search operations…
  ├─ Performance benchmarks for open/close operations…
  └─ User experience metrics for mobile optimization…
```

---

## Summary

The HarmonyOS UI Dataflow design preserves the core dataflow architecture while adapting to the mobile platform's specific constraints:

- **Asynchronous Communication**: Replaces IPC commands with Promise-based FFI
- **Platform Adaptation**: Integrates HarmonyOS lifecycle and security requirements
- **Performance Guidelines**: Built around mobile-specific optimization principles
- **Development Workflow**: Maintains familiar patterns while respecting platform constraints

This design provides a solid foundation for building Cardinal's search UI on HarmonyOS, balancing developer familiarity with platform-specific best practices.
