use crate::{
    ast::{AttributeOperator, AttributeSelector, Combinator, CompoundSelector, Selector, SelectorList, SimpleSelector},
    diag::CssSelectorParseError,
};

enum SimpleStart {
    Universal,
    Type,
    Class,
    Id,
    Attribute,
    Pseudo,
}

/// Parse a CSS selector list into an AST.
pub fn parse_css_selector(source: &str) -> Result<SelectorList, CssSelectorParseError> {
    let mut parser = Parser::new(source);
    let list = parser.parse_list()?;
    parser.expect_end()?;
    Ok(list)
}

struct Parser<'a> {
    source: &'a str,
    pos: usize,
}

impl<'a> Parser<'a> {
    fn new(source: &'a str) -> Self {
        Self { source, pos: 0 }
    }

    fn parse_list(&mut self) -> Result<SelectorList, CssSelectorParseError> {
        let mut selectors = Vec::new();
        self.skip_ws();
        if self.at_end() {
            return Err(CssSelectorParseError::unexpected_eof("selector", self.pos));
        }
        selectors.push(self.parse_selector()?);
        while {
            self.skip_ws();
            self.peek() == Some(',')
        } {
            self.pos += 1;
            self.skip_ws();
            selectors.push(self.parse_selector()?);
        }
        Ok(SelectorList { selectors })
    }

    fn parse_selector(&mut self) -> Result<Selector, CssSelectorParseError> {
        let compound = self.parse_compound()?;
        let mut ancestors = Vec::new();
        let mut current = compound;
        loop {
            self.skip_ws();
            if self.at_end() || self.peek() == Some(',') {
                break;
            }
            let combinator = if self.peek() == Some('>') {
                self.pos += 1;
                Combinator::Child
            }
            else if self.peek() == Some('+') {
                self.pos += 1;
                Combinator::NextSibling
            }
            else if self.peek() == Some('~') {
                self.pos += 1;
                Combinator::SubsequentSibling
            }
            else {
                Combinator::Descendant
            };
            self.skip_ws();
            let ancestor = self.parse_compound()?;
            ancestors.push((combinator, current));
            current = ancestor;
        }
        Ok(Selector { compound: current, ancestors })
    }

    fn parse_compound(&mut self) -> Result<CompoundSelector, CssSelectorParseError> {
        let mut simple = Vec::new();
        while let Some(next) = self.peek_simple_start() {
            match next {
                SimpleStart::Universal => {
                    self.pos += 1;
                    simple.push(SimpleSelector::Universal);
                }
                SimpleStart::Type => simple.push(SimpleSelector::Type(self.parse_ident()?)),
                SimpleStart::Class => {
                    self.pos += 1;
                    simple.push(SimpleSelector::Class(self.parse_ident()?));
                }
                SimpleStart::Id => {
                    self.pos += 1;
                    simple.push(SimpleSelector::Id(self.parse_ident()?));
                }
                SimpleStart::Attribute => simple.push(SimpleSelector::Attribute(self.parse_attribute()?)),
                SimpleStart::Pseudo => {
                    self.pos += 1;
                    self.expect_char(':')?;
                    simple.push(SimpleSelector::PseudoClass(self.parse_ident()?));
                }
            }
        }
        if simple.is_empty() {
            return Err(CssSelectorParseError::unexpected_eof("simple selector", self.pos));
        }
        Ok(CompoundSelector { simple })
    }

    fn parse_attribute(&mut self) -> Result<AttributeSelector, CssSelectorParseError> {
        self.expect_char('[')?;
        self.skip_ws();
        let name = self.parse_ident()?;
        self.skip_ws();
        if self.peek() == Some(']') {
            self.pos += 1;
            return Ok(AttributeSelector { name, operator: AttributeOperator::Present, value: None });
        }
        let (operator, value) = match self.peek() {
            Some('=') => {
                self.pos += 1;
                (AttributeOperator::Equals, Some(self.parse_string_or_ident()?))
            }
            Some('^') => {
                self.pos += 2;
                (AttributeOperator::StartsWith, Some(self.parse_string_or_ident()?))
            }
            Some('$') => {
                self.pos += 2;
                (AttributeOperator::Suffix, Some(self.parse_string_or_ident()?))
            }
            Some('*') => {
                self.pos += 2;
                (AttributeOperator::Substring, Some(self.parse_string_or_ident()?))
            }
            Some('~') => {
                self.pos += 2;
                (AttributeOperator::Includes, Some(self.parse_string_or_ident()?))
            }
            Some('|') => {
                self.pos += 2;
                (AttributeOperator::DashMatch, Some(self.parse_string_or_ident()?))
            }
            _ => return Err(CssSelectorParseError::unexpected_eof("attribute operator", self.pos)),
        };
        self.skip_ws();
        self.expect_char(']')?;
        Ok(AttributeSelector { name, operator, value })
    }

    fn parse_string_or_ident(&mut self) -> Result<String, CssSelectorParseError> {
        if self.peek() == Some('"') || self.peek() == Some('\'') { self.parse_string_literal() } else { self.parse_ident() }
    }

    fn parse_string_literal(&mut self) -> Result<String, CssSelectorParseError> {
        let quote = self.peek().ok_or_else(|| CssSelectorParseError::unexpected_eof("string literal", self.pos))?;
        if quote != '"' && quote != '\'' {
            return Err(CssSelectorParseError::invalid_ident("expected string literal", self.pos));
        }
        self.pos += 1;
        let start = self.pos;
        while let Some(ch) = self.peek() {
            if ch == quote {
                let value = self.source[start..self.pos].to_string();
                self.pos += 1;
                return Ok(value);
            }
            self.pos += 1;
        }
        Err(CssSelectorParseError::unexpected_eof("string literal", start))
    }

    fn parse_ident(&mut self) -> Result<String, CssSelectorParseError> {
        let start = self.pos;
        let mut out = String::new();
        while let Some(ch) = self.peek() {
            if ch.is_ascii_alphanumeric() || ch == '_' || ch == '-' {
                out.push(ch);
                self.pos += 1;
            }
            else {
                break;
            }
        }
        if out.is_empty() {
            return Err(CssSelectorParseError::invalid_ident("", start));
        }
        Ok(out)
    }

    fn peek_simple_start(&self) -> Option<SimpleStart> {
        match self.peek() {
            Some('*') => Some(SimpleStart::Universal),
            Some('.') => Some(SimpleStart::Class),
            Some('#') => Some(SimpleStart::Id),
            Some('[') => Some(SimpleStart::Attribute),
            Some(':') => Some(SimpleStart::Pseudo),
            Some(ch) if ch.is_ascii_alphabetic() || ch == '_' => Some(SimpleStart::Type),
            _ => None,
        }
    }

    fn expect_char(&mut self, expected: char) -> Result<(), CssSelectorParseError> {
        if self.peek() == Some(expected) {
            self.pos += 1;
            Ok(())
        }
        else {
            Err(CssSelectorParseError::unexpected_eof(format!("expected '{expected}'").as_str(), self.pos))
        }
    }

    fn expect_end(&mut self) -> Result<(), CssSelectorParseError> {
        self.skip_ws();
        if self.at_end() { Ok(()) } else { Err(CssSelectorParseError::unexpected_trailing(self.remaining(), self.pos)) }
    }

    fn skip_ws(&mut self) {
        while let Some(ch) = self.peek() {
            if ch.is_whitespace() {
                self.pos += 1;
            }
            else {
                break;
            }
        }
    }

    fn peek(&self) -> Option<char> {
        self.source[self.pos..].chars().next()
    }

    fn at_end(&self) -> bool {
        self.pos >= self.source.len()
    }

    fn remaining(&self) -> &str {
        self.source[self.pos..].trim()
    }
}

#[cfg(test)]
mod tests {
    use super::parse_css_selector;
    use crate::ast::{AttributeOperator, AttributeSelector, Combinator, CompoundSelector, Selector, SelectorList, SimpleSelector};

    #[test]
    fn parses_type_and_class() {
        let list = parse_css_selector("article > h1.title").expect("parse");
        assert_eq!(
            list,
            SelectorList {
                selectors: vec![Selector {
                    compound: CompoundSelector { simple: vec![SimpleSelector::Type("h1".to_string()), SimpleSelector::Class("title".to_string()),] },
                    ancestors: vec![(Combinator::Child, CompoundSelector { simple: vec![SimpleSelector::Type("article".to_string())] },)],
                }],
            }
        );
    }

    #[test]
    fn parses_attribute_prefix() {
        let list = parse_css_selector("nav a[href^=\"/docs/\"]").expect("parse");
        let compound = &list.selectors[0].compound;
        assert!(compound.simple.iter().any(|simple| {
            matches!(
                simple,
                SimpleSelector::Attribute(AttributeSelector {
                    name,
                    operator: AttributeOperator::StartsWith,
                    value: Some(value),
                }) if name == "href" && value == "/docs/"
            )
        }));
    }

    #[test]
    fn parses_selector_list() {
        let list = parse_css_selector("article > h1, article > h2").expect("parse");
        assert_eq!(list.selectors.len(), 2);
    }
}
