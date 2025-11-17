//! Additional edge & boundary tests for date filters.
//! Focus: open ranges, leap day, ambiguous formats, boolean combos.

#[cfg(test)]
mod date_edges {
    use crate::{SearchCache, SlabIndex, SlabNodeMetadataCompact};
    use fswalk::{NodeFileType, NodeMetadata};
    use jiff::{Timestamp, civil::Date, tz::TimeZone};
    use std::{fs, num::NonZeroU64};
    use tempdir::TempDir;

    const DAY: i64 = 24 * 60 * 60;

    fn set_times(cache: &mut SearchCache, idx: SlabIndex, c: i64, m: i64) {
        cache.file_nodes[idx].metadata = SlabNodeMetadataCompact::some(NodeMetadata {
            r#type: NodeFileType::File,
            size: 0,
            ctime: NonZeroU64::new(c as u64),
            mtime: NonZeroU64::new(m as u64),
        });
    }

    fn ts(y: i32, m: u32, d: u32) -> i64 {
        let tz = TimeZone::system();
        let date = Date::new(y as i16, m as i8, d as i8).expect("valid");
        tz.to_zoned(date.at(12, 0, 0, 0)).expect("zoned").timestamp().as_second()
    }

    #[test]
    fn explicit_range_instead_of_open_range() {
        let tmp = TempDir::new("explicit_range").unwrap();
        fs::write(tmp.path().join("a.txt"), b"x").unwrap();
        fs::write(tmp.path().join("b.txt"), b"x").unwrap();
        fs::write(tmp.path().join("c.txt"), b"x").unwrap();
        let mut cache = SearchCache::walk_fs(tmp.path().to_path_buf());
        let a = cache.search("a.txt").unwrap()[0];
        let b = cache.search("b.txt").unwrap()[0];
        let c = cache.search("c.txt").unwrap()[0];
        set_times(&mut cache, a, ts(2024, 5, 1), ts(2024, 5, 1));
        set_times(&mut cache, b, ts(2024, 5, 10), ts(2024, 5, 10));
        set_times(&mut cache, c, ts(2024, 6, 1), ts(2024, 6, 1));
        // Explicit bounded range covering first half of May.
        let hits = cache.search("dm:2024-05-01-2024-05-15").unwrap();
        let mut names: Vec<_> = hits
            .iter()
            .filter(|i| cache.file_nodes[**i].metadata.file_type_hint() == NodeFileType::File)
            .map(|i| cache.node_path(*i).unwrap().file_name().unwrap().to_string_lossy().into_owned())
            .collect();
        names.sort();
        assert_eq!(names, vec!["a.txt", "b.txt"]);
    }

    #[test]
    fn leap_day_query() {
        let tmp = TempDir::new("leap_day").unwrap();
        fs::write(tmp.path().join("leap.txt"), b"x").unwrap();
        let mut cache = SearchCache::walk_fs(tmp.path().to_path_buf());
        let leap = cache.search("leap.txt").unwrap()[0];
        set_times(&mut cache, leap, ts(2024, 2, 29), ts(2024, 2, 29));
        let eq = cache.search("dm:=2024-02-29").unwrap();
        assert_eq!(eq.len(), 1);
        let range = cache.search("dm:2024-02-28-2024-03-01").unwrap();
        assert_eq!(range.len(), 1);
    }

    #[test]
    fn dual_format_distinct_dates() {
        let tmp = TempDir::new("dual_formats").unwrap();
        fs::write(tmp.path().join("fmt_dd_mm.txt"), b"x").unwrap();
        fs::write(tmp.path().join("fmt_mm_dd.txt"), b"x").unwrap();
        let mut cache = SearchCache::walk_fs(tmp.path().to_path_buf());
        let dd_idx = cache.search("fmt_dd_mm.txt").unwrap()[0];
        let mm_idx = cache.search("fmt_mm_dd.txt").unwrap()[0];
        // 02-03-2024 -> 2 Mar 2024; 03-02-2024 -> 3 Feb 2024
        set_times(&mut cache, dd_idx, ts(2024, 3, 2), ts(2024, 3, 2));
        set_times(&mut cache, mm_idx, ts(2024, 2, 3), ts(2024, 2, 3));
        let mar_hits = cache.search("dm:02-03-2024").unwrap();
        assert_eq!(mar_hits.len(), 1);
        let feb_hits = cache.search("dm:03-02-2024").unwrap();
        assert_eq!(feb_hits.len(), 1);
        // Ensure they are distinct
        let mar_name = cache
            .node_path(mar_hits[0])
            .unwrap()
            .file_name()
            .unwrap()
            .to_string_lossy()
            .into_owned();
        let feb_name = cache
            .node_path(feb_hits[0])
            .unwrap()
            .file_name()
            .unwrap()
            .to_string_lossy()
            .into_owned();
        assert_ne!(mar_name, feb_name);
    }

    #[test]
    fn boolean_or_between_dm_and_dc() {
        let tmp = TempDir::new("boolean_dm_dc").unwrap();
        fs::write(tmp.path().join("created.txt"), b"x").unwrap();
        fs::write(tmp.path().join("modified.txt"), b"x").unwrap();
        let mut cache = SearchCache::walk_fs(tmp.path().to_path_buf());
        let created_idx = cache.search("created.txt").unwrap()[0];
        let modified_idx = cache.search("modified.txt").unwrap()[0];
        let zoned_now = Timestamp::now().to_zoned(TimeZone::system());
        let today = zoned_now.date();
        let current_year = today.year();
        let last_year = current_year - 1;
        // created file: ctime last year, mtime this year (matches dc:lastyear, dm:thisyear)
        let now_ts = zoned_now.timestamp().as_second();
        set_times(&mut cache, created_idx, ts(last_year as i32, 7, 15), now_ts);
        // modified file: ctime this year, mtime this year (matches dm:thisyear only)
        set_times(&mut cache, modified_idx, now_ts, now_ts);
        let thisyear_or_lastyear = cache.search("dc:lastyear|dm:thisyear").unwrap();
        let count = thisyear_or_lastyear
            .iter()
            .filter(|i| cache.file_nodes[**i].metadata.file_type_hint() == NodeFileType::File)
            .count();
        assert_eq!(count, 2, "OR across dc/dm should include both files");
    }

    #[test]
    fn boolean_and_intersection() {
        let tmp = TempDir::new("boolean_and_intersection").unwrap();
        fs::write(tmp.path().join("weekly.txt"), b"x").unwrap();
        let mut cache = SearchCache::walk_fs(tmp.path().to_path_buf());
        let idx = cache.search("weekly.txt").unwrap()[0];
        let now = Timestamp::now().as_second();
        set_times(&mut cache, idx, now, now - 2 * DAY); // modified 2 days ago
        let hits = cache.search("dm:pastweek dm:thisyear").unwrap();
        let count = hits
            .iter()
            .filter(|i| cache.file_nodes[**i].metadata.file_type_hint() == NodeFileType::File)
            .count();
        assert_eq!(count, 1, "intersection should retain only the file node");
    }

    #[test]
    fn single_point_range_equivalence() {
        let tmp = TempDir::new("single_point_range").unwrap();
        fs::write(tmp.path().join("point.txt"), b"x").unwrap();
        let mut cache = SearchCache::walk_fs(tmp.path().to_path_buf());
        let idx = cache.search("point.txt").unwrap()[0];
        set_times(&mut cache, idx, ts(2024, 5, 10), ts(2024, 5, 10));
        let eq_hits = cache.search("dm:=2024-05-10").unwrap();
        let range_hits = cache.search("dm:2024-05-10-2024-05-10").unwrap();
        assert_eq!(eq_hits.len(), range_hits.len());
    }
}
