# IPC Commands - HarmonyOS Edition

This document describes the FFI interface between ArkUI frontend and Rust backend for HarmonyOS Cardinal, adapted from the desktop IPC commands.

---

## Architecture Overview

Key differences from desktop version:

- **Communication Protocol**: Uses ohos-rs FFI instead of Tauri commands
- **Async Model**: Promise-based async operations using ohos-rs Task
- **Type Safety**: Automatic TypeScript binding generation
- **Lifecycle Integration**: Tight coupling with HarmonyOS Ability lifecycle

---

## Command Categories

## Task-based Architecture

ohos-rs supports modern async programming using Task/Promise pattern similar to napi-rs v2:

### Rust Async Function Example

```rust
use napi_ohos::bindgen_prelude::*;
use tokio::task;

#[napi]
pub async fn search(
    query: String,
    options: SearchOptionsPayload
) -> Result<SearchResults> {
    task::spawn_blocking(move || {
        // CPU-intensive search operation
        perform_search(&query, &options)
    }).await.map_err(|e| Error::new(
        Status::GenericFailure,
        format!("Search failed: {}", e)
    ))
}
```

### TypeScript Interface Generation

ohos-rs automatically generates TypeScript bindings:

```typescript
// Auto-generated from Rust code
export function search(
  query: string,
  options: SearchOptionsPayload,
): Promise<SearchResults>;
```

---

## Command Categories

### 1. Search and Data Commands

| Command              | Parameters                                             | Return Type              | Description                              | ArkTS Usage     |
| -------------------- | ------------------------------------------------------ | ------------------------ | ---------------------------------------- | --------------- |
| `search`             | `query: string`, `options: SearchOptionsPayload`       | `Promise<SearchResults>` | Execute search with cancellation support | SearchScreen    |
| `getNodesInfo`       | `slabIndices: number[]`, `includeIcons: boolean`       | `Promise<NodeInfo[]>`    | Expand slab indices to full node info    | FileListView    |
| `getSortedView`      | `slabIndices: number[]`, `sortState: SortStatePayload` | `Promise<SlabIndex[]>`   | Get sorted view of results               | SortableTable   |
| `updateIconViewport` | `viewportId: number`, `slabIndices: number[]`          | `void`                   | Update visible items for icon prefetch   | VirtualizedList |
| `triggerRescan`      | -                                                      | `Promise<void>`          | Force full filesystem rescan             | SettingsScreen  |
| `setWatchConfig`     | `watchRoot: string`, `ignorePaths: string[]`           | `Promise<void>`          | Update watch configuration               | SettingsScreen  |

---

### 2. File Operations

| Command               | Parameters     | Return Type             | Description                    | ArkTS Usage  |
| --------------------- | -------------- | ----------------------- | ------------------------------ | ------------ |
| `openFile`            | `path: string` | `Promise<void>`         | Open file with default handler | ContextMenu  |
| `revealInFileManager` | `path: string` | `Promise<void>`         | Reveal in HarmonyOS Files app  | ContextMenu  |
| `getFileMetadata`     | `path: string` | `Promise<FileMetadata>` | Get extended file metadata     | DetailsPanel |

---

### 3. Application Control

| Command                  | Parameters                                   | Return Type          | Description                 | ArkTS Usage  |
| ------------------------ | -------------------------------------------- | -------------------- | --------------------------- | ------------ |
| `getAppStatus`           | -                                            | `Promise<AppStatus>` | Get current lifecycle state | AppRoot      |
| `initializeBackend`      | `watchRoot: string`, `ignorePaths: string[]` | `Promise<void>`      | Initialize Rust backend     | AppRoot      |
| `suspendBackgroundTasks` | -                                            | `void`               | Suspend background work     | LifecycleMgr |
| `resumeBackgroundTasks`  | -                                            | `void`               | Resume background work      | LifecycleMgr |
| `persistCache`           | -                                            | `Promise<void>`      | Request cache persistence   | LifecycleMgr |

---

## Modern Task Features

### Cancellable Tasks with AbortSignal

```rust
use napi_ohos::bindgen_prelude::*;
use tokio::task;

#[napi]
pub async fn cancellable_search(
    query: String,
    options: SearchOptionsPayload,
    signal: Option<AbortSignal>,
) -> Result<SearchResults> {
    let result = task::spawn_blocking(move || {
        // Check for cancellation periodically
        perform_search_with_cancellation(&query, &options, signal)
    }).await?;

    result.map_err(|e| Error::new(
        Status::GenericFailure,
        format!("Search cancelled or failed: {}", e)
    ))
}
```

### AsyncTask for Complex Long-running Operations

```rust
use napi_ohos::{Task, Env};

struct ComplexSearchTask {
    query: String,
    options: SearchOptionsPayload,
}

impl Task for ComplexSearchTask {
    type Output = SearchResults;
    type JsValue = JsObject;

    fn compute(&mut self) -> Result<Self::Output> {
        compute_search_results(&self.query, &self.options)
    }

    fn resolve(&mut self, env: Env, output: Self::Output) -> Result<Self::JsValue> {
        serialize_search_results(env, output)
    }
}

#[napi]
fn async_complex_search(query: String, options: SearchOptionsPayload) -> AsyncTask<ComplexSearchTask> {
    AsyncTask::new(ComplexSearchTask { query, options })
}
```

### Class Export with Async Methods

```rust
#[napi(js_name = "SearchEngine")]
pub struct SearchEngine {
    cache: Arc<SearchCache>,
}

#[napi]
impl SearchEngine {
    #[napi(constructor)]
    pub fn new() -> Self {
        SearchEngine {
            cache: Arc::new(SearchCache::default()),
        }
    }

    #[napi]
    pub async fn search(&self, query: String) -> Result<SearchResults> {
        self.cache.search(&query).await
    }

    #[napi]
    pub async fn prefetch(&self, terms: Vec<String>) -> Result<bool> {
        self.cache.prefetch(&terms).await
    }
}
```

---

## Data Types

### Shared Data Structures

```typescript
// Search options payload
interface SearchOptionsPayload {
  caseInsensitive?: boolean;
  maxResults?: number;
  // ... other search parameters
}

// Node information
interface NodeInfo {
  slabIndex: number;
  path: string;
  metadata?: NodeMetadata;
  icon?: string; // base64 encoded icon
}

// Node metadata
interface NodeMetadata {
  type: number;
  size: number;
  modifiedTime: number;
  // ... other metadata fields
}
```

---

## Error Handling with Promise Rejection

### Promise-based Error Handling

```typescript
// ArkTS usage
try {
  const results = await search(query, options);
  // Handle successful results
} catch (error) {
  // Error automatically propagated through Promise rejection
  console.error("Search failed:", error.message);
}
```

### Rust Error Types Mapping

```rust
use napi_ohos::{Error, Status};

#[napi]
pub async fn perform_operation() -> Result<()> {
    // Different error types map to different Promise rejections
    if permission_denied() {
        return Err(Error::new(
            Status::PermissionDenied,
            "Access denied to filesystem"
        ));
    }

    if timeout_occurred() {
        return Err(Error::new(
            Status::GenericFailure,
            "Operation timed out"
        ));
    }

    Ok(())
}
```

### Task Cancellation Support

```rust
use napi_ohos::bindgen_prelude::AbortSignal;

#[napi]
pub async fn cancellable_operation(
    signal: AbortSignal,
) -> Result<bool> {
    // Periodically check for cancellation
    for i in 0..100 {
        if signal.aborted() {
            return Err(Error::new(Status::Cancelled, "Operation cancelled"));
        }

        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        // Continue with operation...
    }

    Ok(true)
}
```

---

## Lifecycle Integration

### Command Availability by State

| App State     | Available Commands                       |
| ------------- | ---------------------------------------- |
| UNINITIALIZED | `initializeBackend`                      |
| INITIALIZING  | None (blocking state)                    |
| INDEXING      | `getAppStatus`, `suspendBackgroundTasks` |
| READY         | All commands                             |
| UPDATING      | Read-only commands                       |
| ERROR         | Limited recovery commands                |

---

## Performance Considerations

1. **Batching**: Combine multiple node info requests when possible
2. **Debouncing**: Rate-limit rapid successive calls (e.g. viewport updates)
3. **Payload Size**: Limit maximum result set size for memory-constrained devices
4. **Task Pool Optimization**: Configure optimal worker threads for HarmonyOS

---

## Security Model

### Permission Requirements

| Command            | Required Permissions                 |
| ------------------ | ------------------------------------ |
| File operations    | `ohos.permission.FILE_ACCESS`        |
| Filesystem watch   | `ohos.permission.FILE_ACCESS`        |
| System integration | `ohos.permission.SYSTEM_INTEGRATION` |

### Data Validation

- Validate all input paths are within allowed directories
- Sanitize all input parameters to prevent injection attacks
- Limit maximum payload sizes

---

## Testing Guidelines

### Unit Testing

- Mock FFI boundary for isolated command tests
- Validate parameter serialization/deserialization
- Test error conditions and edge cases

### Integration Testing

- End-to-end command flow validation
- Cross-platform behavior consistency checks
- Performance benchmarking

### Manual Testing

- Permission denied scenarios
- Low memory conditions
- Background/foreground transitions

---

## Versioning and Compatibility

### Command Versioning

- Versioned command signatures (v1.search, v2.search)
- Deprecation policy for old versions
- Compatibility layer for mixed versions

### Data Format Evolution

- Additive changes only to data structures
- Default values for new fields
- Schema validation for backward compatibility

---

## Future Extensions

1. **Bulk Operations**: Add batch versions of commands
2. **Streaming Results**: Support for progressive result delivery
3. **Custom Commands**: Plugin system for domain-specific commands
4. **Cross-Device Ops**: Commands for HarmonyOS ecosystem integration

This IPC command interface provides a robust foundation for ArkUI-Rust communication while respecting HarmonyOS platform constraints and security requirements.
