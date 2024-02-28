#![feature(new_range_api)]

use criterion::{Criterion, criterion_group, criterion_main};
use oak_core::{Lexer, ParseSession, Parser, source::SourceText};
use oak_markdown::{language::MarkdownLanguage, lexer::MarkdownLexer, parser::MarkdownParser};
use pulldown_cmark::{Options, Parser as PulldownParser, html};
use std::hint::black_box;

fn generate_markdown(n: usize) -> String {
    let mut s = String::with_capacity(n * 200);
    for i in 0..n {
        s.push_str(&format!(
            "# Section {}\n\nThis is a paragraph with **bold** and *italic* text. \
            Here is some `inline code` and a [link](https://example.com).\n\n\
            - List item 1\n- List item 2\n- [ ] Task item\n\n\
            ```rust\nfn main() {{\n    println!(\"Hello, world!\");\n}}\n```\n\n",
            i
        ))
    }
    s
}

fn bench_markdown_comparison(c: &mut Criterion) {
    let lang = Box::leak(Box::new(MarkdownLanguage::github_flavored()));
    let lexer = MarkdownLexer::new(lang);
    let parser = MarkdownParser::new(lang);

    // 1. Small Markdown
    {
        let mut group = c.benchmark_group("Markdown_Small");
        let s = generate_markdown(5);
        let src = SourceText::new(s.as_str());

        group.bench_function("oak_markdown_lex", |b| {
            b.iter(|| {
                let mut session = ParseSession::<MarkdownLanguage>::new(16);
                let out = lexer.lex(black_box(&src), &[], &mut session);
                black_box(out);
            })
        });

        group.bench_function("oak_markdown_parse", |b| {
            b.iter(|| {
                let mut session = ParseSession::<MarkdownLanguage>::new(16);
                let out = parser.parse(&src, &[], &mut session);
                black_box(out);
            })
        });

        group.bench_function("markdown_rs_to_html", |b| {
            b.iter(|| {
                let out = markdown::to_html(black_box(&s));
                black_box(out)
            })
        });

        group.bench_function("pulldown_cmark_to_html", |b| {
            b.iter(|| {
                let mut options = Options::empty();
                options.insert(Options::ENABLE_STRIKETHROUGH);
                let parser = PulldownParser::new_ext(black_box(&s), options);
                let mut html_output = String::new();
                html::push_html(&mut html_output, parser);
                black_box(html_output)
            })
        });
        group.finish()
    }

    // 2. Medium Markdown
    {
        let mut group = c.benchmark_group("Markdown_Medium");
        let s = generate_markdown(50);
        let src = SourceText::new(s.as_str());

        group.bench_function("oak_markdown_lex", |b| {
            b.iter(|| {
                let mut session = ParseSession::<MarkdownLanguage>::new(16);
                let out = lexer.lex(black_box(&src), &[], &mut session);
                black_box(out);
            })
        });

        group.bench_function("oak_markdown_parse", |b| {
            b.iter(|| {
                let mut session = ParseSession::<MarkdownLanguage>::new(16);
                let out = parser.parse(&src, &[], &mut session);
                black_box(out);
            })
        });

        group.bench_function("markdown_rs_to_html", |b| {
            b.iter(|| {
                let out = markdown::to_html(black_box(&s));
                black_box(out)
            })
        });

        group.bench_function("pulldown_cmark_to_html", |b| {
            b.iter(|| {
                let mut options = Options::empty();
                options.insert(Options::ENABLE_STRIKETHROUGH);
                let parser = PulldownParser::new_ext(black_box(&s), options);
                let mut html_output = String::new();
                html::push_html(&mut html_output, parser);
                black_box(html_output)
            })
        });
        group.finish()
    }

    // 3. Large Markdown
    {
        let mut group = c.benchmark_group("Markdown_Large_200");
        let s = generate_markdown(200);
        let src = SourceText::new(s.as_str());

        group.bench_function("oak_markdown_lex", |b| {
            b.iter(|| {
                let mut session = ParseSession::<MarkdownLanguage>::new(16);
                let out = lexer.lex(black_box(&src), &[], &mut session);
                black_box(out);
            })
        });

        group.bench_function("oak_markdown_parse", |b| {
            b.iter(|| {
                let mut session = ParseSession::<MarkdownLanguage>::new(16);
                let out = parser.parse(&src, &[], &mut session);
                black_box(out);
            })
        });

        group.bench_function("markdown_rs_to_html", |b| {
            b.iter(|| {
                let out = markdown::to_html(black_box(&s));
                black_box(out)
            })
        });

        group.bench_function("pulldown_cmark_to_html", |b| {
            b.iter(|| {
                let mut options = Options::empty();
                options.insert(Options::ENABLE_STRIKETHROUGH);
                let parser = PulldownParser::new_ext(black_box(&s), options);
                let mut html_output = String::new();
                html::push_html(&mut html_output, parser);
                black_box(html_output)
            })
        });
        group.finish()
    }
}

criterion_group!(benches, bench_markdown_comparison);
criterion_main!(benches);
