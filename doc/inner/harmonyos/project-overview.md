# Cardinal Project Overview - HarmonyOS Edition

Cardinal is expanding to become a cross-platform desktop search application, supporting HarmonyOS alongside macOS and Linux. This document outlines how the pieces fit together for the HarmonyOS implementation, leveraging the existing Rust backend through FFI bindings and integrating with ArkUI for the frontend interface.

## High-level architecture

- **Frontend (ArkUI/ArkTS)**: HarmonyOS ArkUI interface built with declarative UI syntax. Communicates with Rust backend via FFI through ohrs toolkit. Handles search UI, results display, file operations, and system integration.
- **Rust FFI bridge**: Uses ohrs (OpenHarmony Rust toolkit) to expose Rust backend functionality to ArkTS. Implements command handlers for search, metadata retrieval, file operations, and system services.
- **Cross-platform Rust backend**: Reuses existing Cardinal components (search-cache, fswalk, cardinal-sdk, fs-icon) with platform abstraction layers. Maintains compatibility with existing macOS/Linux implementations while adding HarmonyOS support.
- **Filesystem events (cardinal-sdk)**: Cross-platform filesystem event monitoring adapted for HarmonyOS. Leverages HarmonyOS file system APIs to monitor changes and keep the search index synchronized.
- **Search engine (search-cache)**: Maintains an in-memory index of the filesystem with slab-based storage, name index, and metadata caching. Persists to disk and serves queries with highlighting and cancellation support.
- **Icon extraction (fs-icon)**: Cross-platform icon retrieval. HarmonyOS implementation currently uses stubbed/placholder implementations to ensure build and runtime compatibility. Full HarmonyOS icon extraction capabilities will be designed and implemented in future iterations.

Overall architecture:
```
┌─────────────────────────────────────────────────────────────────┐
│                    ArkUI/ArkTS frontend                         │
│  - Search bar / filters                                         │
│  - Status bar / overlays                                        │
└─────────────────▲───────────────────────────────────────────────┘
                  │ FFI calls via ohrs
                  │ (search, get_nodes_info, update_icon_viewport,
                  │  open_file, rescan, ...)
                  │
┌─────────────────┴───────────────────────────────────────────────┐
│                Rust FFI Bridge Layer                            │
│  - Command handlers for ArkTS                                  │
│  - Type conversions between Rust and ArkTS                     │
│  - Error handling and async task management                    │
└─────────────────▲───────────────────────────────────────────────┘
                  │ Crossbeam channels / internal APIs
                  │ (search_tx, node_info_tx, icon_viewport_tx,
                  │  rescan_tx, watch_config_tx, ...)
                  │
┌─────────────────┴───────────────────────────────────────────────┐
│              Cross-platform Rust Backend                        │
│  - SearchCache (slab + name index + metadata)                  │
│  - EventWatcher (HarmonyOS file system monitoring)             │
│  - fswalk (initial walk / rescans)                             │
│  - fs-icon (stubbed implementation for HarmonyOS)              │
│  - cardinal-sdk (filesystem abstractions)                      │
└─────────────────────────────────────────────────────────────────┘
```

## Key data flow

1) **Initialization**: ArkUI app initializes and calls into Rust FFI layer via ohrs. Rust backend loads persistent cache when possible (`SearchCache::try_read_persistent_cache`); otherwise walks the filesystem via `build_search_cache`.

2) **Index maintenance**: `EventWatcher` monitors filesystem changes using HarmonyOS APIs. The background loop feeds events to the cache; rescans are triggered on error conditions or when flags suggest the index may be stale.

3) **Search queries**: ArkTS UI sends search requests to Rust backend via FFI. The backend runs `cache.search_with_options`, returning result slab indices and highlights to the frontend.

4) **Metadata & icons**: `get_nodes_info` expands slab indices into paths/metadata. Icon retrieval is currently stubbed for HarmonyOS with placeholder implementations to ensure build and runtime compatibility. Full icon functionality will be implemented in future iterations.

5) **File operations**: ArkTS UI calls Rust backend to perform file operations like opening files/folders in default applications, using HarmonyOS-specific APIs for file handling.

6) **Configuration**: UI can update watch configuration via `set_watch_config`, which rebuilds the cache and watcher when values change.

## HarmonyOS-specific components

- **ArkUI integration**: Uses HarmonyOS ArkUI framework for native UI rendering with declarative syntax. Leverages HarmonyOS-specific components for optimal performance and user experience.
- **ohrs toolkit**: Utilizes the ohrs (OpenHarmony Rust) toolkit to facilitate Rust integration with HarmonyOS applications. Provides standardized interfaces between Rust code and ArkTS.
- **HarmonyOS filesystem APIs**: Adapts cardinal-sdk to use HarmonyOS-specific filesystem monitoring and file attribute APIs for optimal performance and compatibility.
- **HarmonyOS icon system**: Placeholder implementation for HarmonyOS icon extraction. Full integration with HarmonyOS system icon providers will be designed and implemented in future iterations.

## Backend layout (Rust workspace - HarmonyOS compatible)

- `cardinal-harmony/`: ArkUI frontend implementation and HarmonyOS-specific build configurations.
- `harmony-bindings/`: FFI layer using ohrs to expose Rust functionality to ArkTS. Contains type mappings and async task management.
- `search-cache/`: Core index, query engine, persistence, highlighting, and slab management (cross-platform).
- `fswalk/`: Filesystem walker used by the cache to build initial state (cross-platform).
- `cardinal-sdk/`: Cross-platform filesystem event bindings and helpers (extended for HarmonyOS).
- `fs-icon/`: Cross-platform icon extraction (stubbed implementation for HarmonyOS).
- `query-segmentation/`: Parses slash-delimited search tokens into prefix/suffix/exact/substr segments (cross-platform).
- `cardinal-syntax/`: Everything-style query parser (operators, filters, grouping) (cross-platform).
- `search-cancel/`: Cancellation token with versioning for aborting stale searches (cross-platform).
- `namepool/`: Process-wide string interner feeding the slab + name index (cross-platform).
- `slab-mmap/`: Memory-mapped slab allocator backing `search-cache`'s `ThinSlab` (cross-platform).

## Platform compatibility considerations

- **API differences**: HarmonyOS filesystem and UI APIs differ from macOS/Linux; abstraction layers ensure consistent behavior across platforms.
- **Permissions**: HarmonyOS permission model requires specific handling for filesystem access; integrates with HarmonyOS security framework.
- **Performance**: Optimized for HarmonyOS devices with varying hardware capabilities; leverages HarmonyOS-specific optimizations where possible.
- **UI consistency**: ArkUI frontend follows HarmonyOS design guidelines for consistent user experience.

## Development workflow

- **Rust**: `cargo check --workspace`, `cargo test --workspace`, `cargo clippy --workspace --all-targets`, `cargo fmt --all`. Toolchain pinned via `rust-toolchain.toml`.
- **HarmonyOS**: Use DevEco Studio for ArkUI development; integrate with Rust components via ohrs toolkit. Build process includes Rust compilation for HarmonyOS targets using NDK.
- **Cross-compilation**: Rust components compiled for HarmonyOS using appropriate target triples and HarmonyOS NDK.

## Testing strategy

- Unit tests for Rust components remain platform-agnostic
- Integration tests validate FFI layer functionality
- ArkUI components tested with HarmonyOS emulator/device
- Cross-platform compatibility verified through shared test suites

## Release considerations

- Distribute as HarmonyOS Application Package (HAP)
- Optimize for HarmonyOS app store requirements
- Ensure minimal resource usage on HarmonyOS devices
- Follow HarmonyOS security and privacy guidelines
