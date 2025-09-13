#![feature(str_from_raw_parts)]
mod cache_line;

use crate::cache_line::CacheLine;
use core::str;
use parking_lot::{Mutex, RwLock};
use serde::{Deserialize, Serialize};
use std::ffi::CStr;

const CACHE_LINE_CAPACITY: usize = 16 * 1024 * 1024;

#[derive(Serialize, Deserialize)]
pub struct NamePool<const CAPACITY: usize = CACHE_LINE_CAPACITY> {
    lines: Mutex<Vec<RwLock<CacheLine<CAPACITY>>>>,
}

impl<const CAPACITY: usize> NamePool<CAPACITY> {
    pub fn new() -> Self {
        Self {
            lines: Mutex::new(vec![RwLock::new(CacheLine::new())]),
        }
    }

    pub fn len(&self) -> usize {
        self.lines.lock().iter().map(|x| x.read().len()).sum()
    }

    /// This function add a name into last cache line, if the last cache line is
    /// full, a new cache line will be added.
    ///
    /// # Panic
    ///
    /// This function will panic if a new CacheLine cannot hold the given name.
    ///
    /// Returns (line_num, str_offset)
    ///
    /// One important feature of NamePool is that the returned offset is stable
    /// and won't be overwritten.
    pub fn push(&self, name: &str) -> (usize, usize) {
        let mut lines = self.lines.lock();
        // There is at least one cache line
        if let Some(ptr) = lines.last().unwrap().write().push(name) {
            return (lines.len() - 1, ptr);
        }
        let mut cache_line = CacheLine::new();
        let str_offset = cache_line
            .push(name)
            .expect("Cache line is not large enough to hold he given name");
        lines.push(RwLock::new(cache_line));
        (lines.len() - 1, str_offset)
    }

    pub fn get(&self, line_num: usize, offset: usize) -> &str {
        // since namepool doesn't overwrite, doesn't drop, the returned str is static
        let lines = self.lines.lock();
        let line = lines[line_num].read();
        let s = line.get(offset).1;
        unsafe { str::from_raw_parts(s.as_ptr(), s.len()) }
    }

    pub fn search_substr<'search, 'pool: 'search>(
        &'pool self,
        substr: &'search str,
    ) -> Vec<&'pool str> {
        self.lines
            .lock()
            .iter()
            .map(|x| {
                x.read()
                    .search_substr(substr)
                    .map(|s| unsafe { str::from_raw_parts(s.as_ptr(), s.len()) })
                    .collect::<Vec<_>>()
            })
            .flatten()
            .collect()
    }

    pub fn search_subslice<'search, 'pool: 'search>(
        &'pool self,
        subslice: &'search [u8],
    ) -> Vec<&'pool str> {
        self.lines
            .lock()
            .iter()
            .map(|x| {
                x.read()
                    .search_subslice(subslice)
                    .map(|s| unsafe { str::from_raw_parts(s.as_ptr(), s.len()) })
                    .collect::<Vec<_>>()
            })
            .flatten()
            .collect()
    }

    pub fn search_suffix<'search, 'pool: 'search>(
        &'pool self,
        suffix: &'search CStr,
    ) -> Vec<&'pool str> {
        self.lines
            .lock()
            .iter()
            .map(|x| {
                x.read()
                    .search_suffix(suffix)
                    .map(|s| unsafe { str::from_raw_parts(s.as_ptr(), s.len()) })
                    .collect::<Vec<_>>()
            })
            .flatten()
            .collect()
    }

    // prefix should starts with a \0, e.g. b"\0hello"
    pub fn search_prefix<'search, 'pool: 'search>(
        &'pool self,
        prefix: &'search [u8],
    ) -> Vec<&'pool str> {
        self.lines
            .lock()
            .iter()
            .map(|x| {
                x.read()
                    .search_prefix(prefix)
                    .map(|s| unsafe { str::from_raw_parts(s.as_ptr(), s.len()) })
                    .collect::<Vec<_>>()
            })
            .flatten()
            .collect()
    }

    // `exact` should starts with a '\0', and ends with a '\0',
    // e.g. b"\0hello\0"
    pub fn search_exact<'search, 'pool: 'search>(
        &'pool self,
        exact: &'search [u8],
    ) -> Vec<&'pool str> {
        self.lines
            .lock()
            .iter()
            .map(|x| {
                x.read()
                    .search_exact(exact)
                    .map(|s| unsafe { str::from_raw_parts(s.as_ptr(), s.len()) })
                    .collect::<Vec<_>>()
            })
            .flatten()
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let pool = NamePool::<1024>::new();
        assert_eq!(pool.lines.lock().len(), 1);
        assert_eq!(pool.len(), 1);
    }

    #[test]
    fn test_push_basic() {
        let pool = NamePool::<1024>::new();
        let (line_num, offset) = pool.push("hello");
        assert!(line_num < pool.lines.lock().len());
        let s = pool.get(line_num, offset);
        assert_eq!(s, "hello");
    }

    #[test]
    fn test_push_multiple() {
        let pool = NamePool::<1024>::new();
        let (line_num1, offset1) = pool.push("foo");
        let (line_num2, offset2) = pool.push("bar");
        let (line_num3, offset3) = pool.push("baz");
        assert!(line_num1 < pool.lines.lock().len());
        assert!(line_num2 < pool.lines.lock().len());
        assert!(line_num3 < pool.lines.lock().len());
        let s1 = pool.get(line_num1, offset1);
        let s2 = pool.get(line_num2, offset2);
        let s3 = pool.get(line_num3, offset3);
        assert_eq!(s1, "foo");
        assert_eq!(s2, "bar");
        assert_eq!(s3, "baz");
    }

    #[test]
    fn test_push_empty_string() {
        let pool = NamePool::<1024>::new();
        let (line_num, offset) = pool.push("");
        assert!(line_num < pool.lines.lock().len());
        let s = pool.get(line_num, offset);
        assert_eq!(s, "");
    }

    #[test]
    fn test_push_unicode() {
        let pool = NamePool::<1024>::new();
        let (line_num, offset) = pool.push("こんにちは");
        assert!(line_num < pool.lines.lock().len());
        let s = pool.get(line_num, offset);
        assert_eq!(s, "こんにちは");
    }

    #[test]
    fn test_search_substr() {
        let pool = NamePool::<1024>::new();
        pool.push("hello");
        pool.push("world");
        pool.push("hello world");
        pool.push("hello world hello");

        let substr = "hello";
        let result = pool.search_substr(substr);
        assert_eq!(result.len(), 3);
        assert!(result.contains(&"hello"));
        assert!(result.contains(&"hello world"));
        assert!(result.contains(&"hello world hello"));
    }

    #[test]
    fn test_search_subslice() {
        let pool = NamePool::<1024>::new();
        pool.push("hello");
        pool.push("world");
        pool.push("hello world");
        pool.push("hello world hello");

        let subslice = b"world";
        let result = pool.search_subslice(subslice);
        assert_eq!(result.len(), 3);
        assert!(result.contains(&"world"));
        assert!(result.contains(&"hello world"));
        assert!(result.contains(&"hello world hello"));
    }

    #[test]
    fn test_search_suffix() {
        let pool = NamePool::<1024>::new();
        pool.push("hello");
        pool.push("world");
        pool.push("hello world");
        pool.push("hello world hello");

        let suffix = CStr::from_bytes_with_nul(b"world\0").unwrap();
        let result = pool.search_suffix(suffix);
        assert_eq!(result.len(), 2);
        assert!(result.contains(&"world"));
        assert!(result.contains(&"hello world"));
    }

    #[test]
    fn test_search_prefix() {
        let pool = NamePool::<1024>::new();
        pool.push("hello");
        pool.push("world");
        pool.push("hello world");
        pool.push("hello world hello");

        let prefix = b"\0hello";
        let result = pool.search_prefix(prefix);
        assert_eq!(result.len(), 3);
        assert!(result.contains(&"hello"));
        assert!(result.contains(&"hello world"));
        assert!(result.contains(&"hello world hello"));
    }

    #[test]
    fn test_search_exact() {
        let pool = NamePool::<1024>::new();
        pool.push("hello");
        pool.push("world");
        pool.push("hello world");

        let exact = b"\0hello\0";
        let result = pool.search_exact(exact);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], "hello");

        let exact = b"\0world\0";
        let result = pool.search_exact(exact);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], "world");
    }

    #[test]
    fn test_search_nonexistent() {
        let pool = NamePool::<1024>::new();
        pool.push("hello");
        pool.push("world");

        let substr = "nonexistent";
        let result = pool.search_substr(substr);
        assert!(result.is_empty());

        let subslice = b"nonexistent";
        let result = pool.search_subslice(subslice);
        assert!(result.is_empty());
    }

    #[test]
    fn test_search_partial_match() {
        let pool = NamePool::<1024>::new();
        pool.push("hello");
        pool.push("world");
        pool.push("hell");

        let substr = "hell";
        let result = pool.search_substr(substr);
        assert_eq!(result.len(), 2);
        assert!(result.contains(&"hello"));
        assert!(result.contains(&"hell"));
    }

    #[test]
    fn test_search_unicode() {
        let pool = NamePool::<1024>::new();
        pool.push("こんにちは");
        pool.push("世界");
        pool.push("こんにちは世界");

        let substr = "世界";
        let result = pool.search_substr(substr);
        assert_eq!(result.len(), 2);
        assert!(result.contains(&"世界"));
        assert!(result.contains(&"こんにちは世界"));
    }

    #[test]
    fn test_search_prefix_nonexistent() {
        let pool = NamePool::<1024>::new();
        pool.push("hello");
        pool.push("world");

        let prefix = b"\0nonexistent";
        let result = pool.search_prefix(prefix);
        assert!(result.is_empty());
    }

    #[test]
    fn test_search_exact_nonexistent() {
        let pool = NamePool::<1024>::new();
        pool.push("hello");
        pool.push("world");

        let exact = b"\0nonexistent\0";
        let result = pool.search_exact(exact);
        assert!(result.is_empty());
    }

    #[test]
    #[should_panic(expected = "assertion `left == right` failed")]
    fn test_search_prefix_should_panic() {
        let pool = NamePool::<1024>::new();
        pool.push("hello");

        // This should panic because the prefix does not start with \0
        let prefix = b"hello";
        let _result = pool.search_prefix(prefix);
    }

    #[test]
    #[should_panic(expected = "assertion `left == right` failed")]
    fn test_search_exact_should_panic_no_leading_null() {
        let pool = NamePool::<1024>::new();
        pool.push("hello");

        // This should panic because the exact string does not start with \0
        let exact = b"hello\0";
        let _result = pool.search_exact(exact);
    }

    #[test]
    #[should_panic(expected = "assertion `left == right` failed")]
    fn test_search_exact_should_panic_no_trailing_null() {
        let pool = NamePool::<1024>::new();
        pool.push("hello");

        // This should panic because the exact string does not end with '\0'
        let exact = b"\0hello";
        let _result = pool.search_exact(exact);
    }
}
