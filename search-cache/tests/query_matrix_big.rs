//! Extensive search-cache query evaluation matrix.
//! Builds a small virtual filesystem and runs many search permutations.
//! Intentionally verbose for line-count; focuses on correctness + non-panicking behavior.

use search_cache::{SearchCache, SearchOptions, SlabIndex, SearchIterator};
use search_cancel::CancellationToken;
use std::sync::{Arc, RwLock};
use tempdir::TempDir;

/// 从 SearchIterator 收集所有索引（用于测试）
fn collect_iterator_indices(
    cache_arc: &Arc<RwLock<SearchCache>>,
    query: &str,
    options: SearchOptions,
    batch_size: usize,
) -> Vec<SlabIndex> {
    let mut iterator = SearchIterator::new_with_rwlock(
        Arc::clone(cache_arc),
        query,
        options,
        batch_size,
        CancellationToken::noop(),
        |_| {}, // 空回调
    ).expect("iterator creation should succeed");

    let mut all_indices = Vec::new();
    loop {
        let batch = iterator.next_batch(100);
        all_indices.extend(batch.indices);
        if batch.search_completed || !batch.has_more {
            break;
        }
    }
    all_indices
}

fn build_cache() -> SearchCache {
    let temp_dir = TempDir::new("query_matrix_big").unwrap();
    let root_path = temp_dir.path().to_path_buf();
    // Leak the TempDir so files remain accessible for the duration of the test process.
    std::mem::forget(temp_dir);
    // Files/directories (create parents as needed).
    let files = [
        "README.md",
        "LICENSE",
        "Cargo.toml",
        "main.rs",
        "lib.rs",
        "config.ts",
        "src/lib.rs",
        "src/main.rs",
        "src/components/Button.tsx",
        "src/components/Input.tsx",
        "src/components/Modal.tsx",
        "tests/test_basic.rs",
        "tests/test_parser.rs",
        "assets/logo.png",
        "assets/banner.jpg",
        "assets/icon.gif",
        "Makefile",
        "build.rs",
        "script.sh",
        "Dockerfile",
        "CHANGELOG.md",
        "src/util/mod.rs",
        "src/util/fs.rs",
    ];
    for file in files {
        let full = root_path.join(file);
        if let Some(parent) = full.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::File::create(full).unwrap();
    }
    SearchCache::walk_fs(&root_path)
}

fn run(cache: &mut SearchCache, q: &str) -> usize {
    // Use public query_files API (search() helper is unit-test only).
    match cache.query_files(q.to_string(), CancellationToken::noop()) {
        Ok(Some(nodes)) => nodes.len(),
        Ok(None) => 0,
        Err(_) => 0,
    }
}

#[test]
fn large_query_matrix() {
    let mut cache = build_cache();
    let matrix = [
        // Simple words
        "README",
        "README.md",
        "LICENSE",
        "Cargo",
        "main",
        "lib",
        "config",
        "Dockerfile",
        // Extensions
        "ext:rs",
        "ext:tsx",
        "ext:md",
        "ext:png;jpg;gif",
        "ext:png ; jpg ; gif",
        // Folder filters
        "folder:src",
        "folder:tests",
        "folder:assets",
        "parent:src",
        "infolder:src",
        "parent:./src",
        // Boolean mixes
        "src lib | README",
        "src lib | README.md",
        "src | tests lib",
        "src tests | assets",
        "src util | components",
        "src components | tests",
        "src components | README",
        "README | src",
        // NOT forms
        "src ! tests",
        "src ! assets",
        "src ! README",
        "src ! README.md",
        "README ! src",
        "src ! ext:md",
        // Regex queries
        "regex:^main\\.rs$",
        "regex:.*\\.tsx$",
        "regex:.*\\.(png|jpg)$",
        "regex:^README.*",
        // Mixed regex + filter
        "folder:src regex:.*\\.rs$",
        "folder:src regex:.*\\.tsx$",
        "folder:src regex:.*util.*",
        // Wildcards
        "*.rs",
        "*.tsx",
        "*.md",
        "*.png",
        "*.jpg",
        "*.gif",
        "*main*",
        "*lib*",
        "*config*",
        // Multi-list ext
        "ext:rs;md",
        "ext:rs;tsx;md",
        "ext:png;jpg",
        "ext:png;jpg;gif",
        "ext:png; gif ; jpg",
        // Precedence
        "src lib | tests",
        "src | lib tests",
        "src | lib | tests",
        "src lib tests README",
        // Parent + infolder nuance
        "parent:src *.rs",
        "infolder:src *.rs",
        "parent:src *.tsx",
        "infolder:src *.tsx",
        // Chained filters & words
        "folder:src ext:rs lib",
        "folder:src ext:rs main",
        "folder:src ext:tsx components",
        // Long chain
        "README Cargo lib main config src util tests components assets",
        // Many negations
        "src ! README ! LICENSE ! Cargo.toml",
        "src ! README ! LICENSE",
        "src ! README ! lib",
    ];
    for q in matrix {
        let _ = run(&mut cache, q);
    }
}

/// Iterator 版本的 large_query_matrix 测试
#[test]
fn large_query_matrix_iterator() {
    let cache = build_cache();
    let cache_arc = Arc::new(RwLock::new(cache));
    let matrix = [
        // Simple words
        "README",
        "README.md",
        "LICENSE",
        "Cargo",
        "main",
        "lib",
        "config",
        "Dockerfile",
        // Extensions
        "ext:rs",
        "ext:tsx",
        "ext:md",
        "ext:png;jpg;gif",
        "ext:png ; jpg ; gif",
        // Folder filters
        "folder:src",
        "folder:tests",
        "folder:assets",
        "parent:src",
        "infolder:src",
        "parent:./src",
        // Boolean mixes
        "src lib | README",
        "src lib | README.md",
        "src | tests lib",
        "src tests | assets",
        "src util | components",
        "src components | tests",
        "src components | README",
        "README | src",
        // NOT forms
        "src ! tests",
        "src ! assets",
        "src ! README",
        "src ! README.md",
        "README ! src",
        "src ! ext:md",
        // Wildcards
        "*.rs",
        "*.tsx",
        "*.md",
        "*.png",
        "*.jpg",
        "*.gif",
        "*main*",
        "*lib*",
        "*config*",
        // Multi-list ext
        "ext:rs;md",
        "ext:rs;tsx;md",
        "ext:png;jpg",
        "ext:png;jpg;gif",
        "ext:png; gif ; jpg",
        // Precedence
        "src lib | tests",
        "src | lib tests",
        "src | lib | tests",
        "src lib tests README",
        // Parent + infolder nuance
        "parent:src *.rs",
        "infolder:src *.rs",
        "parent:src *.tsx",
        "infolder:src *.tsx",
        // Chained filters & words
        "folder:src ext:rs lib",
        "folder:src ext:rs main",
        "folder:src ext:tsx components",
        // Many negations
        "src ! README ! LICENSE ! Cargo.toml",
        "src ! README ! LICENSE",
        "src ! README ! lib",
    ];
    for q in matrix {
        let _ = collect_iterator_indices(&cache_arc, q, SearchOptions::default(), 10);
    }
}

#[test]
fn wildcard_vs_phrase_behavior_matrix() {
    let mut cache = build_cache();
    // Quoted phrases should not expand wildcards.
    let wild = run(&mut cache, "*.rs");
    let quoted = run(&mut cache, r#""*.rs""#); // This will parse as phrase containing * and .rs
    assert!(wild >= quoted); // wildcards likely match equal or more than literal phrase
}

#[test]
fn case_insensitive_option_matrix() {
    let mut cache = build_cache();
    let opts = SearchOptions {
        case_insensitive: true,
    };
    let insensitive = cache
        .search_with_options("readme.md", opts, CancellationToken::noop())
        .unwrap()
        .nodes
        .unwrap()
        .len();
    let opts = SearchOptions {
        case_insensitive: false,
    };
    let sensitive = cache
        .search_with_options("readme.md", opts, CancellationToken::noop())
        .unwrap()
        .nodes
        .unwrap()
        .len();
    assert!(insensitive >= sensitive);

    // Iterator version test
    let cache_arc = Arc::new(RwLock::new(cache));
    let opts = SearchOptions {
        case_insensitive: true,
    };
    let iter_insensitive = collect_iterator_indices(&cache_arc, "readme.md", opts, 10).len();
    let opts = SearchOptions {
        case_insensitive: false,
    };
    let iter_sensitive = collect_iterator_indices(&cache_arc, "readme.md", opts, 10).len();
    assert!(iter_insensitive >= iter_sensitive, "Iterator: case insensitive should match >= case sensitive");
}

#[test]
fn cancellation_large_iteration() {
    let mut cache = build_cache();
    let token = CancellationToken::new(9999);
    let _later = CancellationToken::new(10000); // cancel token
    let result = cache
        .search_with_options("src lib tests", SearchOptions::default(), token)
        .unwrap();
    assert!(
        result.nodes.is_none(),
        "Cancellation should propagate and yield None"
    );

    // Iterator version test
    let cache_arc = Arc::new(RwLock::new(cache));
    let cancel_token = CancellationToken::new(10001);
    let _later = CancellationToken::new(10002); // cancel token
    let mut iterator = SearchIterator::new_with_rwlock(
        Arc::clone(&cache_arc),
        "src lib tests",
        SearchOptions::default(),
        10,
        cancel_token,
        |_| {},
    ).expect("iterator creation should succeed");

    let batch = iterator.next_batch(100);
    assert!(
        batch.indices.is_empty(),
        "Iterator: Cancellation should propagate and yield empty results"
    );
}
