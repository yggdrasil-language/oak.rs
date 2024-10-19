use oak_core::{Lexer, parser::session::ParseSession};
use oak_plantuml::{PlantUmlLanguage, PlantUmlLexer, PlantUmlTokenType};

fn lex(input: &str) -> Vec<(String, PlantUmlTokenType)> {
    let language = PlantUmlLanguage::new();
    let lexer = PlantUmlLexer::new(&language);
    let mut cache = ParseSession::<PlantUmlLanguage>::default();
    let output = lexer.lex(input, &[], &mut cache);
    assert!(output.diagnostics.is_empty(), "unexpected lexer diagnostics: {:?}", output.diagnostics);

    output
        .result
        .expect("lexing should succeed")
        .iter()
        .filter(|token| !matches!(token.kind, PlantUmlTokenType::Whitespace | PlantUmlTokenType::Newline | PlantUmlTokenType::Eof))
        .map(|token| (input[token.span.start..token.span.end].to_string(), token.kind))
        .collect()
}

#[test]
fn lexes_declaration_keywords() {
    let tokens = lex("class Foo\ninterface Bar\nabstract class Baz\nenum E\nparticipant P\nactor A\nactivate P\ndeactivate P\n");

    let kinds: Vec<_> = tokens.iter().map(|(_, kind)| *kind).collect();
    assert_eq!(
        kinds,
        vec![
            PlantUmlTokenType::Class,
            PlantUmlTokenType::Identifier,
            PlantUmlTokenType::Interface,
            PlantUmlTokenType::Identifier,
            PlantUmlTokenType::Abstract,
            PlantUmlTokenType::Class,
            PlantUmlTokenType::Identifier,
            PlantUmlTokenType::Enum,
            PlantUmlTokenType::Identifier,
            PlantUmlTokenType::Participant,
            PlantUmlTokenType::Identifier,
            PlantUmlTokenType::Actor,
            PlantUmlTokenType::Identifier,
            PlantUmlTokenType::Activate,
            PlantUmlTokenType::Identifier,
            PlantUmlTokenType::Deactivate,
            PlantUmlTokenType::Identifier,
        ]
    );
}

#[test]
fn lexes_all_arrows_as_single_tokens() {
    let tokens = lex("A --> B\nA <-- B\nA <|-- B\nA --|> B\nA *-- B\nA --* B\nA o-- B\nA --o B\nA ..> B\nA <.. B\nA .. B\nA ->> B\nA -->> B\n");

    let arrows: Vec<_> = tokens.iter().filter(|(_, kind)| *kind == PlantUmlTokenType::Arrow).map(|(text, _)| text.as_str()).collect();

    assert_eq!(arrows, vec!["-->", "<--", "<|--", "--|>", "*--", "--*", "o--", "--o", "..>", "<..", "..", "->>", "-->>",]);
}

#[test]
fn lexes_activity_state_and_component_statement_tokens() {
    let tokens = lex("(*) --> \"First\"\n:Do;\nif (x) then\n|Lane|\n[*] --> Idle\n[Web] --> [API]\n");

    let pairs: Vec<_> = tokens.iter().map(|(text, kind)| (text.as_str(), *kind)).collect();
    assert_eq!(
        pairs,
        vec![
            ("(", PlantUmlTokenType::LeftParen),
            ("*", PlantUmlTokenType::Text),
            (")", PlantUmlTokenType::RightParen),
            ("-->", PlantUmlTokenType::Arrow),
            ("\"First\"", PlantUmlTokenType::String),
            (":", PlantUmlTokenType::Colon),
            ("Do", PlantUmlTokenType::Identifier),
            (";", PlantUmlTokenType::Text),
            ("if", PlantUmlTokenType::Identifier),
            ("(", PlantUmlTokenType::LeftParen),
            ("x", PlantUmlTokenType::Identifier),
            (")", PlantUmlTokenType::RightParen),
            ("then", PlantUmlTokenType::Identifier),
            ("|Lane|", PlantUmlTokenType::Text),
            ("[*]", PlantUmlTokenType::Text),
            ("-->", PlantUmlTokenType::Arrow),
            ("Idle", PlantUmlTokenType::Identifier),
            ("[Web]", PlantUmlTokenType::Text),
            ("-->", PlantUmlTokenType::Arrow),
            ("[API]", PlantUmlTokenType::Text),
        ]
    );
}

#[test]
fn lexes_line_comments_and_strings() {
    let tokens = lex("' a note\nclass \"Display Name\" as Alias\n");

    assert_eq!(tokens[0], ("' a note".to_string(), PlantUmlTokenType::Comment));
    assert_eq!(tokens[1], ("class".to_string(), PlantUmlTokenType::Class));
    assert_eq!(tokens[2], ("\"Display Name\"".to_string(), PlantUmlTokenType::String));
    assert_eq!(tokens[3], ("as".to_string(), PlantUmlTokenType::As));
    assert_eq!(tokens[4], ("Alias".to_string(), PlantUmlTokenType::Identifier));
}
