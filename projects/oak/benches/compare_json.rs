#![feature(new_range_api)]

use criterion::{Criterion, criterion_group, criterion_main};
use oak_core::{Lexer, Parser, parser::ParseSession, source::SourceText};
use oak_json::{language::JsonLanguage, lexer::JsonLexer, parser::JsonParser};
use std::hint::black_box;

fn small_json() -> String {
    include_str!("complex.json").to_string()
}

fn medium_json() -> String {
    r#"{
        "user": {"id": 123, "name": "Bob", "tags": ["dev","ops","rust"], "meta": {"created": "2024-01-01", "enabled": true}},
        "items": [
            {"id":1,"name":"item-1","price":1.23,"attrs":{"color":"red","size":"S"}},
            {"id":2,"name":"item-2","price":2.34,"attrs":{"color":"green","size":"M"}},
            {"id":3,"name":"item-3","price":3.45,"attrs":{"color":"blue","size":"L"}}
        ],
        "ok": true,
        "count": 3
    }"#
    .to_string()
}

fn large_json(n: usize) -> String {
    let mut s = String::with_capacity(n * 64);
    s.push_str("{\"items\": [");
    for i in 0..n {
        if i > 0 {
            s.push_str(",")
        }
        s.push_str(&format!("{{\"id\":{},\"name\":\"item-{}\",\"price\":{},\"tags\":[\"a\",\"b\",\"c\"],\"active\":{}}}", i, i, (i as f64) * 1.2345, if i % 3 == 0 { "true" } else { "false" }))
    }
    s.push_str("]}");
    s
}

fn bench_json_comparison(c: &mut Criterion) {
    let lang = Box::leak(Box::new(JsonLanguage::standard()));
    let lexer = JsonLexer::new(lang);
    let parser = JsonParser::new(lang);

    // 1. Small JSON
    {
        let mut group = c.benchmark_group("JSON_Small");
        let s = small_json();
        let src = SourceText::new(s.as_str());

        group.bench_function("oak_json_lex", |b| {
            b.iter(|| {
                let mut session = ParseSession::<JsonLanguage>::new(16);
                let out = lexer.lex(black_box(&src), &[], &mut session);
                black_box(out);
            })
        });

        group.bench_function("oak_json_parse", |b| {
            b.iter(|| {
                let mut session = ParseSession::<JsonLanguage>::new(16);
                let out = parser.parse(black_box(&src), &[], &mut session);
                black_box(out);
            })
        });

        // group.bench_function("serde_json_parse", |b| {
        // b.iter(|| {
        // let v: serde_json::Value = serde_json::from_str(black_box(&s)).unwrap();
        // black_box(v);
        // })
        // });
        //
        // group.bench_function("simd_json_parse", |b| {
        // let bytes = s.clone().into_bytes();
        // b.iter_batched(
        // || bytes.clone(),
        // |mut b| {
        // let v = simd_json::to_borrowed_value(&mut b).unwrap();
        // black_box(v);
        // },
        // criterion::BatchSize::SmallInput,
        // )
        // });
        //
        // group.bench_function("json_rust_parse", |b| {
        // b.iter(|| {
        // let v = json::parse(black_box(&s)).unwrap();
        // black_box(v);
        // })
        // });
        group.finish()
    }

    // 2. Medium JSON
    {
        let mut group = c.benchmark_group("JSON_Medium");
        let m = medium_json();
        let src = SourceText::new(m.as_str());

        group.bench_function("oak_json_lex", |b| {
            b.iter(|| {
                let mut session = ParseSession::<JsonLanguage>::new(16);
                let out = lexer.lex(black_box(&src), &[], &mut session);
                black_box(out);
            })
        });

        group.bench_function("oak_json_parse", |b| {
            b.iter(|| {
                let mut session = ParseSession::<JsonLanguage>::new(16);
                let out = parser.parse(black_box(&src), &[], &mut session);
                black_box(out);
            })
        });

        // group.bench_function("serde_json_parse", |b| {
        // b.iter(|| {
        // let v: serde_json::Value = serde_json::from_str(black_box(&m)).unwrap();
        // black_box(v);
        // })
        // });
        //
        // group.bench_function("simd_json_parse", |b| {
        // let bytes = m.clone().into_bytes();
        // b.iter_batched(
        // || bytes.clone(),
        // |mut b| {
        // let v = simd_json::to_borrowed_value(&mut b).unwrap();
        // black_box(v);
        // },
        // criterion::BatchSize::SmallInput,
        // )
        // });
        //
        // group.bench_function("json_rust_parse", |b| {
        // b.iter(|| {
        // let v = json::parse(black_box(&m)).unwrap();
        // black_box(v);
        // })
        // });
        group.finish()
    }

    // 3. Large JSON
    {
        let mut group = c.benchmark_group("JSON_Large_500");
        let l = large_json(500);
        let src = SourceText::new(l.as_str());

        group.bench_function("oak_json_lex", |b| {
            b.iter(|| {
                let mut session = ParseSession::<JsonLanguage>::new(16);
                let out = lexer.lex(black_box(&src), &[], &mut session);
                black_box(out);
            })
        });

        group.bench_function("oak_json_parse", |b| {
            b.iter(|| {
                let mut session = ParseSession::<JsonLanguage>::new(16);
                let out = parser.parse(black_box(&src), &[], &mut session);
                black_box(out);
            })
        });

        // group.bench_function("serde_json_parse", |b| {
        // b.iter(|| {
        // let v: serde_json::Value = serde_json::from_str(black_box(&l)).unwrap();
        // black_box(v);
        // })
        // });
        //
        // group.bench_function("simd_json_parse", |b| {
        // let bytes = l.clone().into_bytes();
        // b.iter_batched(
        // || bytes.clone(),
        // |mut b| {
        // let v = simd_json::to_borrowed_value(&mut b).unwrap();
        // black_box(v);
        // },
        // criterion::BatchSize::LargeInput,
        // )
        // });
        //
        // group.bench_function("json_rust_parse", |b| {
        // b.iter(|| {
        // let v = json::parse(black_box(&l)).unwrap();
        // black_box(v);
        // })
        // });
        group.finish()
    }
}

criterion_group!(benches, bench_json_comparison);
criterion_main!(benches);
