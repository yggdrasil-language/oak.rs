use oak_core::{Lexer, RedNode, RedTree, SourceText, TokenType, parser::session::ParseSession, Parser};
use oak_valkyrie::{ValkyrieLanguage, ValkyrieLexer, ValkyrieParser, ValkyrieTokenType, lexer::ValkyrieKeywords, parser::element_type::ValkyrieElementType};

const MICRO_BODY: &str = r#"micro main() -> i32 {
<% match arch %>
<% case "wasm32" %>
return 23
<% else %>
return 0
<% end %>
}"#;

#[test]
fn micro_body_lexer_emits_template_tokens() {
    let language = ValkyrieLanguage { support_t_grammar: true, ..ValkyrieLanguage::default() };
    let lexer = ValkyrieLexer::new(&language);
    let text = SourceText::new(MICRO_BODY);
    let mut cache = oak_core::lexer::NoLexerCache;
    let output = lexer.lex(&text, &[], &mut cache);
    assert!(output.diagnostics.is_empty(), "{:?}", output.diagnostics);
    let tokens = output.result.expect("lex");
    let kinds = tokens.iter().map(|token| token.kind).collect::<Vec<_>>();
    assert!(kinds.contains(&ValkyrieTokenType::TemplateL), "missing TemplateL: {kinds:?}");
    assert!(kinds.contains(&ValkyrieTokenType::TemplateR), "missing TemplateR: {kinds:?}");
    // `match` 在指令内为 TGrammar 关键词 token，不得出现在首个 `TemplateL` 之前的编程词法段。
    let before_first_template = kinds
        .iter()
        .take_while(|kind| *kind != &ValkyrieTokenType::TemplateL)
        .cloned()
        .collect::<Vec<_>>();
    assert!(
        !before_first_template.contains(&ValkyrieTokenType::Keyword(ValkyrieKeywords::Match)),
        "programming lexer must not emit `match` before template region: {before_first_template:?}"
    );
}

#[test]
fn micro_body_cst_contains_template_match() {
    let language = ValkyrieLanguage { support_t_grammar: true, ..ValkyrieLanguage::default() };
    let parser = ValkyrieParser::new(&language);
    let text = SourceText::new(MICRO_BODY);
    let mut session = ParseSession::<ValkyrieLanguage>::default();
    let output = parser.parse(&text, &[], &mut session);
    assert!(output.diagnostics.is_empty(), "{:?}", output.diagnostics);
    let root = output.result.expect("parse");
    let red_root = RedNode::<ValkyrieLanguage>::new(root, 0);
    let mut kinds = Vec::new();
    walk(&red_root, &mut |kind| kinds.push(kind));
    assert!(
        kinds.contains(&ValkyrieElementType::TemplateMatch),
        "CST must contain `TemplateMatch` under micro body, got: {:?}",
        kinds
    );
}

fn walk(node: &RedNode<ValkyrieLanguage>, visit: &mut dyn FnMut(ValkyrieElementType)) {
    visit(node.green.kind);
    for child in node.children() {
        if let RedTree::Node(child) = child {
            walk(&child, visit);
        }
    }
}
