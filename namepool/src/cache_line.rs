use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::ffi::CStr;

#[derive(Serialize, Deserialize)]
pub struct CacheLine<const CAPACITY: usize> {
    // len: 9
    // data: b"\0aaa\0bbb\0\0....\0"
    len: usize,
    data: Box<[u8]>,
}

impl<const CAPACITY: usize> CacheLine<CAPACITY> {
    pub fn new() -> Self {
        Self {
            len: 1, // reserve a leading \0 guard
            data: vec![0; CAPACITY].into_boxed_slice(),
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn push(&mut self, name: &str) -> Option<usize> {
        let len = self.len;
        let name_len = name.as_bytes().len();
        // reserve an ending \0 guard
        if len + name_len + 1 > CAPACITY {
            return None;
        }
        self.data[len..len + name_len].copy_from_slice(name.as_bytes());
        self.len = len + name_len + 1;
        // since data is written, we can safely wrapping here
        Some(len)
    }

    // returns index of the trailing \0 and the string
    pub fn get(&self, offset: usize) -> (usize, &str) {
        // as this function should only be called by ourselves
        debug_assert!(offset < CAPACITY);
        // offset seperates string like this `\0 aaa\0 bbb\0 ccc\0`
        let begin = self.data[..offset]
            .iter()
            .rposition(|&x| x == 0)
            .map(|x| x + 1)
            .unwrap_or(0);
        let end = self.data[offset..]
            .iter()
            .position(|&x| x == 0)
            .map(|x| x + offset)
            .unwrap_or(self.data.len());
        (end, unsafe {
            std::str::from_utf8_unchecked(&self.data[begin..end])
        })
    }

    pub fn search_substr<'search, 'pool: 'search>(
        &'pool self,
        substr: &'search str,
    ) -> impl Iterator<Item = &'pool str> + 'search {
        memchr::memmem::find_iter(&self.data, substr.as_bytes())
            .map(|x| self.get(x))
            .dedup_by(|(x, _), (y, _)| x == y)
            .map(|(_, s)| s)
    }

    pub fn search_subslice<'search, 'pool: 'search>(
        &'pool self,
        subslice: &'search [u8],
    ) -> impl Iterator<Item = &'pool str> + 'search {
        memchr::memmem::find_iter(&self.data, subslice)
            .map(|x| self.get(x))
            .dedup_by(|(x, _), (y, _)| x == y)
            .map(|(_, s)| s)
    }

    pub fn search_suffix<'search, 'pool: 'search>(
        &'pool self,
        suffix: &'search CStr,
    ) -> impl Iterator<Item = &'pool str> + 'search {
        memchr::memmem::find_iter(&self.data, suffix.to_bytes_with_nul())
            .map(|x| self.get(x))
            .dedup_by(|(x, _), (y, _)| x == y)
            .map(|(_, s)| s)
    }

    // prefix should starts with a \0, e.g. b"\0hello"
    pub fn search_prefix<'search, 'pool: 'search>(
        &'pool self,
        prefix: &'search [u8],
    ) -> impl Iterator<Item = &'pool str> + 'search {
        assert_eq!(prefix[0], 0);
        memchr::memmem::find_iter(&self.data, prefix)
            // To make sure it points to the end of the prefix. If we use the begin index, we will get a string before the correct one.
            .map(|x| x + prefix.len() - 1)
            .map(|x| self.get(x))
            .dedup_by(|(x, _), (y, _)| x == y)
            .map(|(_, s)| s)
    }

    // `exact` should starts with a '\0', and ends with a '\0',
    // e.g. b"\0hello\0"
    pub fn search_exact<'search, 'pool: 'search>(
        &'pool self,
        exact: &'search [u8],
    ) -> impl Iterator<Item = &'pool str> + 'search {
        assert_eq!(exact[0], 0);
        assert_eq!(exact[exact.len() - 1], 0);
        memchr::memmem::find_iter(&self.data, exact)
            .map(|x| x + exact.len() - 1)
            .map(|x| self.get(x))
            .dedup_by(|(x, _), (y, _)| x == y)
            .map(|(_, s)| s)
    }
}

#[cfg(test)]
mod cacheline_tests {
    use super::*;

    #[test]
    fn test_cacheline_push() {
        const CAPACITY: usize = 4 * 1024 * 1024;
        let mut cl = CacheLine::<CAPACITY>::new();
        assert_eq!(cl.len, 1);
        let p1 = cl.push("aaa").unwrap();
        assert_eq!(cl.len, 5);
        assert_eq!(cl.get(p1).1, "aaa");
        let p2 = cl.push("bbb").unwrap();
        assert_eq!(cl.len, 9);
        assert_eq!(cl.get(p2).1, "bbb");
    }

    #[test]
    fn test_cacheline_max() {
        let mut cl = CacheLine::<9>::new();
        assert_eq!(cl.len, 1);
        let p1 = cl.push("aaa").unwrap();
        assert_eq!(cl.len, 5);
        assert_eq!(cl.get(p1).1, "aaa");
        let p2 = cl.push("bbb").unwrap();
        assert_eq!(cl.len, 9);
        assert_eq!(cl.get(p2).1, "bbb");
        assert!(cl.push("").is_none());
        assert!(cl.push("c").is_none());
        assert!(cl.push("cc").is_none());
    }

    #[test]
    fn test_cacheline_full() {
        let mut cl = CacheLine::<4000>::new();
        let mut last_ptr = None;
        for i in 0.. {
            let name = format!("name{}", i);
            if let Some(p) = cl.push(&name) {
                last_ptr = Some((p, name));
            } else {
                break;
            }
        }
        assert!(last_ptr.is_some());
        let (p, name) = last_ptr.unwrap();
        assert_eq!(cl.get(p).1, name);
        dbg!(&name);
        assert!(cl.push(&(name + "!")).is_none());
    }
}
