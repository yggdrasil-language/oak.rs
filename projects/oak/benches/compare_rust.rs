#![feature(new_range_api)]

use criterion::{Criterion, criterion_group, criterion_main};
use oak_core::{Lexer, Parser, parser::ParseSession, source::SourceText};
use oak_rust::{RustLanguage, RustLexer, RustParser};
use std::hint::black_box;

fn generate_rust(n: usize) -> String {
    let mut s = String::with_capacity(n * 200);
    s.push_str("fn main() {\n");
    for i in 0..n {
        s.push_str(&format!("    let var_{} = {} + {}\n    println!(\"value: {{}}\", var_{});\n", i, i, i * 2, i))
    }
    s.push_str("}\n");
    s
}

fn bench_rust_comparison(c: &mut Criterion) {
    let lang = Box::leak(Box::new(RustLanguage::default()));
    let parser = RustParser::new(lang);
    let lexer = RustLexer::new(lang);

    // 1. Small Rust
    {
        let mut group = c.benchmark_group("Rust_Small");
        let s = generate_rust(5);
        let src = SourceText::new(s.as_str());

        group.bench_function("oak_rust_lex", |b| {
            b.iter(|| {
                let mut session = ParseSession::<RustLanguage>::new(16);
                let out = lexer.lex(black_box(&src), &[], &mut session);
                black_box(out);
            })
        });

        group.bench_function("oak_rust_parse", |b| {
            b.iter(|| {
                let mut session = ParseSession::<RustLanguage>::new(16);
                let out = parser.parse(black_box(&src), &[], &mut session);
                black_box(out);
            })
        });

        group.bench_function("syn_parse", |b| {
            b.iter(|| {
                let file = syn::parse_file(black_box(&s)).unwrap();
                black_box(file);
            })
        });
        group.finish()
    }

    // 2. Medium Rust
    {
        let mut group = c.benchmark_group("Rust_Medium");
        let s = generate_rust(50);
        let src = SourceText::new(s.as_str());

        group.bench_function("oak_rust_lex", |b| {
            b.iter(|| {
                let mut session = ParseSession::<RustLanguage>::new(16);
                let out = lexer.lex(black_box(&src), &[], &mut session);
                black_box(out);
            })
        });

        group.bench_function("oak_rust_parse", |b| {
            b.iter(|| {
                let mut session = ParseSession::<RustLanguage>::new(16);
                let out = parser.parse(black_box(&src), &[], &mut session);
                black_box(out);
            })
        });

        group.bench_function("syn_parse", |b| {
            b.iter(|| {
                let file = syn::parse_file(black_box(&s)).unwrap();
                black_box(file);
            })
        });
        group.finish()
    }

    // 3. Large Rust
    {
        let mut group = c.benchmark_group("Rust_Large_500");
        let s = generate_rust(500);
        let src = SourceText::new(s.as_str());

        group.bench_function("oak_rust_lex", |b| {
            b.iter(|| {
                let mut session = ParseSession::<RustLanguage>::new(16);
                let out = lexer.lex(black_box(&src), &[], &mut session);
                black_box(out);
            })
        });

        group.bench_function("oak_rust_parse", |b| {
            b.iter(|| {
                let mut session = ParseSession::<RustLanguage>::new(16);
                let out = parser.parse(black_box(&src), &[], &mut session);
                black_box(out);
            })
        });

        group.bench_function("syn_parse", |b| {
            b.iter(|| {
                let file = syn::parse_file(black_box(&s)).unwrap();
                black_box(file);
            })
        });
        group.finish()
    }
}

criterion_group!(benches, bench_rust_comparison);
criterion_main!(benches);
