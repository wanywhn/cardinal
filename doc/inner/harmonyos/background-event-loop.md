# Background Event Loop - HarmonyOS Edition

This chapter explains how the background processing is coordinated for HarmonyOS Cardinal, including search execution, metadata expansion, filesystem monitoring, and icon handling.

---

## Architecture Overview

HarmonyOS Cardinal uses a modified background event loop optimized for the mobile/embedded environment:

- **Single-process model**: Unlike desktop version's foreground/background threads, HarmonyOS runs in a single process with ArkUI and Rust sharing memory via FFI
- **Task-based architecture**: Uses HarmonyOS TaskPool for background operations instead of long-lived background threads
- **Event-driven design**: Leverages HarmonyOS system events for lifecycle and filesystem changes

---

## Communication Architecture

### FFI-based Communication Pattern

```
ArkUI Frontend
    ↓ (FFI calls via ohos-rs)
Rust Backend Service Layer
    ↓ (Internal task spawning)
Background TaskPool Workers
    ↓ (Results via callbacks)
Frontend Event Handlers
```

### Command Channels (Adapted for HarmonyOS)

```
[search_command]        ──▶ Search request (query + options + callback ID)
[node_info_command]     ──▶ Slab indices needing path/metadata expansion
[icon_prefetch_command] ──▶ Visible slab indices for icon prefetching
[rescan_command]        ──▶ Manual rescan requests
[config_update_command] ──▶ Watch configuration updates
[file_op_command]       ──▶ File operations (open, reveal, etc.)
[lifecycle_command]     ──▶ Lifecycle state changes
```

Unlike desktop version using crossbeam channels, HarmonyOS uses:

- **Direct FFI calls**: Synchronous calls for simple operations
- **Async tasks with callbacks**: For long-running operations via HarmonyOS TaskPool
- **Event emission**: Results delivered through registered callbacks

---

## Main Processing Loop

### Event Dispatching Architecture

```text
┌─────────────────────────────────────────────┐
│             HarmonyOS App Task              │
│  ┌─────────────────────────────────────┐   │
│  │    Main Event Dispatcher            │   │
│  │  • Receives FFI calls from ArkTS    │   │
│  │  • Dispatches to appropriate handler│   │
│  └──────────────┬──────────────────────┘   │
│                 │                          │
│  ┌──────────────▼──────────────────────┐   │
│  │      TaskPool Workers               │   │
│  │    ┌────┐ ┌────┐ ┌────┐            │   │
│  │    │CPU1│ │CPU2│ │CPU3│            │   │
│  │    └────┘ └────┘ └────┘            │   │
│  └────────────────────────────────────┘   │
│                 │                          │
│  ┌──────────────▼──────────────────────┐   │
│  │      Callback Registry               │   │
│  │  • Maps callback IDs to ArkTS funcs  │   │
│  │  • Thread-safe result delivery       │   │
│  └────────────────────────────────────┘   │
└─────────────────────────────────────────────┘
```

### Key Differences from Desktop

| Aspect            | Desktop (macOS/Linux)        | HarmonyOS               |
| ----------------- | ---------------------------- | ----------------------- |
| **Thread Model**  | Dedicated background thread  | TaskPool worker threads |
| **Communication** | Crossbeam channels           | FFI calls + callbacks   |
| **Concurrency**   | Single event loop            | Parallel task execution |
| **Resource Mgmt** | Manual thread control        | HarmonyOS-managed tasks |
| **Lifecycle**     | Independent thread lifecycle | Tied to app lifecycle   |

---

## Filesystem Monitoring on HarmonyOS

### HarmonyOS Filesystem API Integration

- **EventWatcher Adaptation**: `cardinal-sdk` extended with HarmonyOS-specific filesystem monitoring
- **Polling Strategy**: Limited background polling on HarmonyOS (reduced frequency for battery optimization)
- **Change Batches**: Events grouped and delivered in batches for efficiency

### Filesystem Event Flow

```
HarmonyOS Filesystem API
    ↓ (File change events)
HarmonyOSEventWatcher
    ↓ (Process and filter)
EventBuffer (by path, timestamp)
    ↓ (Batch processing)
SearchCache::handle_fs_events()
    ↓ (Update index)
    if successful → UI update via callback
    if error → trigger rescan
```

### Adaptive Monitoring

- **Foreground Mode**: Active monitoring with normal frequency
- **Background Mode**: Reduced monitoring or paused (depending on HarmonyOS policy)
- **Battery Saver**: Further reduced frequency or disabled

---

## Search Processing Pipeline

### Search Request Flow

```
1. ArkTS UI → search() FFI call
2. FFI layer → creates search job with callback ID
3. TaskPool dispatches to worker thread
4. Worker executes: search_cache.search_with_options(query, options)
5. Results marshaled to FFI-compatible format
6. Callback invoked with results via registered callback
```

### Performance Optimizations

- **Query Caching**: Cache recent search results (limited memory)
- **Incremental Results**: Deliver partial results as they become available
- **Priority Queue**: Search tasks prioritized over metadata/icon tasks

---

## Metadata and Icon Pipeline

### Node Information Expansion

- **Lazy Loading**: Metadata loaded on-demand when UI needs it
- **Batch Processing**: Multiple slab indices expanded in single FFI call
- **Caching**: Frequently accessed node info cached in memory

### Icon Handling (HarmonyOS Adaptation)

- **Current Status**: Placeholder/stub implementation (full icon system TBD)
- **Future Integration**:
  - HarmonyOS system icon providers
  - File-type detection via MIME
  - Custom icon extraction for specific file types
- **Placeholder Strategy**: Generic icons based on file type/category

### Icon Prefetch Strategy

```
Visible Viewport Indices
    ↓ (via icon_prefetch_command)
Priority Queue Manager
    ↓ (prioritizes visible items)
Icon Generation Workers
    ↓ (uses HarmonyOS icon APIs)
Icon Cache (memory + disk)
    ↓ (delivered to UI)
UI icon updates via callback
```

---

## Rescan and Configuration Updates

### Rescan Flow (Adapted)

```
perform_rescan_harmonyos:
  1. Set lifecycle state → INDEXING
  2. Emit progress callback (0%, "Starting rescan...")
  3. Spawn TaskPool worker for filesystem walk
  4. Periodic progress updates (files/dirs counted)
  5. Cancel checkpoints (respects user cancellation)
  6. On completion: Rebuild search cache
  7. Restart filesystem monitoring
  8. Set lifecycle state → READY
  9. Emit completion callback
```

### Watch Configuration Updates

- **Dynamic Updates**: Configuration can change without restart
- **Validation**: Path validation using HarmonyOS filesystem APIs
- **Seamless Transition**: Old cache retained during rebuild, swapped on success

---

## Lifecycle Integration

### HarmonyOS App State Coordination

```
Ability.onCreate
    ↓ Initialize Rust backend (lightweight)
Ability.onWindowStageCreate
    ↓ Full backend initialization (if not already done)
Ability.onForeground
    ↓ Resume filesystem monitoring, refresh UI state
Ability.onBackground
    ↓ Pause intensive operations, persist cache if needed
Ability.onDestroy
    ↓ Clean shutdown, persist final cache state
```

### Background Task Management

- **Task Priorities**: Search > UI updates > Prefetch > Maintenance
- **Resource Awareness**: Scale back during low battery/thermal throttling
- **Graceful Degradation**: Reduce functionality rather than crash

---

## Error Handling and Recovery

### Error Categories

1. **Permission Errors**: Handle HarmonyOS permission denials gracefully
2. **Storage Errors**: Disk full, corrupt cache files
3. **Filesystem Errors**: Unavailable paths, mount changes
4. **Resource Errors**: Memory pressure, task limits

### Recovery Strategies

- **Automatic Retry**: With exponential backoff for transient errors
- **User Notification**: Clear error messages with recovery options
- **Fallback Modes**: Reduced functionality instead of complete failure
- **State Preservation**: Save partial work, resume when possible

---

## Performance Considerations

### Memory Management

- **Cache Limits**: Configurable limits based on device capabilities
- **LRU Eviction**: Least recently used data removed first
- **Memory Pressure Response**: Proactive cache reduction during memory warnings

### Battery Optimization

- **Aggressive Batching**: Group operations to minimize wake-ups
- **Intelligent Polling**: Adaptive filesystem monitoring frequency
- **Background Restrictions**: Respect HarmonyOS background task policies

### Cold Start Optimization

- **Lazy Initialization**: Defer non-essential setup
- **Progressive Loading**: Load search cache in background while UI is responsive
- **Placeholder Content**: Show UI immediately, populate with data as available

---

## Testing and Debugging

### Testing Strategy

- **Unit Tests**: Core logic tested independently of HarmonyOS
- **Integration Tests**: FFI boundary testing with mock callbacks
- **Performance Tests**: Measure memory, battery, and responsiveness
- **Edge Cases**: Permission changes, storage full, network interruptions

### Debugging Tools

- **Unified Logging**: `tracing` in Rust, `hilog` in ArkTS, cross-referenced
- **Performance Profiling**: HarmonyOS performance analysis tools
- **Memory Debugging**: HarmonyOS memory leak detection
- **State Inspection**: Dump current state for post-mortem analysis

### Monitoring Points

1. **Task Queue Length**: Monitor backlog of pending operations
2. **Memory Usage**: Track cache size and overall memory footprint
3. **Battery Impact**: Measure background activity battery consumption
4. **Response Times**: Search latency, UI update delays

---

## Future Optimizations

### Planned Enhancements

1. **Predictive Prefetch**: Anticipate user actions based on usage patterns
2. **Intelligent Caching**: Adaptive cache size based on usage frequency
3. **Cloud Integration**: (Future) Sync and search across devices
4. **ML-powered Ranking**: Improve result relevance with on-device ML

### Platform Evolution

- **New HarmonyOS APIs**: Leverage improved filesystem and background APIs
- **Hardware Acceleration**: Use device-specific capabilities where available
- **Cross-device Features**: Integration with HarmonyOS ecosystem

This background event loop design ensures Cardinal delivers responsive search capabilities on HarmonyOS while respecting platform constraints and optimizing for mobile/embedded device characteristics.
