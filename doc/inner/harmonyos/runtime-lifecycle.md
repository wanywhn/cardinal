# Runtime Lifecycle - HarmonyOS Edition

This document describes Cardinal's lifecycle states and lifecycle management specifically for the HarmonyOS implementation, focusing on the integration between ArkUI frontend and Rust backend through FFI.

---

## Architecture Overview

HarmonyOS Cardinal follows a client-server architecture where:

- **ArkUI Frontend**: Manages UI state, user interactions, and lifecycle events
- **Rust Backend**: Handles search indexing, filesystem monitoring, and core logic
- **FFI Bridge**: Mediates communication between ArkUI and Rust using ohrs toolkit

The lifecycle is inherently asynchronous due to the inter-process communication nature of FFI calls.

---

## Lifecycle States

### Application-Level States

1. **UNINITIALIZED**
   - **Description**: Application loaded but Rust backend not initialized
   - **UI Behavior**: Show loading screen or splash
   - **Backend State**: Rust components not loaded, memory structures empty

2. **INITIALIZING**
   - **Description**: Backend initialization in progress
   - **UI Behavior**: Show progress indicator, disable search functionality
   - **Backend State**:
     - Loading persistent cache (`SearchCache::try_read_persistent_cache`)
     - Starting filesystem walk if cache unavailable
     - Setting up HarmonyOS filesystem event monitoring

3. **INDEXING**
   - **Description**: Filesystem walk in progress
   - **UI Behavior**: Show indexing progress, partial search capability
   - **Backend State**: `fswalk` actively traversing filesystem, updating search cache

4. **READY**
   - **Description**: Ready state with filesystem monitoring active
   - **UI Behavior**: Full functionality enabled, search available
   - **Backend State**:
     - Filesystem event watcher running
     - Search cache fully populated
     - Background update loop active

5. **UPDATING**
   - **Description**: Background updates being applied
   - **UI Behavior**: Search available but results may be updating
   - **Backend State**: Applying filesystem changes to cache incrementally

6. **ERROR**
   - **Description**: Error state requiring user intervention
   - **UI Behavior**: Show error message, disable search, offer retry options
   - **Backend State**: Various error conditions (permission denied, storage full, etc.)

### HarmonyOS-Specific Lifecycle Events

- **AbilityStage.onCreate()**: Application instance creation
- **Ability.onWindowStageCreate()**: Window stage ready for UI rendering
- **Ability.onForeground()**: Application moved to foreground
- **Ability.onBackground()**: Application moved to background
- **Ability.onDestroy()**: Application destruction

---

## State Transitions

### Initialization Flow

```
UNINITIALIZED
    ↓ (App launch)
INITIALIZING
    ↓ (Backend loads persistent cache)
    if cache valid → READY
    if cache invalid → INDEXING
        ↓ (Filesystem walk complete)
        READY
```

### Runtime Flow

```
READY
    ↓ (Filesystem change detected)
UPDATING
    ↓ (Changes applied)
READY

READY
    ↓ (Rescan triggered by user/config change)
INDEXING
    ↓ (Rescan complete)
READY

READY/INDEXING/UPDATING
    ↓ (Error condition)
ERROR
    ↓ (User retry/restart)
INITIALIZING
```

### Background/Foreground Transitions

```
READY
    ↓ (App backgrounded)
BACKGROUND (Filesystem monitoring may pause)
    ↓ (App foregrounded, filesystem changes occurred)
UPDATING → READY
    ↓ (App foregrounded, no filesystem changes)
READY
```

---

## Implementation Details

### State Management Components

#### Rust Backend State (`harmony-bindings/src/state_mgr.rs`)

```rust
pub struct HarmonyAppState {
    pub lifecycle_state: AtomicU8,      // Current lifecycle state
    pub search_cache: Option<Arc<SearchCache>>,
    pub event_watcher: Option<EventWatcher>,
    pub background_task: Option<BackgroundTaskHandle>,
}

// State constants matching ArkUI expectations
pub const STATE_UNINITIALIZED: u8 = 0;
pub const STATE_INITIALIZING: u8 = 1;
pub const STATE_INDEXING: u8 = 2;
pub const STATE_READY: u8 = 3;
pub const STATE_UPDATING: u8 = 4;
pub const STATE_ERROR: u8 = 5;
```

#### ArkUI State Manager (`cardinal-harmony/entry/src/main/ets/services/StateManager.ets`)

```typescript
export class StateManager {
  private currentState: AppLifecycleState = AppLifecycleState.UNINITIALIZED;
  private stateListeners: Array<StateListener> = [];

  // HarmonyOS lifecycle integration
  onAbilityCreate(): void {
    this.transitionTo(AppLifecycleState.INITIALIZING);
    this.initializeBackend();
  }

  onAbilityForeground(): void {
    this.checkForUpdates();
  }

  onAbilityBackground(): void {
    this.pauseMonitoring(); // Optional - HarmonyOS may handle this
  }
}
```

### FFI Lifecycle Interface

#### Backend Initialization (`harmony-bindings/src/lifecycle.rs`)

```rust
#[napi]
pub async fn initialize_backend() -> napi::Result<u8> {
    // Set state to INITIALIZING
    update_lifecycle_state(STATE_INITIALIZING);

    // Initialize search cache
    match SearchCache::try_read_persistent_cache() {
        Some(cache) => {
            // Start filesystem monitoring
            start_event_watcher();
            update_lifecycle_state(STATE_READY);
            Ok(STATE_READY)
        }
        None => {
            // Start filesystem walk
            start_initial_indexing();
            update_lifecycle_state(STATE_INDEXING);
            Ok(STATE_INDEXING)
        }
    }
}
```

#### State Notification Bridge

```rust
#[napi]
pub fn subscribe_to_state_changes(callback: js_function) -> napi::Result<()> {
    // Register callback for state changes
    STATE_CALLBACKS.register(callback);
}

fn update_lifecycle_state(new_state: u8) {
    CURRENT_STATE.store(new_state);
    STATE_CALLBACKS.notify_all(new_state);
}
```

---

## HarmonyOS Integration Points

### Application Entry Points

- **EntryAbility.ets**: Main application entry, manages HarmonyOS lifecycle
- **StateManager.ets**: Coordinates between HarmonyOS events and internal state
- **CardinalService.ets**: FFI interface to Rust backend

### Resource Management

- **Memory**: Rust backend manages its own memory; ArkUI manages UI resources
- **Filesystem**: Shared access managed through HarmonyOS permissions
- **Background Tasks**: Use HarmonyOS TaskPool for long-running operations

### Error Handling Strategy

1. **Permissions Errors**: Guide user through HarmonyOS permission granting flow
2. **Storage Errors**: Check available space, cleanup options
3. **Network Errors**: (Future) For cloud sync features
4. **Backend Errors**: Graceful degradation or restart options

---

## State Persistence

### Cache Persistence

- **Location**: HarmonyOS application-specific storage
- **Format**: Memory-mapped slab files (`slab-mmap`)
- **Recovery**: Validate cache integrity on load, rebuild if corrupted

### UI State Persistence

- **Location**: HarmonyOS Preferences data store
- **Content**: Last search query, window state, user preferences
- **Scope**: Per-application instance

---

## Performance Considerations

### Memory Optimization

- **Lazy Loading**: Load icon cache only when needed
- **Cache Limits**: Implement size limits for HarmonyOS memory constraints
- **Background Throttling**: Reduce activity when app is backgrounded

### Battery Optimization

- **Filesystem Monitoring**: Use HarmonyOS efficient filesystem APIs
- **Update Batching**: Batch updates to reduce wake-ups
- **Background Restrictions**: Respect HarmonyOS background task limits

---

## Debugging and Monitoring

### Logging Strategy

-- **FFI Bridge**： Use `ohos-hilog-binding` (a binding crate for HarmonyOS's hilog) for harmony-bindings logging

- **Rust Backend**: Use `tracing` with HarmonyOS log integration
- **ArkUI Frontend**: Use `hilog` for HarmonyOS-compatible logging
- **Cross-platform**: Unified log format for easier debugging

### State Transition Tracing

```typescript
// Enable detailed lifecycle logging in debug builds
if (isDebugMode()) {
  stateManager.addStateListener((oldState, newState) => {
    hilog.info(
      0x0000,
      "Lifecycle",
      "Transition: %s → %s",
      AppLifecycleState[oldState],
      AppLifecycleState[newState],
    );
  });
}
```

### Testing Lifecycle Scenarios

1. **Cold Start**: App launch with no cached data
2. **Warm Start**: App launch with valid cache
3. **Background/Foreground**: State persistence across transitions
4. **Error Recovery**: Graceful handling of various error conditions
5. **Permission Changes**: Dynamic response to permission grants/revocations

---

## Compatibility Notes

### HarmonyOS Version Requirements

- **Minimum**: HarmonyOS 6.0 (API version 22)
- **Target**: HarmonyOS 6.0+ (API version 22+) for optimal features

This lifecycle design ensures smooth integration with HarmonyOS ecosystem while maintaining the robust search capabilities of the Cardinal backend.
