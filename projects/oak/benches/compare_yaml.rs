#![feature(new_range_api)]

use criterion::{Criterion, criterion_group, criterion_main};
use oak_core::{Lexer, ParseSession, Parser, source::SourceText};
use oak_yaml::{language::YamlLanguage, lexer::YamlLexer, parser::YamlParser};
use std::hint::black_box;

fn generate_yaml(n: usize) -> String {
    let mut s = String::with_capacity(n * 100);
    for i in 0..n {
        s.push_str(&format!(
            "item-{}:\n  id: {}\n  name: \"Product {}\"\n  price: {}\n  tags:\n    - electronic\n    - sale\n    - v1\n  meta:\n    created: \"2024-01-01\"\n    enabled: {}\n\n",
            i,
            i,
            i,
            (i as f64) * 1.5,
            if i % 2 == 0 { "true" } else { "false" }
        ))
    }
    s
}

fn bench_yaml_comparison(c: &mut Criterion) {
    let lang = Box::leak(Box::new(YamlLanguage::default()));
    let lexer = YamlLexer::new(lang);
    let parser = YamlParser::new(lang);

    // 1. Small YAML
    {
        let mut group = c.benchmark_group("YAML_Small");
        let s = generate_yaml(5);
        let src = SourceText::new(s.as_str());

        group.bench_function("oak_yaml_lex", |b| {
            b.iter(|| {
                let mut session = ParseSession::<YamlLanguage>::new(16);
                let out = lexer.lex(black_box(&src), &[], &mut session);
                black_box(out);
            })
        });

        group.bench_function("oak_yaml_parse", |b| {
            b.iter(|| {
                let mut session = ParseSession::<YamlLanguage>::new(16);
                let out = parser.parse(&src, &[], &mut session);
                black_box(out);
            })
        });

        group.bench_function("serde_yaml_parse", |b| {
            b.iter(|| {
                let val: serde_yaml::Value = serde_yaml::from_str(black_box(&s)).unwrap();
                black_box(val);
            })
        });
        group.finish()
    }

    // 2. Medium YAML
    {
        let mut group = c.benchmark_group("YAML_Medium");
        let s = generate_yaml(50);
        let src = SourceText::new(s.as_str());

        group.bench_function("oak_yaml_lex", |b| {
            b.iter(|| {
                let mut session = ParseSession::<YamlLanguage>::new(16);
                let out = lexer.lex(black_box(&src), &[], &mut session);
                black_box(out);
            })
        });

        group.bench_function("oak_yaml_parse", |b| {
            b.iter(|| {
                let mut session = ParseSession::<YamlLanguage>::new(16);
                let out = parser.parse(&src, &[], &mut session);
                black_box(out);
            })
        });

        group.bench_function("serde_yaml_parse", |b| {
            b.iter(|| {
                let val: serde_yaml::Value = serde_yaml::from_str(black_box(&s)).unwrap();
                black_box(val);
            })
        });
        group.finish()
    }

    // 3. Large YAML
    {
        let mut group = c.benchmark_group("YAML_Large_500");
        let s = generate_yaml(500);
        let src = SourceText::new(s.as_str());

        group.bench_function("oak_yaml_lex", |b| {
            b.iter(|| {
                let mut session = ParseSession::<YamlLanguage>::new(16);
                let out = lexer.lex(black_box(&src), &[], &mut session);
                black_box(out);
            })
        });

        group.bench_function("oak_yaml_parse", |b| {
            b.iter(|| {
                let mut session = ParseSession::<YamlLanguage>::new(16);
                let out = parser.parse(&src, &[], &mut session);
                black_box(out);
            })
        });

        group.bench_function("serde_yaml_parse", |b| {
            b.iter(|| {
                let val: serde_yaml::Value = serde_yaml::from_str(black_box(&s)).unwrap();
                black_box(val);
            })
        });
        group.finish()
    }
}

criterion_group!(benches, bench_yaml_comparison);
criterion_main!(benches);
