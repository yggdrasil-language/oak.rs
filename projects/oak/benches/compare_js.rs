#![feature(new_range_api)]

use criterion::{Criterion, criterion_group, criterion_main};
use oak_core::{Lexer, ParseSession, Parser, source::SourceText};
use oak_javascript::{language::JavaScriptLanguage, lexer::JavaScriptLexer, parser::JavaScriptParser};
use std::hint::black_box;
// use oxc_allocator::Allocator;
// use oxc_parser::Parser as OxcParser;
// use oxc_span::SourceType;

fn generate_js(n: usize) -> String {
    let mut s = String::with_capacity(n * 150);
    for i in 0..n {
        s.push_str(&format!(
            "class User{} {{\n  constructor(name) {{\n    this.name = name;\n  }}\n  \
            getName() {{\n    return this.name;\n  }}\n}}\n\n\
            const user{} = new User{}('Alice');\nconsole.log(user{}.getName());\n\n",
            i, i, i, i
        ))
    }
    s
}

fn bench_js_comparison(c: &mut Criterion) {
    let lang = Box::leak(Box::new(JavaScriptLanguage::modern()));
    let lexer = JavaScriptLexer::new(lang);
    let parser = JavaScriptParser::new(lang);

    // 1. Small JS
    {
        let mut group = c.benchmark_group("JS_Small");
        let s = generate_js(5);
        let src = SourceText::new(s.as_str());

        group.bench_function("oak_js_lex", |b| {
            b.iter(|| {
                let mut session = ParseSession::<JavaScriptLanguage>::new(16);
                let out = lexer.lex(black_box(&src), &[], &mut session);
                black_box(out);
            })
        });

        group.bench_function("oak_js_parse", |b| {
            b.iter(|| {
                let mut session = ParseSession::<JavaScriptLanguage>::new(16);
                let out = parser.parse(&src, &[], &mut session);
                black_box(out);
            })
        });

        // group.bench_function("oxc_js_parse", |b| {
        // b.iter_batched(
        // || Allocator::default(),
        // |allocator| {
        // let source_type = SourceType::default();
        // let parser = OxcParser::new(&allocator, &s, source_type);
        // let program = parser.parse();
        // black_box(program);
        // },
        // criterion::BatchSize::SmallInput,
        // )
        // });
        group.finish()
    }

    // 2. Medium JS
    {
        let mut group = c.benchmark_group("JS_Medium");
        let s = generate_js(50);
        let src = SourceText::new(s.as_str());

        group.bench_function("oak_js_lex", |b| {
            b.iter(|| {
                let mut session = ParseSession::<JavaScriptLanguage>::new(16);
                let out = lexer.lex(black_box(&src), &[], &mut session);
                black_box(out);
            })
        });

        group.bench_function("oak_js_parse", |b| {
            b.iter(|| {
                let mut session = ParseSession::<JavaScriptLanguage>::new(16);
                let out = parser.parse(&src, &[], &mut session);
                black_box(out);
            })
        });

        // group.bench_function("oxc_js_parse", |b| {
        // b.iter_batched(
        // || Allocator::default(),
        // |allocator| {
        // let source_type = SourceType::default();
        // let parser = OxcParser::new(&allocator, &s, source_type);
        // let program = parser.parse();
        // black_box(program);
        // },
        // criterion::BatchSize::SmallInput,
        // )
        // });
        group.finish()
    }

    // 3. Large JS
    {
        let mut group = c.benchmark_group("JS_Large_500");
        let s = generate_js(500);
        let src = SourceText::new(s.as_str());

        group.bench_function("oak_js_lex", |b| {
            b.iter(|| {
                let mut session = ParseSession::<JavaScriptLanguage>::new(16);
                let out = lexer.lex(black_box(&src), &[], &mut session);
                black_box(out);
            })
        });

        group.bench_function("oak_js_parse", |b| {
            b.iter(|| {
                let mut session = ParseSession::<JavaScriptLanguage>::new(16);
                let out = parser.parse(&src, &[], &mut session);
                black_box(out);
            })
        });

        // group.bench_function("oxc_js_parse", |b| {
        // b.iter_batched(
        // || Allocator::default(),
        // |allocator| {
        // let source_type = SourceType::default();
        // let parser = OxcParser::new(&allocator, &s, source_type);
        // let program = parser.parse();
        // black_box(program);
        // },
        // criterion::BatchSize::LargeInput,
        // )
        // });
        group.finish()
    }
}

criterion_group!(benches, bench_js_comparison);
criterion_main!(benches);
