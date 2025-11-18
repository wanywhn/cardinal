mod common;
use cardinal_syntax::*;
use common::*;

#[test]
fn block_06_filters_mix() {
    let s1 = parse_ok("folder:src ext:rs regex:.*\\.rs$");
    let p1 = as_and(&s1);
    assert!(p1.len() >= 3);
    let s2 = parse_ok("ext:rs folder:src dm:today");
    let p2 = as_and(&s2);
    let l2 = p2.len();
    filter_is_kind(&p2[l2 - 1], &FilterKind::DateModified);
    let s3 = parse_ok("dc:pastweek a b c");
    let p3 = as_and(&s3);
    let l3 = p3.len();
    filter_is_kind(&p3[l3 - 1], &FilterKind::DateCreated);
    let s4 = parse_ok("type:picture folder:assets a b");
    let p4 = as_and(&s4);
    assert!(p4.len() >= 3);
    let s5 = parse_ok("doc: a b c dm:today");
    let p5 = as_and(&s5);
    let l5 = p5.len();
    filter_is_kind(&p5[l5 - 1], &FilterKind::DateModified);
    let s6 = parse_ok("video: a b dc:pastweek");
    let p6 = as_and(&s6);
    let l6 = p6.len();
    filter_is_kind(&p6[l6 - 1], &FilterKind::DateCreated);
    let s7 = parse_ok("audio: ext:mp3 a b");
    let p7 = as_and(&s7);
    assert!(p7.len() >= 3);
    let s8 = parse_ok("folder:src !ext:md a");
    let p8 = as_and(&s8);
    assert!(p8.len() >= 2);
    let s9 = parse_ok("folder:src (!ext:md) a");
    let p9 = as_and(&s9);
    assert!(p9.len() >= 2);
    let s10 = parse_ok("(folder:src folder:components) ext:tsx");
    let p10 = as_and(&s10);
    assert!(p10.len() >= 2);
}
