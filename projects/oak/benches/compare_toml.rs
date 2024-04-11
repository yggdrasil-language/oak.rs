#![feature(new_range_api)]

use criterion::{Criterion, criterion_group, criterion_main};
use oak_core::{Lexer, ParseSession, Parser, source::SourceText};
use oak_toml::{language::TomlLanguage, lexer::TomlLexer, parser::TomlParser};
use std::hint::black_box;

fn generate_toml(n: usize) -> String {
    let mut s = String::with_capacity(n * 100);
    for i in 0..n {
        s.push_str(&format!("[server-{}]\nport = {}\nhost = \"127.0.0.1\"\nenabled = {}\ntags = [\"rust\", \"api\", \"v1\"]\n\n", i, 8000 + i, if i % 2 == 0 { "true" } else { "false" }))
    }
    s
}

fn bench_toml_comparison(c: &mut Criterion) {
    let lang = Box::leak(Box::new(TomlLanguage::default()));
    let lexer = TomlLexer::new(lang);
    let parser = TomlParser::new(lang);

    // 1. Small TOML
    {
        let mut group = c.benchmark_group("TOML_Small");
        let s = generate_toml(5);
        let src = SourceText::new(s.as_str());

        group.bench_function("oak_toml_lex", |b| {
            b.iter(|| {
                let mut session = ParseSession::<TomlLanguage>::new(16);
                let out = lexer.lex(black_box(&src), &[], &mut session);
                black_box(out);
            })
        });

        group.bench_function("oak_toml_parse", |b| {
            b.iter(|| {
                let mut session = ParseSession::<TomlLanguage>::new(16);
                let out = parser.parse(&src, &[], &mut session);
                black_box(out);
            })
        });

        // group.bench_function("toml_rs_parse", |b| {
        // b.iter(|| {
        // let val: toml::Value = toml::from_str(black_box(&s)).unwrap();
        // black_box(val);
        // })
        // });
        //
        // group.bench_function("toml_edit_parse", |b| {
        // b.iter(|| {
        // let doc = s.parse::<toml_edit::DocumentMut>().unwrap();
        // black_box(doc);
        // })
        // });
        group.finish()
    }

    // 2. Medium TOML
    {
        let mut group = c.benchmark_group("TOML_Medium");
        let s = generate_toml(50);
        let src = SourceText::new(s.as_str());

        group.bench_function("oak_toml_lex", |b| {
            b.iter(|| {
                let mut session = ParseSession::<TomlLanguage>::new(16);
                let out = lexer.lex(black_box(&src), &[], &mut session);
                black_box(out);
            })
        });

        group.bench_function("oak_toml_parse", |b| {
            b.iter(|| {
                let mut session = ParseSession::<TomlLanguage>::new(16);
                let out = parser.parse(&src, &[], &mut session);
                black_box(out);
            })
        });

        // group.bench_function("toml_rs_parse", |b| {
        // b.iter(|| {
        // let val: toml::Value = toml::from_str(black_box(&s)).unwrap();
        // black_box(val);
        // })
        // });
        //
        // group.bench_function("toml_edit_parse", |b| {
        // b.iter(|| {
        // let doc = s.parse::<toml_edit::DocumentMut>().unwrap();
        // black_box(doc);
        // })
        // });
        group.finish()
    }

    // 3. Large TOML
    {
        let mut group = c.benchmark_group("TOML_Large_500");
        let s = generate_toml(500);
        let src = SourceText::new(s.as_str());

        group.bench_function("oak_toml_lex", |b| {
            b.iter(|| {
                let mut session = ParseSession::<TomlLanguage>::new(16);
                let out = lexer.lex(black_box(&src), &[], &mut session);
                black_box(out);
            })
        });

        group.bench_function("oak_toml_parse", |b| {
            b.iter(|| {
                let mut session = ParseSession::<TomlLanguage>::new(16);
                let out = parser.parse(&src, &[], &mut session);
                black_box(out);
            })
        });

        // group.bench_function("toml_rs_parse", |b| {
        // b.iter(|| {
        // let val: toml::Value = toml::from_str(black_box(&s)).unwrap();
        // black_box(val);
        // })
        // });
        //
        // group.bench_function("toml_edit_parse", |b| {
        // b.iter(|| {
        // let doc = s.parse::<toml_edit::DocumentMut>().unwrap();
        // black_box(doc);
        // })
        // });
        group.finish()
    }
}

criterion_group!(benches, bench_toml_comparison);
criterion_main!(benches);
