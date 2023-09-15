use oak_core::{Builder, GreenNode, GreenTree, ParseSession, Parser, Source, SourceText};
use oak_vue::{VueBuilder, VueLanguage, VueParser, ast::VueAttribute, ast::VueNode};
use std::{
    alloc::{GlobalAlloc, Layout, System},
    sync::atomic::{AtomicBool, AtomicUsize, Ordering},
    time::{Duration, Instant},
};

struct CountingAllocator;

static TRACKING: AtomicBool = AtomicBool::new(false);
static ALLOCATIONS: AtomicUsize = AtomicUsize::new(0);
static BYTES: AtomicUsize = AtomicUsize::new(0);

#[global_allocator]
static ALLOCATOR: CountingAllocator = CountingAllocator;

unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if TRACKING.load(Ordering::Relaxed) {
            ALLOCATIONS.fetch_add(1, Ordering::Relaxed);
            BYTES.fetch_add(layout.size(), Ordering::Relaxed);
        }
        unsafe { System.alloc(layout) }
    }

    unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
        unsafe { System.dealloc(pointer, layout) }
    }

    unsafe fn realloc(&self, pointer: *mut u8, layout: Layout, size: usize) -> *mut u8 {
        if TRACKING.load(Ordering::Relaxed) {
            ALLOCATIONS.fetch_add(1, Ordering::Relaxed);
            BYTES.fetch_add(size, Ordering::Relaxed);
        }
        unsafe { System.realloc(pointer, layout, size) }
    }
}

fn count_nodes(node: &GreenNode<'_, VueLanguage>) -> usize {
    1 + node.children.iter().map(|child| match child {
        GreenTree::Node(child) => count_nodes(child),
        GreenTree::Leaf(_) => 0,
    }).sum::<usize>()
}

#[test]
fn event_shell_directive_has_bounded_allocations_and_exact_spans() {
    let source = SourceText::new(r#"<button @click="(() => armed = !armed)">"#);
    let language = VueLanguage::default();
    let parser = VueParser::new(&language);
    let mut session = ParseSession::default();
    TRACKING.store(true, Ordering::Relaxed);
    let start = Instant::now();
    let parsed = parser.parse(&source, &[], &mut session);
    let elapsed = start.elapsed();
    TRACKING.store(false, Ordering::Relaxed);
    let root = parsed.result.expect("minimal event directive CST");
    let allocations = ALLOCATIONS.load(Ordering::Relaxed);
    let bytes = BYTES.load(Ordering::Relaxed);
    let nodes = count_nodes(root);
    assert!(elapsed < Duration::from_millis(100), "parse took {elapsed:?}");
    assert!(allocations < 1024, "{allocations} allocations");
    assert!(bytes < 1024 * 1024, "{bytes} allocated bytes");
    assert!(nodes < 128, "{nodes} nodes");
    assert_eq!(root.byte_length as usize, source.length());
    eprintln!("event_shell: {elapsed:?}, {allocations} allocations, {bytes} bytes, {nodes} nodes");

    let source = SourceText::new(r#"<template><button @click="(() => armed = !armed)" :title="label">{{ label }}</button><li v-for="item in items" :key="item.id">{{ item }}</li></template>"#);
    let mut session = ParseSession::default();
    let parsed = parser.parse(&source, &[], &mut session);
    assert_eq!(parsed.result.unwrap().byte_length as usize, source.length());
    let builder = VueBuilder::new();
    let mut session = ParseSession::default();
    let built = builder.build(&source, &[], &mut session).result.unwrap();
    let VueNode::Element(button) = &built.blocks[0].children[0] else {
        panic!("expected button");
    };
    let VueAttribute::Directive(click) = &button.attributes[0] else {
        panic!("expected event directive");
    };
    assert_eq!(source.get_text_in(click.value.as_ref().unwrap().span.clone()), "(() => armed = !armed)");
    let VueAttribute::Directive(title) = &button.attributes[1] else {
        panic!("expected title directive");
    };
    assert_eq!(source.get_text_in(title.value.as_ref().unwrap().span.clone()), "label");
    let VueNode::Element(list) = &built.blocks[0].children[1] else {
        panic!("expected list element");
    };
    assert_eq!(source.get_text_in(list.tag_name.clone()), "li");
}
