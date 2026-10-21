#![feature(new_range_api)]

use criterion::{Criterion, criterion_group, criterion_main};
use html5ever::tokenizer::{BufferQueue, Token, TokenSink, TokenSinkResult, Tokenizer, TokenizerOpts};
use oak_core::{Lexer, ParseSession, Parser, source::SourceText};
use oak_html::{HtmlLanguage, HtmlLexer, HtmlParser};
use std::hint::black_box;

fn generate_html(n: usize) -> String {
    let mut s = String::with_capacity(n * 200);
    s.push_str("<!DOCTYPE html><html><body>");
    for i in 0..n {
        s.push_str(&format!("<div id=\"item-{}\" class=\"container\">\n  <h1>Title {}</h1>\n  <p>Some description for item {}.</p>\n  <a href=\"/link/{}\">Click here</a>\n</div>\n", i, i, i, i))
    }
    s.push_str("</body></html>");
    s
}

struct Sink;
impl TokenSink for Sink {
    type Handle = ();
    fn process_token(&self, token: Token, _line_number: u64) -> TokenSinkResult<()> {
        black_box(token);
        TokenSinkResult::Continue
    }
}

fn bench_html_comparison(c: &mut Criterion) {
    let lang = Box::leak(Box::new(HtmlLanguage::default()));
    let lexer = HtmlLexer::new(lang);
    let parser = HtmlParser::new(lang);

    // 1. Small HTML
    {
        let mut group = c.benchmark_group("HTML_Small");
        let s = generate_html(5);
        let src = SourceText::new(s.as_str());

        group.bench_function("oak_html_lex", |b| {
            b.iter(|| {
                let mut session = ParseSession::<HtmlLanguage>::new(16);
                let out = lexer.lex(black_box(&src), &[], &mut session);
                black_box(out);
            })
        });

        group.bench_function("oak_html_parse", |b| {
            b.iter(|| {
                let mut session = ParseSession::<HtmlLanguage>::new(16);
                let out = parser.parse(&src, &[], &mut session);
                black_box(out);
            })
        });

        group.bench_function("html5ever_lex", |b| {
            b.iter(|| {
                let tokenizer = Tokenizer::new(Sink, TokenizerOpts::default());
                let mut buffer = BufferQueue::default();
                buffer.push_back(html5ever::tendril::StrTendril::from(black_box(&s as &str)));
                let _ = tokenizer.feed(&mut buffer);
                tokenizer.end();
            })
        });
        group.finish()
    }

    // 2. Medium HTML
    {
        let mut group = c.benchmark_group("HTML_Medium");
        let s = generate_html(50);
        let src = SourceText::new(s.as_str());

        group.bench_function("oak_html_lex", |b| {
            b.iter(|| {
                let mut session = ParseSession::<HtmlLanguage>::new(16);
                let out = lexer.lex(black_box(&src), &[], &mut session);
                black_box(out);
            })
        });

        group.bench_function("oak_html_parse", |b| {
            b.iter(|| {
                let mut session = ParseSession::<HtmlLanguage>::new(16);
                let out = parser.parse(&src, &[], &mut session);
                black_box(out);
            })
        });

        group.bench_function("html5ever_lex", |b| {
            b.iter(|| {
                let tokenizer = Tokenizer::new(Sink, TokenizerOpts::default());
                let mut buffer = BufferQueue::default();
                buffer.push_back(html5ever::tendril::StrTendril::from(black_box(&s as &str)));
                let _ = tokenizer.feed(&mut buffer);
                tokenizer.end();
            })
        });
        group.finish()
    }

    // 3. Large HTML
    {
        let mut group = c.benchmark_group("HTML_Large_500");
        let s = generate_html(500);
        let src = SourceText::new(s.as_str());

        group.bench_function("oak_html_lex", |b| {
            b.iter(|| {
                let mut session = ParseSession::<HtmlLanguage>::new(16);
                let out = lexer.lex(black_box(&src), &[], &mut session);
                black_box(out);
            })
        });

        group.bench_function("oak_html_parse", |b| {
            b.iter(|| {
                let mut session = ParseSession::<HtmlLanguage>::new(16);
                let out = parser.parse(&src, &[], &mut session);
                black_box(out);
            })
        });

        group.bench_function("html5ever_lex", |b| {
            b.iter(|| {
                let tokenizer = Tokenizer::new(Sink, TokenizerOpts::default());
                let mut buffer = BufferQueue::default();
                buffer.push_back(html5ever::tendril::StrTendril::from(black_box(&s as &str)));
                let _ = tokenizer.feed(&mut buffer);
                tokenizer.end();
            })
        });
        group.finish()
    }
}

criterion_group!(benches, bench_html_comparison);
criterion_main!(benches);
