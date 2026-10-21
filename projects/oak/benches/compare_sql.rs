#![feature(new_range_api)]

use criterion::{Criterion, criterion_group, criterion_main};
use oak_core::{Lexer, ParseSession, Parser, source::SourceText};
use oak_sql::{language::SqlLanguage, lexer::SqlLexer, parser::SqlParser};
use std::hint::black_box;

fn generate_sql(n: usize) -> String {
    let mut s = String::with_capacity(n * 150);
    for i in 0..n {
        s.push_str(&format!("SELECT id, name, price FROM products WHERE category_id = {} AND status = 'active' ORDER BY price DESC LIMIT 10;\n", i));
        s.push_str(&format!("UPDATE inventory SET stock = stock - 1 WHERE product_id = {} AND stock > 0;\n", i))
    }
    s
}

fn bench_sql_comparison(c: &mut Criterion) {
    let lang = Box::leak(Box::new(SqlLanguage::standard()));
    let parser = SqlParser::new(lang);
    let lexer = SqlLexer::new(lang);

    // 1. Small SQL
    {
        let mut group = c.benchmark_group("SQL_Small");
        let s = generate_sql(5);
        let src = SourceText::new(s.as_str());

        group.bench_function("oak_sql_lex", |b| {
            b.iter(|| {
                let mut session = ParseSession::<SqlLanguage>::new(16);
                let out = lexer.lex(black_box(&src), &[], &mut session);
                black_box(out);
            })
        });

        group.bench_function("oak_sql_parse", |b| {
            b.iter(|| {
                let mut session = ParseSession::<SqlLanguage>::new(16);
                let out = parser.parse(&src, &[], &mut session);
                black_box(out);
            })
        });

        group.finish()
    }

    // 2. Medium SQL
    {
        let mut group = c.benchmark_group("SQL_Medium");
        let s = generate_sql(50);
        let src = SourceText::new(s.as_str());

        group.bench_function("oak_sql_lex", |b| {
            b.iter(|| {
                let mut session = ParseSession::<SqlLanguage>::new(16);
                let out = lexer.lex(black_box(&src), &[], &mut session);
                black_box(out);
            })
        });

        group.bench_function("oak_sql_parse", |b| {
            b.iter(|| {
                let mut session = ParseSession::<SqlLanguage>::new(16);
                let out = parser.parse(&src, &[], &mut session);
                black_box(out);
            })
        });

        group.finish()
    }

    // 3. Large SQL
    {
        let mut group = c.benchmark_group("SQL_Large_500");
        let s = generate_sql(500);
        let src = SourceText::new(s.as_str());

        group.bench_function("oak_sql_lex", |b| {
            b.iter(|| {
                let mut session = ParseSession::<SqlLanguage>::new(16);
                let out = lexer.lex(black_box(&src), &[], &mut session);
                black_box(out);
            })
        });

        group.bench_function("oak_sql_parse", |b| {
            b.iter(|| {
                let mut session = ParseSession::<SqlLanguage>::new(16);
                let out = parser.parse(&src, &[], &mut session);
                black_box(out);
            })
        });

        group.finish()
    }
}

criterion_group!(benches, bench_sql_comparison);
criterion_main!(benches);
