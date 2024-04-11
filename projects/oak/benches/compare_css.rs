#![feature(new_range_api)]

use criterion::{Criterion, criterion_group, criterion_main};
use cssparser::{Parser as CssParserLib, ParserInput};
use oak_core::{ParseSession, Parser, source::SourceText};
use oak_css::{CssLanguage, CssParser};
use std::hint::black_box;

fn generate_css(n: usize) -> String {
    let mut s = String::with_capacity(n * 150);
    for i in 0..n {
        s.push_str(&format!(".container-{} {{\n  color: red;\n  margin: 10px;\n  padding: {}px;\n  border: 1px solid #ccc;\n  display: flex;\n  justify-content: center;\n}}\n\n", i, i % 20))
    }
    s
}

fn bench_css_comparison(c: &mut Criterion) {
    // 1. Small CSS
    {
        let mut group = c.benchmark_group("CSS_Small");
        let s = generate_css(5);
        let src = SourceText::new(s.as_str());
        let lang = Box::leak(Box::new(CssLanguage::default()));
        let parser = CssParser::new(lang);

        group.bench_function("oak_css_parse", |b| {
            b.iter(|| {
                let mut session = ParseSession::<CssLanguage>::new(16);
                let out = parser.parse(&src, &[], &mut session);
                black_box(out);
            })
        });

        group.bench_function("cssparser_lex", |b| {
            b.iter(|| {
                let mut input = ParserInput::new(black_box(&s));
                let mut css_parser = CssParserLib::new(&mut input);
                while let Ok(token) = css_parser.next() {
                    black_box(token);
                }
            })
        });
        group.finish()
    }

    // 2. Large CSS
    {
        let mut group = c.benchmark_group("CSS_Large_500");
        let s = generate_css(500);
        let src = SourceText::new(s.as_str());
        let lang = Box::leak(Box::new(CssLanguage::default()));
        let parser = CssParser::new(lang);

        group.bench_function("oak_css_parse", |b| {
            b.iter(|| {
                let mut session = ParseSession::<CssLanguage>::new(16);
                let out = parser.parse(&src, &[], &mut session);
                black_box(out);
            })
        });

        group.bench_function("cssparser_lex", |b| {
            b.iter(|| {
                let mut input = ParserInput::new(black_box(&s));
                let mut css_parser = CssParserLib::new(&mut input);
                while let Ok(token) = css_parser.next() {
                    black_box(token);
                }
            })
        });
        group.finish()
    }
}

criterion_group!(benches, bench_css_comparison);
criterion_main!(benches);
