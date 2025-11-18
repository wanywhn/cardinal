mod common;
use cardinal_syntax::*;
use common::*;

#[test]
fn block_01_metadata_tail() {
    let e1 = parse_ok("dm:today a b dc:pastweek");
    let p1 = as_and(&e1);
    let l1 = p1.len();
    filter_is_kind(&p1[l1 - 2], &FilterKind::DateModified);
    filter_is_kind(&p1[l1 - 1], &FilterKind::DateCreated);
    let e2 = parse_ok("a dm:today b dc:pastweek c");
    let p2 = as_and(&e2);
    let l2 = p2.len();
    filter_is_kind(&p2[l2 - 2], &FilterKind::DateModified);
    filter_is_kind(&p2[l2 - 1], &FilterKind::DateCreated);
    let e3 = parse_ok("dc:pastweek dm:today a b");
    let p3 = as_and(&e3);
    let l3 = p3.len();
    filter_is_kind(&p3[l3 - 2], &FilterKind::DateCreated);
    filter_is_kind(&p3[l3 - 1], &FilterKind::DateModified);
    let e4 = parse_ok("dm:today dm:pastweek a b");
    let p4 = as_and(&e4);
    let l4 = p4.len();
    filter_is_kind(&p4[l4 - 2], &FilterKind::DateModified);
    filter_is_kind(&p4[l4 - 1], &FilterKind::DateModified);
}

#[test]
fn branch_and_reorder_mixed() {
    let e = parse_ok("folder:src ext:rs report dm:today dc:pastweek");
    let p = as_and(&e);
    filter_is_kind(&p[p.len() - 2], &FilterKind::DateModified);
    filter_is_kind(&p[p.len() - 1], &FilterKind::DateCreated);
    filter_is_kind(&p[0], &FilterKind::Folder);
    filter_is_kind(&p[1], &FilterKind::Ext);
    word_is(&p[2], "report");
}

#[test]
fn branch_metadata_relative_order() {
    let e = parse_ok("dc:pastweek dm:today a b");
    let p = as_and(&e);
    let l = p.len();
    filter_is_kind(&p[l - 2], &FilterKind::DateCreated);
    filter_is_kind(&p[l - 1], &FilterKind::DateModified);
}

#[test]
fn branch_and_no_metadata() {
    let e = parse_ok("folder:src ext:rs report");
    let p = as_and(&e);
    filter_is_kind(&p[0], &FilterKind::Folder);
    filter_is_kind(&p[1], &FilterKind::Ext);
    word_is(&p[2], "report");
}

#[test]
fn metadata_tail_preserves_non_metadata_order() {
    let e = parse_ok("dm:today x y z dc:pastweek w");
    let p = as_and(&e);
    let l = p.len();
    word_is(&p[0], "x");
    word_is(&p[1], "y");
    word_is(&p[2], "z");
    word_is(&p[3], "w");
    filter_is_kind(&p[l - 2], &FilterKind::DateModified);
    filter_is_kind(&p[l - 1], &FilterKind::DateCreated);
}
