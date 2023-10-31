use oak_core::{ParseSession, Parser, Source, SourceText};
use oak_typescript::{TypeScriptLanguage, TypeScriptParser};
use std::{
    alloc::{GlobalAlloc, Layout, System},
    sync::atomic::{AtomicBool, AtomicUsize, Ordering},
    time::{Duration, Instant},
};

struct BoundedAllocator;

static TRACKING: AtomicBool = AtomicBool::new(false);
static BYTES: AtomicUsize = AtomicUsize::new(0);

#[global_allocator]
static ALLOCATOR: BoundedAllocator = BoundedAllocator;

fn count_allocation(size: usize) {
    if TRACKING.load(Ordering::Relaxed) && BYTES.fetch_add(size, Ordering::Relaxed) + size > 8 * 1024 * 1024 {
        std::process::abort();
    }
}

unsafe impl GlobalAlloc for BoundedAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        count_allocation(layout.size());
        unsafe { System.alloc(layout) }
    }

    unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
        unsafe { System.dealloc(pointer, layout) }
    }

    unsafe fn realloc(&self, pointer: *mut u8, layout: Layout, size: usize) -> *mut u8 {
        count_allocation(size);
        unsafe { System.realloc(pointer, layout, size) }
    }
}

#[test]
fn object_members_have_bounded_allocations() {
    let language = TypeScriptLanguage::default();
    let parser = TypeScriptParser::new(&language);
    for text in [r#"const value = { "key": 1 };"#, r#"const value = { 0: 1, default: 2, [name]: 3 };"#, r#"const value = { method() { return 1; }, ...other, plain };"#] {
        eprintln!("parsing {text}");
        let source = SourceText::new(text);
        let mut session = ParseSession::default();
        BYTES.store(0, Ordering::Relaxed);
        TRACKING.store(true, Ordering::Relaxed);
        let start = Instant::now();
        let parsed = parser.parse(&source, &[], &mut session);
        TRACKING.store(false, Ordering::Relaxed);
        assert!(!parsed.has_errors(), "{:?}", parsed.diagnostics);
        assert_eq!(parsed.result.unwrap().byte_length as usize, source.length());
        assert!(start.elapsed() < Duration::from_millis(100));
        assert!(BYTES.load(Ordering::Relaxed) < 1024 * 1024);
    }
}
