use bincode::{Decode, Encode};
use std::ffi::CStr;

#[derive(Encode, Decode)]
pub struct NamePool {
    // e.g. `\0aaa\0bbb\0ccc\0`
    // \0 is used as a separator
    pool: Vec<u8>,
}

impl NamePool {
    pub fn new() -> Self {
        Self { pool: vec![b'\0'] }
    }

    pub fn len(&self) -> usize {
        self.pool.len()
    }

    pub fn push(&mut self, name: &str) -> usize {
        let start = self.pool.len();
        self.pool.extend_from_slice(name.as_bytes());
        self.pool.push(0);
        start
    }

    // returns index of the trailing \0 and the string
    fn get(&self, offset: usize) -> (usize, &str) {
        // as this function should only be called by ourselves
        debug_assert!(offset < self.pool.len());
        // offset seperates string like this `\0 aaa\0 bbb\0 ccc\0`
        let begin = self.pool[..offset]
            .iter()
            .rposition(|&x| x == 0)
            .map(|x| x + 1)
            .unwrap_or(0);
        let end = self.pool[offset..]
            .iter()
            .position(|&x| x == 0)
            .map(|x| x + offset)
            .unwrap_or(self.pool.len());
        (end, unsafe {
            std::str::from_utf8_unchecked(&self.pool[begin..end])
        })
    }

    pub fn search_substr<'a>(&'a self, substr: &'a str) -> impl Iterator<Item = &'a str> + 'a {
        let mut last_end = 0;
        memchr::memmem::find_iter(&self.pool, substr.as_bytes()).filter_map(move |x| {
            if x <= last_end {
                None
            } else {
                let (new_end, s) = self.get(x);
                last_end = new_end;
                Some(s)
            }
        })
    }

    pub fn search_subslice<'a>(&'a self, subslice: &'a [u8]) -> impl Iterator<Item = &'a str> + 'a {
        let mut last_end = 0;
        memchr::memmem::find_iter(&self.pool, subslice).filter_map(move |x| {
            if x <= last_end {
                None
            } else {
                let (new_end, s) = self.get(x);
                last_end = new_end;
                Some(s)
            }
        })
    }

    pub fn search_suffix<'a>(&'a self, suffix: &'a CStr) -> impl Iterator<Item = &'a str> + 'a {
        let mut last_end = 0;
        memchr::memmem::find_iter(&self.pool, suffix.to_bytes_with_nul()).filter_map(move |x| {
            if x <= last_end {
                None
            } else {
                let (new_end, s) = self.get(x);
                last_end = new_end;
                Some(s)
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_name_pool_get() {
        let mut pool = NamePool::new();
        let offset = pool.push("hello");
        assert_eq!(offset, 1);
        assert_eq!(pool.get(offset), (6, "hello"));
        assert_eq!(pool.get(0), (0, ""));
        assert_eq!(pool.get(1), (6, "hello"));
        assert_eq!(pool.get(2), (6, "hello"));
        assert_eq!(pool.get(3), (6, "hello"));
        assert_eq!(pool.get(4), (6, "hello"));
        assert_eq!(pool.get(5), (6, "hello"));
        assert_eq!(pool.get(6), (6, "hello"));

        let offset = pool.push("world");
        assert_eq!(offset, 7);
        assert_eq!(pool.get(offset), (12, "world"));
        assert_eq!(pool.get(8), (12, "world"));
        assert_eq!(pool.get(9), (12, "world"));
        assert_eq!(pool.get(10), (12, "world"));
        assert_eq!(pool.get(11), (12, "world"));
        assert_eq!(pool.get(12), (12, "world"));
    }

    #[test]
    fn test_name_pool_search() {
        let mut pool = NamePool::new();
        pool.push("hello");
        pool.push("world");
        pool.push("hello world");
        pool.push("hello world hello");

        let substr = "hello";
        let result: Vec<_> = pool.search_substr(substr).collect();
        assert_eq!(result.len(), 3);
        assert_eq!(result[0], "hello");
        assert_eq!(result[1], "hello world");
        assert_eq!(result[2], "hello world hello");

        let subslice = b"world";
        let result: Vec<_> = pool.search_subslice(subslice).collect();
        assert_eq!(result.len(), 3);
        assert_eq!(result[0], "world");
        assert_eq!(result[1], "hello world");
        assert_eq!(result[2], "hello world hello");
    }

    #[test]
    fn test_name_pool_search_suffix() {
        let mut pool = NamePool::new();
        pool.push("hello");
        pool.push("world");
        pool.push("hello world");
        pool.push("hello world hello");

        let suffix = c"world";
        let result: Vec<_> = pool.search_suffix(suffix).collect();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], "world");
        assert_eq!(result[1], "hello world");
    }
}
