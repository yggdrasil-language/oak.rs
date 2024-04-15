use core::range::Range;

use oak_core::{Arc, source::Source};
use oak_vfs::{DocumentSession, MemoryVfs};

#[test]
fn document_session_bumps_revision_on_change() {
    let vfs = MemoryVfs::new();
    let first = vfs.open_document("file://a", Arc::<str>::from("a"));
    let second = vfs.change_document("file://a", Arc::<str>::from("ab"));
    assert_ne!(first.revision(), second.revision());
    assert_eq!(second.source().get_text_in(Range { start: 0, end: 2 }), "ab");
}

#[test]
fn request_source_prefers_open_buffer() {
    let vfs = MemoryVfs::new();
    vfs.write_file("file://a", Arc::<str>::from("disk"));
    vfs.open_document("file://a", Arc::<str>::from("buffer"));
    assert_eq!(vfs.request_source("file://a").unwrap().get_text_in(Range { start: 0, end: 6 }), "buffer");
    assert!(vfs.document_snapshot("file://a").is_some());
}

#[test]
fn close_document_clears_snapshot() {
    let vfs = MemoryVfs::new();
    vfs.open_document("file://a", Arc::<str>::from("open"));
    assert!(vfs.document_snapshot("file://a").is_some());
    vfs.close_document("file://a");
    assert!(vfs.document_snapshot("file://a").is_none());
}
