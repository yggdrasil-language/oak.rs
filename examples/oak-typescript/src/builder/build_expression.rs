use crate::{ast::*, builder::TypeScriptBuilder, language::TypeScriptLanguage, lexer::token_type::TypeScriptTokenType, parser::element_type::TypeScriptElementType};
use oak_core::{OakError, RedNode, RedTree, Source, SourceText, TokenType};

impl<'config> TypeScriptBuilder<'config> {
    pub(crate) fn build_expression(&self, node: &RedNode<TypeScriptLanguage>, source: &SourceText) -> Result<Option<Expression>, OakError> {
        let kind = node.green.kind;
        let span = node.span();

        match kind {
            TypeScriptElementType::IdentifierName | TypeScriptElementType::This | TypeScriptElementType::Super => {
                let mut name = String::new();
                for child in node.children() {
                    if let RedTree::Leaf(leaf) = child {
                        if matches!(leaf.kind, TypeScriptTokenType::IdentifierName | TypeScriptTokenType::This | TypeScriptTokenType::Super) {
                            name = source.get_text_in(leaf.span.into()).trim().to_string();
                            break;
                        }
                    }
                }
                if name.is_empty() {
                    name = source.get_text_in(span.into()).trim().to_string();
                }
                Ok(Some(Expression::new(ExpressionKind::Identifier(name), span.into())))
            }
            TypeScriptElementType::NumericLiteral => {
                let text = source.get_text_in(span.into());
                let trimmed = text.trim();
                if let Ok(n) = trimmed.parse::<f64>() {
                    Ok(Some(Expression::new(ExpressionKind::NumericLiteral(n), span.into())))
                }
                else {
                    // Try children if parsing failed (sometimes nested)
                    for child in node.children() {
                        if let RedTree::Node(child_node) = child {
                            if let Some(expr) = self.build_expression(&child_node, source)? {
                                return Ok(Some(expr));
                            }
                        }
                    }
                    Ok(None)
                }
            }
            TypeScriptElementType::StringLiteral => {
                let content = decode_string_literal_text(&source.get_text_in(span.into()));
                Ok(Some(Expression::new(ExpressionKind::StringLiteral(content), span.into())))
            }
            TypeScriptElementType::BigIntLiteral => {
                let text = source.get_text_in(span.into());
                Ok(Some(Expression::new(ExpressionKind::BigIntLiteral(text.to_string()), span.into())))
            }
            TypeScriptElementType::BooleanLiteral | TypeScriptElementType::True | TypeScriptElementType::False => {
                let val = match kind {
                    TypeScriptElementType::True => true,
                    TypeScriptElementType::False => false,
                    _ => source.get_text_in(span.into()).trim() == "true",
                };
                Ok(Some(Expression::new(ExpressionKind::BooleanLiteral(val), span.into())))
            }
            TypeScriptElementType::Null => Ok(Some(Expression::new(ExpressionKind::NullLiteral, span.into()))),
            TypeScriptElementType::JsxExpressionContainer => {
                let mut left = None;
                let mut operator = String::new();
                let mut right = None;

                for child in node.children() {
                    match child {
                        RedTree::Node(child_node) => {
                            if left.is_none() {
                                left = self.build_expression(&child_node, source)?
                            }
                            else {
                                right = self.build_expression(&child_node, source)?
                            }
                        }
                        RedTree::Leaf(leaf) => {
                            if left.is_some() && right.is_none() && !leaf.kind.is_ignored() {
                                operator = source.get_text_in(leaf.span.into()).to_string()
                            }
                        }
                    }
                }

                if let (Some(l), Some(r)) = (left, right) {
                    let is_assignment = matches!(operator.as_str(), "=" | "+=" | "-=" | "*=" | "/=" | "%=" | "**=" | "<<=" | ">>=" | ">>>=" | "&=" | "|=" | "^=" | "&&=" | "||=" | "??=");

                    if is_assignment {
                        Ok(Some(Expression::new(ExpressionKind::AssignmentExpression { left: Box::new(l), operator, right: Box::new(r) }, span.into())))
                    }
                    else {
                        Ok(Some(Expression::new(ExpressionKind::BinaryExpression { left: Box::new(l), operator, right: Box::new(r) }, span.into())))
                    }
                }
                else {
                    for child in node.children() {
                        if let RedTree::Node(child_node) = child {
                            if let Some(expr) = self.build_expression(&child_node, source)? {
                                return Ok(Some(expr));
                            }
                        }
                    }
                    Ok(None)
                }
            }
            TypeScriptElementType::BinaryExpression => {
                let mut left = None;
                let mut operator = String::new();
                let mut right = None;

                for child in node.children() {
                    match child {
                        RedTree::Node(child_node) => {
                            if left.is_none() {
                                left = self.build_expression(&child_node, source)?
                            }
                            else {
                                right = self.build_expression(&child_node, source)?
                            }
                        }
                        RedTree::Leaf(leaf) => {
                            if left.is_some() && right.is_none() && operator.is_empty() && !leaf.kind.is_ignored() {
                                operator = match leaf.kind {
                                    TypeScriptTokenType::Plus => "+".to_string(),
                                    TypeScriptTokenType::Minus => "-".to_string(),
                                    TypeScriptTokenType::Star => "*".to_string(),
                                    TypeScriptTokenType::Slash => "/".to_string(),
                                    TypeScriptTokenType::Percent => "%".to_string(),
                                    TypeScriptTokenType::StarStar => "**".to_string(),
                                    TypeScriptTokenType::Less => "<".to_string(),
                                    TypeScriptTokenType::Greater => ">".to_string(),
                                    TypeScriptTokenType::LessEqual => "<=".to_string(),
                                    TypeScriptTokenType::GreaterEqual => ">=".to_string(),
                                    TypeScriptTokenType::EqualEqual => "==".to_string(),
                                    TypeScriptTokenType::EqualEqualEqual => "===".to_string(),
                                    TypeScriptTokenType::NotEqual => "!=".to_string(),
                                    TypeScriptTokenType::NotEqualEqual => "!==".to_string(),
                                    TypeScriptTokenType::AmpersandAmpersand => "&&".to_string(),
                                    TypeScriptTokenType::PipePipe => "||".to_string(),
                                    TypeScriptTokenType::Ampersand => "&".to_string(),
                                    TypeScriptTokenType::Pipe => "|".to_string(),
                                    TypeScriptTokenType::Caret => "^".to_string(),
                                    TypeScriptTokenType::LeftShift => "<<".to_string(),
                                    TypeScriptTokenType::RightShift => ">>".to_string(),
                                    TypeScriptTokenType::UnsignedRightShift => ">>>".to_string(),
                                    TypeScriptTokenType::PlusEqual => "+=".to_string(),
                                    TypeScriptTokenType::MinusEqual => "-=".to_string(),
                                    TypeScriptTokenType::StarEqual => "*=".to_string(),
                                    TypeScriptTokenType::SlashEqual => "/=".to_string(),
                                    TypeScriptTokenType::PercentEqual => "%=".to_string(),
                                    TypeScriptTokenType::StarStarEqual => "**=".to_string(),
                                    TypeScriptTokenType::LeftShiftEqual => "<<=".to_string(),
                                    TypeScriptTokenType::RightShiftEqual => ">>=".to_string(),
                                    TypeScriptTokenType::UnsignedRightShiftEqual => ">>>=".to_string(),
                                    TypeScriptTokenType::AmpersandEqual => "&=".to_string(),
                                    TypeScriptTokenType::PipeEqual => "|=".to_string(),
                                    TypeScriptTokenType::CaretEqual => "^=".to_string(),
                                    TypeScriptTokenType::AmpersandAmpersandEqual => "&&=".to_string(),
                                    TypeScriptTokenType::PipePipeEqual => "||=".to_string(),
                                    _ => source.get_text_in(leaf.span.into()).to_string(),
                                };
                            }
                        }
                    }
                }

                if let (Some(l), Some(r)) = (left, right) {
                    let is_assignment = matches!(operator.as_str(), "=" | "+=" | "-=" | "*=" | "/=" | "%=" | "**=" | "<<=" | ">>=" | ">>>=" | "&=" | "|=" | "^=" | "&&=" | "||=" | "??=");

                    if is_assignment {
                        Ok(Some(Expression::new(ExpressionKind::AssignmentExpression { left: Box::new(l), operator, right: Box::new(r) }, span.into())))
                    }
                    else {
                        Ok(Some(Expression::new(ExpressionKind::BinaryExpression { left: Box::new(l), operator, right: Box::new(r) }, span.into())))
                    }
                }
                else {
                    Ok(None)
                }
            }
            TypeScriptElementType::UnaryExpression => {
                let mut operator = String::new();
                let mut argument = None;

                for child in node.children() {
                    match child {
                        RedTree::Node(child_node) => argument = self.build_expression(&child_node, source)?,
                        RedTree::Leaf(leaf) => operator = source.get_text_in(leaf.span.into()).to_string(),
                    }
                }

                if let Some(arg) = argument { Ok(Some(Expression::new(ExpressionKind::UnaryExpression { operator, argument: Box::new(arg) }, span.into()))) } else { Ok(None) }
            }
            TypeScriptElementType::CallExpression => {
                let mut func = None;
                let mut args = Vec::new();

                for child in node.children() {
                    if let RedTree::Node(child_node) = child {
                        if func.is_none() {
                            func = self.build_expression(&child_node, source)?
                        }
                        else if child_node.green.kind == TypeScriptElementType::CallArgument {
                            for arg_child in child_node.children() {
                                if let RedTree::Node(arg_node) = arg_child {
                                    if let Some(arg) = self.build_expression(&arg_node, source)? {
                                        args.push(arg)
                                    }
                                }
                            }
                        }
                        else {
                            // Try to build as expression directly if it's not a CallArgument node
                            if let Some(arg) = self.build_expression(&child_node, source)? {
                                args.push(arg)
                            }
                        }
                    }
                }

                if let Some(f) = func { Ok(Some(Expression::new(ExpressionKind::CallExpression { func: Box::new(f), args }, span.into()))) } else { Ok(None) }
            }
            TypeScriptElementType::MemberExpression => {
                let mut object = None;
                let mut property = None;
                let mut computed = false;
                let mut optional = false;

                for child in node.children() {
                    match child {
                        RedTree::Node(child_node) => {
                            if object.is_none() {
                                object = self.build_expression(&child_node, source)?;
                            }
                            else {
                                property = self.build_expression(&child_node, source)?;
                            }
                        }
                        RedTree::Leaf(leaf) => match leaf.kind {
                            TypeScriptTokenType::LeftBracket => computed = true,
                            TypeScriptTokenType::QuestionDot => optional = true,
                            TypeScriptTokenType::IdentifierName => {
                                if object.is_some() && property.is_none() {
                                    let name = source.get_text_in(leaf.span.into()).trim().to_string();
                                    property = Some(Expression::new(ExpressionKind::Identifier(name), leaf.span.into()));
                                }
                            }
                            _ => {}
                        },
                    }
                }

                if object.is_some() && property.is_some() { Ok(Some(Expression::new(ExpressionKind::MemberExpression { object: Box::new(object.unwrap()), property: Box::new(property.unwrap()), computed, optional }, span.into()))) } else { Ok(None) }
            }
            TypeScriptElementType::ArrayExpression => {
                let mut elements = Vec::new();
                for child in node.children() {
                    if let RedTree::Node(child_node) = child {
                        if let Some(expr) = self.build_expression(&child_node, source)? {
                            elements.push(expr);
                        }
                    }
                }
                if elements.is_empty() {
                    // Try to find an inner ArrayExpression if current one seems empty
                    for child in node.children() {
                        if let RedTree::Node(child_node) = child {
                            if child_node.green.kind == TypeScriptElementType::ArrayExpression {
                                if let Some(expr) = self.build_expression(&child_node, source)? {
                                    return Ok(Some(expr));
                                }
                            }
                        }
                    }
                }
                Ok(Some(Expression::new(ExpressionKind::ArrayLiteral { elements }, span.into())))
            }
            TypeScriptElementType::ObjectExpression => {
                let mut properties = Vec::new();
                for child in node.children() {
                    if let RedTree::Node(child_node) = child {
                        let child_kind = child_node.green.kind;
                        if child_kind == TypeScriptElementType::PropertyAssignment {
                            let mut name = String::new();
                            let mut value = None;
                            let child_span = child_node.span();

                            for prop_child in child_node.children() {
                                if let RedTree::Node(prop_node) = prop_child {
                                    match prop_node.green.kind {
                                        TypeScriptElementType::IdentifierName => {
                                            if name.is_empty() {
                                                name = source.get_text_in(prop_node.span().into()).trim().to_string();
                                            }
                                            else if value.is_none() {
                                                value = self.build_expression(&prop_node, source)?;
                                            }
                                        }
                                        _ => {
                                            if value.is_none() {
                                                value = self.build_expression(&prop_node, source)?;
                                            }
                                        }
                                    }
                                }
                            }
                            if name.is_empty() {
                                let raw = source.get_text_in(child_span.into()).trim();
                                if let Some((key, _)) = raw.split_once(':') {
                                    name = key.trim().trim_matches(['"', '\'']).to_string();
                                }
                            }
                            if let Some(v) = value {
                                properties.push(ObjectProperty::Property { name, value: v, shorthand: false, span: child_span.into() });
                            }
                        }
                        else if child_kind == TypeScriptElementType::ShorthandPropertyAssignment {
                            let mut name = String::new();
                            let child_span = child_node.span();
                            for prop_child in child_node.children() {
                                if let RedTree::Node(prop_node) = prop_child {
                                    if prop_node.green.kind == TypeScriptElementType::IdentifierName {
                                        name = source.get_text_in(prop_node.span().into()).trim().to_string();
                                    }
                                }
                            }
                            let value = Expression::new(ExpressionKind::Identifier(name.clone()), child_span.into());
                            properties.push(ObjectProperty::Property { name, value, shorthand: true, span: child_span.into() });
                        }
                        else if child_kind == TypeScriptElementType::SpreadElement {
                            for spread_child in child_node.children() {
                                if let RedTree::Node(spread_node) = spread_child {
                                    if let Some(expr) = self.build_expression(&spread_node, source)? {
                                        properties.push(ObjectProperty::Spread(expr));
                                    }
                                }
                            }
                        }
                        else if child_kind == TypeScriptElementType::ObjectExpression {
                            // Recursively build nested object expression
                            if let Some(expr) = self.build_expression(&child_node, source)? {
                                if let ExpressionKind::ObjectLiteral { properties: inner_props } = *expr.kind {
                                    properties.extend(inner_props);
                                }
                            }
                        }
                    }
                }
                Ok(Some(Expression::new(ExpressionKind::ObjectLiteral { properties }, span.into())))
            }
            TypeScriptElementType::ArrowFunction => {
                let mut type_params = Vec::new();
                let mut params = Vec::new();
                let mut return_type = None;
                let mut body = None;
                let mut async_ = false;

                fn collect_arrow_parts(
                    builder: &TypeScriptBuilder,
                    node: &RedNode<TypeScriptLanguage>,
                    source: &SourceText,
                    type_params: &mut Vec<TypeParameter>,
                    params: &mut Vec<FunctionParam>,
                    return_type: &mut Option<TypeAnnotation>,
                    body: &mut Option<Box<Statement>>,
                    async_: &mut bool,
                ) -> Result<(), OakError> {
                    for child in node.children() {
                        match child {
                            RedTree::Node(child_node) => match child_node.green.kind {
                                TypeScriptElementType::ArrowFunction => {
                                    collect_arrow_parts(builder, &child_node, source, type_params, params, return_type, body, async_)?;
                                }
                                TypeScriptElementType::TypeParameter => {
                                    if let Some(tp) = builder.build_type_parameter(&child_node, source)? {
                                        type_params.push(tp);
                                    }
                                }
                                TypeScriptElementType::Parameter => {
                                    if let Some(p) = builder.build_parameter(&child_node, source)? {
                                        params.push(p);
                                    }
                                }
                                TypeScriptElementType::TypeAnnotation => {
                                    *return_type = builder.build_type_annotation(&child_node, source)?;
                                }
                                _ => {
                                    if let Some(stmt) = builder.build_statement(&child_node, source)? {
                                        *body = Some(Box::new(stmt));
                                    }
                                    else if let Some(expr) = builder.build_expression(&child_node, source)? {
                                        let expr_stmt = ExpressionStatement { decorators: Vec::new(), is_declare: false, expression: expr, span: child_node.span().into() };
                                        *body = Some(Box::new(Statement::ExpressionStatement(expr_stmt)));
                                    }
                                }
                            },
                            RedTree::Leaf(leaf) => {
                                if leaf.kind == TypeScriptTokenType::Async {
                                    *async_ = true;
                                }
                            }
                        }
                    }
                    Ok(())
                }

                collect_arrow_parts(self, node, source, &mut type_params, &mut params, &mut return_type, &mut body, &mut async_)?;

                if let Some(b) = body { Ok(Some(Expression::new(ExpressionKind::ArrowFunction { type_params, params, return_type, body: b, async_ }, span.into()))) } else { Ok(None) }
            }
            TypeScriptElementType::AsExpression => {
                let mut expression = None;
                let mut type_annotation = None;

                for child in node.children() {
                    if let RedTree::Node(child_node) = child {
                        if expression.is_none() { expression = self.build_expression(&child_node, source)? } else { type_annotation = self.build_type_annotation(&child_node, source)? }
                    }
                }

                if let (Some(expr), Some(ty)) = (expression, type_annotation) { Ok(Some(Expression::new(ExpressionKind::AsExpression { expression: Box::new(expr), type_annotation: ty }, span.into()))) } else { Ok(None) }
            }
            TypeScriptElementType::JsxElement => {
                if let Some(e) = self.build_jsx_element(node, source)? {
                    Ok(Some(Expression::new(ExpressionKind::JsxElement(Box::new(e)), span.into())))
                }
                else {
                    Ok(None)
                }
            }
            TypeScriptElementType::JsxSelfClosingElement => {
                if let Some(e) = self.build_jsx_self_closing_element(node, source)? {
                    Ok(Some(Expression::new(ExpressionKind::JsxSelfClosingElement(Box::new(e)), span.into())))
                }
                else {
                    Ok(None)
                }
            }
            TypeScriptElementType::JsxFragment => {
                if let Some(f) = self.build_jsx_fragment(node, source)? {
                    Ok(Some(Expression::new(ExpressionKind::JsxFragment(Box::new(f)), span.into())))
                }
                else {
                    Ok(None)
                }
            }
            TypeScriptElementType::TemplateExpression => {
                let text = source.get_text_in(span.into()).to_string();
                Ok(Some(Expression::new(ExpressionKind::TemplateString(text), span.into())))
            }
            TypeScriptElementType::ConditionalExpression => {
                let mut test = None;
                let mut consequent = None;
                let mut alternate = None;

                for child in node.children() {
                    if let RedTree::Node(child_node) = child {
                        if test.is_none() {
                            test = self.build_expression(&child_node, source)?
                        }
                        else if consequent.is_none() {
                            consequent = self.build_expression(&child_node, source)?
                        }
                        else {
                            alternate = self.build_expression(&child_node, source)?
                        }
                    }
                }

                if let (Some(t), Some(c), Some(a)) = (test, consequent, alternate) { Ok(Some(Expression::new(ExpressionKind::ConditionalExpression { test: Box::new(t), consequent: Box::new(c), alternate: Box::new(a) }, span.into()))) } else { Ok(None) }
            }
            TypeScriptElementType::NewExpression => {
                let mut func = None;
                let mut args = Vec::new();

                for child in node.children() {
                    if let RedTree::Node(child_node) = child {
                        if func.is_none() {
                            func = self.build_expression(&child_node, source)?
                        }
                        else if child_node.green.kind == TypeScriptElementType::CallArgument {
                            for arg_child in child_node.children() {
                                if let RedTree::Node(arg_node) = arg_child {
                                    if let Some(arg) = self.build_expression(&arg_node, source)? {
                                        args.push(arg)
                                    }
                                }
                            }
                        }
                        else if let Some(expr) = self.build_expression(&child_node, source)? {
                            args.push(expr)
                        }
                    }
                }

                if let Some(f) = func { Ok(Some(Expression::new(ExpressionKind::NewExpression { func: Box::new(f), args }, span.into()))) } else { Ok(None) }
            }
            TypeScriptElementType::UpdateExpression => {
                let mut operator = String::new();
                let mut argument = None;
                let mut prefix = true;

                for child in node.children() {
                    match child {
                        RedTree::Node(child_node) => argument = self.build_expression(&child_node, source)?,
                        RedTree::Leaf(leaf) => {
                            let leaf_text = source.get_text_in(leaf.span.into()).to_string();
                            if leaf_text == "++" || leaf_text == "--" {
                                operator = leaf_text;
                                if argument.is_some() {
                                    prefix = false
                                }
                            }
                        }
                    }
                }

                if let Some(arg) = argument { Ok(Some(Expression::new(ExpressionKind::UpdateExpression { operator, argument: Box::new(arg), prefix }, span.into()))) } else { Ok(None) }
            }
            _ => {
                for child in node.children() {
                    if let RedTree::Node(child_node) = child {
                        if let Some(expr) = self.build_expression(&child_node, source)? {
                            return Ok(Some(expr));
                        }
                    }
                }
                Ok(None)
            }
        }
    }
}

fn decode_string_literal_text(raw: &str) -> String {
    let text = raw.trim();
    if (text.starts_with('"') && text.ends_with('"')) || (text.starts_with('\'') && text.ends_with('\'')) { text[1..text.len() - 1].to_string() } else { text.to_string() }
}
