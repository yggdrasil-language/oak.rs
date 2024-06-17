use crate::{ast::*, language::TexLanguage, lexer::token_type::TexTokenType, parser::TexParser};
use oak_core::{Builder, BuilderCache, GreenNode, OakDiagnostics, OakError, Parser, RedNode, RedTree, TextEdit, source::Source};

/// Names of TeX environments that represent math mode (AMS + classic).
const TEX_MATH_ENVIRONMENTS: &[&str] =
    &["equation", "align", "gather", "multline", "eqnarray", "flalign", "alignat", "aligned", "alignedat", "gathered", "split", "cases", "matrix", "pmatrix", "bmatrix", "Bmatrix", "vmatrix", "Vmatrix", "smallmatrix", "math", "displaymath"];

/// An AST builder for the TeX language.
#[derive(Clone)]
pub struct TexBuilder<'config> {
    /// The language configuration.
    config: &'config TexLanguage,
}

impl<'config> TexBuilder<'config> {
    /// Creates a new TeX builder.
    pub fn new(config: &'config TexLanguage) -> Self {
        Self { config }
    }
}

impl<'config> Builder<TexLanguage> for TexBuilder<'config> {
    fn build<'a, S: Source + ?Sized>(&self, source: &'a S, edits: &[TextEdit], cache: &'a mut impl BuilderCache<TexLanguage>) -> oak_core::builder::BuildOutput<TexLanguage> {
        let parser = TexParser::new(self.config);

        let parse_result = parser.parse(source, edits, cache);

        match parse_result.result {
            Ok(green_tree) => match self.build_root_internal(green_tree, source) {
                Ok(ast_root) => OakDiagnostics { result: Ok(ast_root), diagnostics: parse_result.diagnostics },
                Err(build_error) => {
                    let mut diagnostics = parse_result.diagnostics;
                    diagnostics.push(build_error.clone());
                    OakDiagnostics { result: Err(build_error), diagnostics }
                }
            },
            Err(parse_error) => OakDiagnostics { result: Err(parse_error), diagnostics: parse_result.diagnostics },
        }
    }
}

impl<'config> TexBuilder<'config> {
    /// Builds the root node of the TeX AST.
    fn build_root_internal<S: Source + ?Sized>(&self, green_tree: &GreenNode<TexLanguage>, source: &S) -> Result<TexRoot, OakError> {
        let red_root = RedNode::new(green_tree, 0);
        self.build_content(red_root, source)
    }

    /// Builds the content of a node into a `TexRoot`.
    fn build_content<S: Source + ?Sized>(&self, node: RedNode<TexLanguage>, source: &S) -> Result<TexRoot, OakError> {
        let mut items = Vec::new();
        let children: Vec<_> = node.children().collect();

        if children.is_empty() {
            return Ok(TexRoot { span: node.span().into(), items });
        }

        // Skip leading/trailing delimiter leaf tokens — but NOT CST sub-nodes
        // that happen to share the same kind (e.g. a `ParenInlineMath` CST node
        // whose kind maps to `TexTokenType::ParenInlineMathDelim`).
        let first = children.first().unwrap();
        let last = children.last().unwrap();
        let start = if first.as_token().is_some() && is_delimiter(first.kind::<TexTokenType>()) { 1 } else { 0 };
        let end = if children.len() > start && last.as_token().is_some() && is_delimiter(last.kind::<TexTokenType>()) { children.len() - 1 } else { children.len() };

        for i in start..end {
            if let Some(item) = self.build_item(children[i], source)? {
                items.push(item)
            }
        }

        Ok(TexRoot { span: node.span().into(), items })
    }

    fn build_item<S: Source + ?Sized>(&self, tree: RedTree<TexLanguage>, source: &S) -> Result<Option<TexItem>, OakError> {
        let kind: TexTokenType = tree.kind();
        match kind {
            TexTokenType::Command
            | TexTokenType::BeginKeyword
            | TexTokenType::EndKeyword
            | TexTokenType::Frac
            | TexTokenType::Sqrt
            | TexTokenType::Sum
            | TexTokenType::Int
            | TexTokenType::Lim
            | TexTokenType::SectionKeyword
            | TexTokenType::SubsectionKeyword
            | TexTokenType::SubsubsectionKeyword
            | TexTokenType::ChapterKeyword
            | TexTokenType::PartKeyword
            | TexTokenType::TitleKeyword
            | TexTokenType::AuthorKeyword
            | TexTokenType::DateKeyword
            | TexTokenType::MaketitleKeyword
            | TexTokenType::TableofcontentsKeyword
            | TexTokenType::ItemKeyword
            | TexTokenType::LabelKeyword
            | TexTokenType::RefKeyword
            | TexTokenType::CiteKeyword
            | TexTokenType::IncludegraphicsKeyword
            | TexTokenType::TextbfKeyword
            | TexTokenType::TextitKeyword
            | TexTokenType::EmphKeyword
            | TexTokenType::Alpha
            | TexTokenType::Beta
            | TexTokenType::Gamma
            | TexTokenType::Delta
            | TexTokenType::Epsilon
            | TexTokenType::Zeta
            | TexTokenType::Eta
            | TexTokenType::Theta
            | TexTokenType::Iota
            | TexTokenType::Kappa
            | TexTokenType::Lambda
            | TexTokenType::Mu
            | TexTokenType::Nu
            | TexTokenType::Xi
            | TexTokenType::Omicron
            | TexTokenType::Pi
            | TexTokenType::Rho
            | TexTokenType::Sigma
            | TexTokenType::Tau
            | TexTokenType::Upsilon
            | TexTokenType::Phi
            | TexTokenType::Chi
            | TexTokenType::Psi
            | TexTokenType::Omega
            | TexTokenType::VarEpsilon
            | TexTokenType::VarTheta
            | TexTokenType::VarKappa
            | TexTokenType::VarPi
            | TexTokenType::VarRho
            | TexTokenType::VarSigma
            | TexTokenType::VarPhi
            | TexTokenType::UpperGamma
            | TexTokenType::UpperDelta
            | TexTokenType::UpperTheta
            | TexTokenType::UpperLambda
            | TexTokenType::UpperXi
            | TexTokenType::UpperPi
            | TexTokenType::UpperSigma
            | TexTokenType::UpperUpsilon
            | TexTokenType::UpperPhi
            | TexTokenType::UpperPsi
            | TexTokenType::UpperOmega => {
                if let Some(node) = tree.as_node() {
                    Ok(Some(TexItem::Command(self.build_command(node, source)?)))
                }
                else {
                    let name = match kind {
                        TexTokenType::Sum => "sum".to_string(),
                        TexTokenType::Int => "int".to_string(),
                        TexTokenType::Lim => "lim".to_string(),
                        TexTokenType::Frac => "frac".to_string(),
                        TexTokenType::Sqrt => "sqrt".to_string(),
                        TexTokenType::Alpha => "alpha".to_string(),
                        TexTokenType::Beta => "beta".to_string(),
                        TexTokenType::Gamma => "gamma".to_string(),
                        TexTokenType::Delta => "delta".to_string(),
                        TexTokenType::Epsilon => "epsilon".to_string(),
                        TexTokenType::Zeta => "zeta".to_string(),
                        TexTokenType::Eta => "eta".to_string(),
                        TexTokenType::Theta => "theta".to_string(),
                        TexTokenType::Iota => "iota".to_string(),
                        TexTokenType::Kappa => "kappa".to_string(),
                        TexTokenType::Lambda => "lambda".to_string(),
                        TexTokenType::Mu => "mu".to_string(),
                        TexTokenType::Nu => "nu".to_string(),
                        TexTokenType::Xi => "xi".to_string(),
                        TexTokenType::Omicron => "omicron".to_string(),
                        TexTokenType::Pi => "pi".to_string(),
                        TexTokenType::Rho => "rho".to_string(),
                        TexTokenType::Sigma => "sigma".to_string(),
                        TexTokenType::Tau => "tau".to_string(),
                        TexTokenType::Upsilon => "upsilon".to_string(),
                        TexTokenType::Phi => "phi".to_string(),
                        TexTokenType::Chi => "chi".to_string(),
                        TexTokenType::Psi => "psi".to_string(),
                        TexTokenType::Omega => "omega".to_string(),
                        TexTokenType::VarEpsilon => "varepsilon".to_string(),
                        TexTokenType::VarTheta => "vartheta".to_string(),
                        TexTokenType::VarKappa => "varkappa".to_string(),
                        TexTokenType::VarPi => "varpi".to_string(),
                        TexTokenType::VarRho => "varrho".to_string(),
                        TexTokenType::VarSigma => "varsigma".to_string(),
                        TexTokenType::VarPhi => "varphi".to_string(),
                        TexTokenType::UpperGamma => "Gamma".to_string(),
                        TexTokenType::UpperDelta => "Delta".to_string(),
                        TexTokenType::UpperTheta => "Theta".to_string(),
                        TexTokenType::UpperLambda => "Lambda".to_string(),
                        TexTokenType::UpperXi => "Xi".to_string(),
                        TexTokenType::UpperPi => "Pi".to_string(),
                        TexTokenType::UpperSigma => "Sigma".to_string(),
                        TexTokenType::UpperUpsilon => "Upsilon".to_string(),
                        TexTokenType::UpperPhi => "Phi".to_string(),
                        TexTokenType::UpperPsi => "Psi".to_string(),
                        TexTokenType::UpperOmega => "Omega".to_string(),
                        _ => tree.text(source).trim_start_matches('\\').to_string(),
                    };
                    Ok(Some(TexItem::Command(TexCommand { span: tree.span().into(), name, starred: false, arguments: Vec::new() })))
                }
            }
            TexTokenType::Environment => Ok(Some(TexItem::Environment(self.build_environment(tree.as_node().unwrap(), source)?))),
            TexTokenType::Group => Ok(Some(TexItem::Group(self.build_group(tree.as_node().unwrap(), source)?))),
            TexTokenType::InlineMath | TexTokenType::DisplayMath | TexTokenType::ParenInlineMathDelim | TexTokenType::BracketDisplayMathDelim => Ok(Some(TexItem::Math(self.build_math(tree.as_node().unwrap(), source)?))),
            TexTokenType::LeftRightPair => Ok(Some(TexItem::LeftRightPair(self.build_left_right(tree.as_node().unwrap(), source)?))),
            TexTokenType::Limits | TexTokenType::Nolimits => Ok(Some(TexItem::Limits(self.build_limits(tree.as_node().unwrap(), source)?))),
            TexTokenType::Superscript => Ok(Some(TexItem::Superscript(self.build_superscript(tree.as_node().unwrap(), source)?))),
            TexTokenType::Subscript => Ok(Some(TexItem::Subscript(self.build_subscript(tree.as_node().unwrap(), source)?))),
            TexTokenType::Identifier | TexTokenType::Number | TexTokenType::Text => Ok(Some(TexItem::Text { span: tree.span().into(), content: tree.text(source).to_string() })),
            TexTokenType::Comment => Ok(Some(TexItem::Comment { span: tree.span().into(), content: tree.text(source).to_string() })),
            _ => {
                match kind {
                    // Skip synthetic or trivia tokens that carry no semantic value.
                    TexTokenType::Eof | TexTokenType::Whitespace | TexTokenType::Newline => Ok(None),
                    _ => {
                        if tree.as_token().is_some() {
                            Ok(Some(TexItem::Text { span: tree.span().into(), content: tree.text(source).to_string() }))
                        }
                        else {
                            Ok(None)
                        }
                    }
                }
            }
        }
    }

    fn build_superscript<S: Source + ?Sized>(&self, node: RedNode<TexLanguage>, source: &S) -> Result<TexSuperscript, OakError> {
        let mut content = TexRoot::new(node.span().into());
        for child in node.children() {
            let kind: TexTokenType = child.kind();
            if kind == TexTokenType::Caret {
                continue;
            }
            if let Some(item) = self.build_item(child, source)? {
                content.items.push(item)
            }
        }
        Ok(TexSuperscript {
            span: node.span().into(),
            target: None, // Will be filled later by a post-processor if needed
            content: Box::new(content),
        })
    }

    fn build_subscript<S: Source + ?Sized>(&self, node: RedNode<TexLanguage>, source: &S) -> Result<TexSubscript, OakError> {
        let mut content = TexRoot::new(node.span().into());
        for child in node.children() {
            let kind: TexTokenType = child.kind();
            if kind == TexTokenType::Underscore {
                continue;
            }
            if let Some(item) = self.build_item(child, source)? {
                content.items.push(item)
            }
        }
        Ok(TexSubscript {
            span: node.span().into(),
            target: None, // Will be filled later by a post-processor if needed
            content: Box::new(content),
        })
    }

    fn build_command<S: Source + ?Sized>(&self, node: RedNode<TexLanguage>, source: &S) -> Result<TexCommand, OakError> {
        let mut name = String::new();
        let mut starred = false;
        let mut arguments = Vec::new();

        for child in node.children() {
            let kind: TexTokenType = child.kind();
            match kind {
                TexTokenType::Star => {
                    starred = true;
                }
                TexTokenType::Backslash
                | TexTokenType::Command
                | TexTokenType::BeginKeyword
                | TexTokenType::EndKeyword
                | TexTokenType::Frac
                | TexTokenType::Sqrt
                | TexTokenType::Sum
                | TexTokenType::Int
                | TexTokenType::Lim
                | TexTokenType::SectionKeyword
                | TexTokenType::SubsectionKeyword
                | TexTokenType::SubsubsectionKeyword
                | TexTokenType::ChapterKeyword
                | TexTokenType::PartKeyword
                | TexTokenType::TitleKeyword
                | TexTokenType::AuthorKeyword
                | TexTokenType::DateKeyword
                | TexTokenType::MaketitleKeyword
                | TexTokenType::TableofcontentsKeyword
                | TexTokenType::ItemKeyword
                | TexTokenType::LabelKeyword
                | TexTokenType::RefKeyword
                | TexTokenType::CiteKeyword
                | TexTokenType::IncludegraphicsKeyword
                | TexTokenType::TextbfKeyword
                | TexTokenType::TextitKeyword
                | TexTokenType::EmphKeyword
                | TexTokenType::Alpha
                | TexTokenType::Beta
                | TexTokenType::Gamma
                | TexTokenType::Delta
                | TexTokenType::Epsilon
                | TexTokenType::Zeta
                | TexTokenType::Eta
                | TexTokenType::Theta
                | TexTokenType::Iota
                | TexTokenType::Kappa
                | TexTokenType::Lambda
                | TexTokenType::Mu
                | TexTokenType::Nu
                | TexTokenType::Xi
                | TexTokenType::Omicron
                | TexTokenType::Pi
                | TexTokenType::Rho
                | TexTokenType::Sigma
                | TexTokenType::Tau
                | TexTokenType::Upsilon
                | TexTokenType::Phi
                | TexTokenType::Chi
                | TexTokenType::Psi
                | TexTokenType::Omega
                | TexTokenType::VarEpsilon
                | TexTokenType::VarTheta
                | TexTokenType::VarKappa
                | TexTokenType::VarPi
                | TexTokenType::VarRho
                | TexTokenType::VarSigma
                | TexTokenType::VarPhi
                | TexTokenType::UpperGamma
                | TexTokenType::UpperDelta
                | TexTokenType::UpperTheta
                | TexTokenType::UpperLambda
                | TexTokenType::UpperXi
                | TexTokenType::UpperPi
                | TexTokenType::UpperSigma
                | TexTokenType::UpperUpsilon
                | TexTokenType::UpperPhi
                | TexTokenType::UpperPsi
                | TexTokenType::UpperOmega
                | TexTokenType::TextBf
                | TexTokenType::TextIt
                | TexTokenType::TextSc
                | TexTokenType::TextTt
                | TexTokenType::Emph
                | TexTokenType::Underline
                | TexTokenType::LeftCommand
                | TexTokenType::RightCommand
                | TexTokenType::MiddleCommand
                | TexTokenType::Limits
                | TexTokenType::Nolimits => {
                    let text = child.text(source);
                    if text.starts_with('\\') { name = text[1..].to_string() } else { name = text.to_string() }
                }
                TexTokenType::OptionalArgument => arguments.push(TexArgument::Optional(self.build_content(child.as_node().unwrap(), source)?)),
                TexTokenType::MandatoryArgument => arguments.push(TexArgument::Required(self.build_content(child.as_node().unwrap(), source)?)),
                _ => {}
            }
        }

        Ok(TexCommand { span: node.span().into(), name, starred, arguments })
    }

    fn build_group<S: Source + ?Sized>(&self, node: RedNode<TexLanguage>, source: &S) -> Result<TexGroup, OakError> {
        Ok(TexGroup { span: node.span().into(), content: self.build_content(node, source)? })
    }

    fn build_math<S: Source + ?Sized>(&self, node: RedNode<TexLanguage>, source: &S) -> Result<TexMath, OakError> {
        let delimiter = match node.kind::<TexTokenType>() {
            TexTokenType::DisplayMath | TexTokenType::DoubleDollar => crate::ast::MathDelimiter::DoubleDollar,
            TexTokenType::InlineMath | TexTokenType::Dollar => crate::ast::MathDelimiter::Dollar,
            TexTokenType::ParenInlineMathDelim => crate::ast::MathDelimiter::Paren,
            TexTokenType::BracketDisplayMathDelim => crate::ast::MathDelimiter::Bracket,
            _ => crate::ast::MathDelimiter::Environment,
        };
        Ok(TexMath { span: node.span().into(), content: self.build_content(node, source)?, is_display: delimiter.is_display(), delimiter })
    }

    fn build_left_right<S: Source + ?Sized>(&self, node: RedNode<TexLanguage>, source: &S) -> Result<TexLeftRightPair, OakError> {
        let mut left_delim = String::new();
        let mut right_delim = String::new();
        let mut content = TexRoot::new(node.span().into());
        let mut stage: u8 = 0; // 0: left delim, 1: content, 2: right delim

        for child in node.children() {
            let kind: TexTokenType = child.kind();
            match kind {
                TexTokenType::LeftCommand => {
                    stage = 0;
                }
                TexTokenType::RightCommand => {
                    stage = 2;
                }
                TexTokenType::MiddleCommand => {
                    // \middle is still part of the content semantically;
                    // treat as a text token carrying the command name.
                    content.items.push(TexItem::Command(TexCommand { span: child.span().into(), name: "middle".to_string(), starred: false, arguments: Vec::new() }));
                    stage = 1;
                }
                _ => {
                    let text = child.text(source).to_string();
                    if stage == 0 && left_delim.is_empty() {
                        left_delim = text;
                        stage = 1;
                    }
                    else if stage == 2 && right_delim.is_empty() {
                        right_delim = text;
                        stage = 2; // stay for anything unexpected
                    }
                    else if let Some(item) = self.build_item(child, source)? {
                        content.items.push(item);
                    }
                }
            }
        }

        Ok(TexLeftRightPair { span: node.span().into(), left_delim, right_delim, content })
    }

    fn build_limits<S: Source + ?Sized>(&self, node: RedNode<TexLanguage>, _source: &S) -> Result<TexLimits, OakError> {
        let is_limits = node.kind::<TexTokenType>() == TexTokenType::Limits;
        Ok(TexLimits { span: node.span().into(), is_limits })
    }

    fn build_environment<S: Source + ?Sized>(&self, node: RedNode<TexLanguage>, source: &S) -> Result<TexEnvironment, OakError> {
        let mut name = String::new();
        let mut arguments = Vec::new();
        let mut content = TexRoot::new(node.span().into());

        for child in node.children() {
            let kind: TexTokenType = child.kind();
            match kind {
                TexTokenType::BeginEnvironment => {
                    for sub_child in child.as_node().unwrap().children() {
                        let sub_kind: TexTokenType = sub_child.kind();
                        match sub_kind {
                            TexTokenType::MandatoryArgument => {
                                // The first mandatory argument is the environment name
                                if name.is_empty() {
                                    let arg_root = self.build_content(sub_child.as_node().unwrap(), source)?;
                                    for item in arg_root.items {
                                        if let TexItem::Text { content, .. } = item {
                                            name.push_str(&content)
                                        }
                                    }
                                }
                                else {
                                    arguments.push(TexArgument::Required(self.build_content(sub_child.as_node().unwrap(), source)?))
                                }
                            }
                            TexTokenType::OptionalArgument => arguments.push(TexArgument::Optional(self.build_content(sub_child.as_node().unwrap(), source)?)),
                            _ => {}
                        }
                    }
                }
                TexTokenType::EndEnvironment => {}
                _ => {
                    if let Some(item) = self.build_item(child, source)? {
                        content.items.push(item)
                    }
                }
            }
        }

        let starred = name.ends_with('*');
        if starred {
            name.truncate(name.len() - 1);
        }

        let is_math = TEX_MATH_ENVIRONMENTS.contains(&name.as_str());

        Ok(TexEnvironment { span: node.span().into(), name, starred, is_math, arguments, content })
    }
}

fn is_delimiter(kind: TexTokenType) -> bool {
    matches!(kind, TexTokenType::LeftBrace | TexTokenType::RightBrace | TexTokenType::LeftBracket | TexTokenType::RightBracket | TexTokenType::Dollar | TexTokenType::DoubleDollar | TexTokenType::ParenInlineMathDelim | TexTokenType::BracketDisplayMathDelim)
}
