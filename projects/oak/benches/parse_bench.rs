#![feature(new_range_api)]

use core::range::Range;
use criterion::{Criterion, criterion_group, criterion_main};
use oak_core::{
    Parser,
    parser::ParseSession,
    source::{SourceText, TextEdit},
};
use oak_json::{JsonLanguage, JsonParser};
use std::hint::black_box;

fn large_json(n: usize) -> String {
    let mut s = String::with_capacity(n * 64);
    s.push_str("{\"items\": [");
    for i in 0..n {
        if i > 0 {
            s.push_str(",");
        }
        s.push_str(&format!("{{\"id\":{},\"name\":\"item-{}\",\"price\":{},\"tags\":[\"a\",\"b\",\"c\"],\"active\":{}}}", i, i, (i as f64) * 1.2345, if i % 3 == 0 { "true" } else { "false" }));
    }
    s.push_str("]}");
    s
}

fn bench_full_parse(c: &mut Criterion) {
    let lang = Box::leak(Box::new(JsonLanguage::default()));
    let parser = JsonParser::new(lang);
    let s = large_json(500);
    let src = SourceText::new(s.as_str());

    c.bench_function("oak_json_full_parse_500", |b| {
        b.iter(|| {
            let mut session = ParseSession::<JsonLanguage>::new(16);
            let out = parser.parse(&src, &[], &mut session);
            black_box(out);
        })
    });

    // c.bench_function("serde_json_parse_500", |b| {
    // b.iter(|| {
    // let v: serde_json::Value = serde_json::from_str(black_box(&s)).unwrap();
    // black_box(v);
    // })
    // })
}

fn bench_incremental_parse(c: &mut Criterion) {
    let lang = Box::leak(Box::new(JsonLanguage::default()));
    let parser = JsonParser::new(lang);
    let s1 = large_json(500);
    let mut s2 = s1.clone();
    // Modify a value in the middle
    let change_pos = 1000;
    let new_text = "\"modified\"";
    s2.replace_range(change_pos..change_pos + 10, new_text);
    let src1 = SourceText::new(s1.as_str());
    let src2 = SourceText::new(s2.as_str());
    let edits = vec![TextEdit { span: Range { start: change_pos, end: change_pos + 10 }, text: new_text.into() }];

    c.bench_function("oak_json_incremental_parse_500", |b| {
        b.iter_batched(
            || {
                // Setup: Perform the first parse to build results
                let mut session = ParseSession::<JsonLanguage>::new(16);
                parser.parse(&src1, &[], &mut session);
                session
            },
            |mut session| {
                // Incremental parse
                let out = parser.parse(&src2, &edits, &mut session);
                black_box(out);
            },
            criterion::BatchSize::SmallInput,
        );
    });
}

criterion_group!(benches, bench_full_parse, bench_incremental_parse);
criterion_main!(benches);
