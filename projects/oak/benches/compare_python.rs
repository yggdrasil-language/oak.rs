#![feature(new_range_api)]

use criterion::{Criterion, criterion_group, criterion_main};
use oak_core::{Lexer, Parser, parser::ParseSession, source::SourceText};
use oak_python::{PythonLanguage, PythonLexer, PythonParser};
use std::hint::black_box;

fn generate_python(n: usize) -> String {
    let mut s = String::with_capacity(n * 150);
    s.push_str("def main():\n");
    for i in 0..n {
        s.push_str(&format!("    val_{} = {} * 2\n    if val_{} > 10:\n        print(f'large value: {{val_{}}}')\n    else:\n        print('small')\n", i, i, i, i))
    }
    s
}

fn bench_python_comparison(c: &mut Criterion) {
    let lang = Box::leak(Box::new(PythonLanguage::default()));
    let lexer = PythonLexer::new(lang);
    let parser = PythonParser::new(lang);

    // 1. Small Python
    {
        let mut group = c.benchmark_group("Python_Small");
        let s = generate_python(5);
        let src = SourceText::new(s.as_str());

        group.bench_function("oak_python_lex", |b| {
            b.iter(|| {
                let mut session = ParseSession::<PythonLanguage>::new(16);
                let out = lexer.lex(black_box(&src), &[], &mut session);
                black_box(out);
            })
        });

        group.bench_function("oak_python_parse", |b| {
            b.iter(|| {
                let mut session = ParseSession::<PythonLanguage>::new(16);
                let out = parser.parse(black_box(&src), &[], &mut session);
                black_box(out);
            })
        });

        group.bench_function("rustpython_parse", |b| {
            b.iter(|| {
                let ast = rustpython_parser::parse(black_box(&s), rustpython_parser::Mode::Module, "<string>").unwrap();
                black_box(ast);
            })
        });
        group.finish()
    }

    // 2. Medium Python
    {
        let mut group = c.benchmark_group("Python_Medium");
        let s = generate_python(50);
        let src = SourceText::new(s.as_str());

        group.bench_function("oak_python_lex", |b| {
            b.iter(|| {
                let mut session = ParseSession::<PythonLanguage>::new(16);
                let out = lexer.lex(black_box(&src), &[], &mut session);
                black_box(out);
            })
        });

        group.bench_function("oak_python_parse", |b| {
            b.iter(|| {
                let mut session = ParseSession::<PythonLanguage>::new(16);
                let out = parser.parse(black_box(&src), &[], &mut session);
                black_box(out);
            })
        });

        group.bench_function("rustpython_parse", |b| {
            b.iter(|| {
                let ast = rustpython_parser::parse(black_box(&s), rustpython_parser::Mode::Module, "<string>").unwrap();
                black_box(ast);
            })
        });
        group.finish()
    }

    // 3. Large Python
    {
        let mut group = c.benchmark_group("Python_Large_500");
        let s = generate_python(500);
        let src = SourceText::new(s.as_str());

        group.bench_function("oak_python_lex", |b| {
            b.iter(|| {
                let mut session = ParseSession::<PythonLanguage>::new(16);
                let out = lexer.lex(black_box(&src), &[], &mut session);
                black_box(out);
            })
        });

        group.bench_function("oak_python_parse", |b| {
            b.iter(|| {
                let mut session = ParseSession::<PythonLanguage>::new(16);
                let out = parser.parse(black_box(&src), &[], &mut session);
                black_box(out);
            })
        });

        group.bench_function("rustpython_parse", |b| {
            b.iter(|| {
                let ast = rustpython_parser::parse(black_box(&s), rustpython_parser::Mode::Module, "<string>").unwrap();
                black_box(ast);
            })
        });
        group.finish()
    }
}

criterion_group!(benches, bench_python_comparison);
criterion_main!(benches);
