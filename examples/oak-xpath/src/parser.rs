use crate::{
    ast::{Axis, NodeTest, PathExpr, Predicate, QName, Step, XpathExpr},
    diag::XpathParseError,
};

/// Parse an XPath subset selector into an AST.
pub fn parse_xpath(source: &str) -> Result<XpathExpr, XpathParseError> {
    let mut parser = Parser::new(source);
    let expr = parser.parse_expr()?;
    parser.expect_end()?;
    Ok(expr)
}

struct Parser<'a> {
    source: &'a str,
    pos: usize,
}

impl<'a> Parser<'a> {
    fn new(source: &'a str) -> Self {
        Self { source, pos: 0 }
    }

    fn parse_expr(&mut self) -> Result<XpathExpr, XpathParseError> {
        let first = self.parse_path()?;
        let mut paths = vec![first];
        while {
            self.skip_ws();
            self.peek() == Some('|')
        } {
            self.pos += 1;
            paths.push(self.parse_path()?);
        }
        Ok(XpathExpr { paths })
    }

    fn parse_path(&mut self) -> Result<PathExpr, XpathParseError> {
        self.skip_ws();
        let mut absolute = false;
        let mut leading_descendant = false;

        if self.peek_two() == Some("//") {
            self.pos += 2;
            absolute = true;
            leading_descendant = true;
        }
        else if self.peek() == Some('/') {
            self.pos += 1;
            absolute = true;
        }

        let mut steps = Vec::new();
        if leading_descendant && !self.at_end() && self.peek() != Some('|') && self.peek() != Some(')') {
            let step = self.parse_step(Axis::Descendant)?;
            steps.push(step);
        }

        while !self.at_end() {
            self.skip_ws();
            if self.peek() == Some('|') || self.peek() == Some(')') {
                break;
            }

            let axis = if self.peek_two() == Some("//") {
                self.pos += 2;
                Axis::Descendant
            }
            else if self.peek() == Some('/') {
                self.pos += 1;
                Axis::Child
            }
            else if steps.is_empty() {
                Axis::Child
            }
            else {
                break;
            };

            if self.at_end() || self.peek() == Some('|') {
                return Err(XpathParseError::unexpected_eof("step after path separator", self.pos));
            }

            steps.push(self.parse_step(axis)?);
        }

        Ok(PathExpr { absolute, steps })
    }

    fn parse_step(&mut self, axis: Axis) -> Result<Step, XpathParseError> {
        self.skip_ws();
        let (axis, test) = if self.peek() == Some('@') {
            self.pos += 1;
            let qname = self.parse_qname()?;
            (Axis::Attribute, NodeTest::Name(qname))
        }
        else {
            (axis, self.parse_node_test()?)
        };

        let mut predicates = Vec::new();
        while self.peek() == Some('[') {
            predicates.push(self.parse_predicate()?);
        }

        Ok(Step { axis, test, predicates })
    }

    fn parse_node_test(&mut self) -> Result<NodeTest, XpathParseError> {
        self.skip_ws();
        let start = self.pos;

        if self.peek() == Some('*') {
            self.pos += 1;
            if self.peek() == Some(':') {
                self.pos += 1;
                if self.peek() == Some('*') {
                    self.pos += 1;
                    return Err(XpathParseError::invalid_qname("*:*", start));
                }
                let local = self.parse_name()?;
                return Ok(NodeTest::PrefixedWildcard(QName { prefix: Some(local), local: "*".to_string() }));
            }
            return Ok(NodeTest::AnyElement);
        }

        let qname = self.parse_qname()?;
        if qname.local == "*" {
            match qname.prefix {
                Some(prefix) => Ok(NodeTest::PrefixedWildcard(QName { prefix: Some(prefix), local: "*".to_string() })),
                None => Ok(NodeTest::AnyElement),
            }
        }
        else {
            Ok(NodeTest::Name(qname))
        }
    }

    fn parse_qname(&mut self) -> Result<QName, XpathParseError> {
        let start = self.pos;
        let first = self.parse_name()?;
        if self.peek() == Some(':') {
            self.pos += 1;
            if self.peek() == Some('*') {
                self.pos += 1;
                return Ok(QName { prefix: Some(first), local: "*".to_string() });
            }
            let local = self.parse_name()?;
            return Ok(QName { prefix: Some(first), local });
        }
        if first.is_empty() {
            return Err(XpathParseError::invalid_qname("", start));
        }
        Ok(QName { prefix: None, local: first })
    }

    fn parse_name(&mut self) -> Result<String, XpathParseError> {
        let start = self.pos;
        let mut out = String::new();
        while let Some(ch) = self.peek() {
            if ch.is_ascii_alphanumeric() || ch == '_' || ch == '-' || ch == '.' {
                out.push(ch);
                self.pos += 1;
            }
            else {
                break;
            }
        }
        if out.is_empty() {
            return Err(XpathParseError::invalid_qname("", start));
        }
        Ok(out)
    }

    fn parse_predicate(&mut self) -> Result<Predicate, XpathParseError> {
        self.expect_char('[')?;
        self.skip_ws();
        let predicate = self.parse_predicate_body()?;
        self.skip_ws();
        self.expect_char(']')?;
        Ok(predicate)
    }

    fn parse_predicate_body(&mut self) -> Result<Predicate, XpathParseError> {
        let start = self.pos;
        if self.peek() == Some('@') {
            self.pos += 1;
            let qname = self.parse_qname()?;
            self.skip_ws();
            if self.peek() == Some('=') {
                self.pos += 1;
                self.skip_ws();
                let value = self.parse_string_literal()?;
                return Ok(Predicate::AttributeEquals(qname, value));
            }
        }

        if let Some(position) = self.parse_u32()? {
            return Ok(Predicate::Position(position));
        }

        let end = self.find_matching(']')?;
        let fragment = self.source[start..end].trim();
        Ok(Predicate::Unsupported(fragment.to_string()))
    }

    fn parse_u32(&mut self) -> Result<Option<u32>, XpathParseError> {
        let start = self.pos;
        let mut value = 0u32;
        let mut digits = 0usize;
        while let Some(ch) = self.peek() {
            if let Some(digit) = ch.to_digit(10) {
                value = value.checked_mul(10).and_then(|next| next.checked_add(digit)).ok_or_else(|| XpathParseError::unsupported_predicate("numeric overflow", start))?;
                digits += 1;
                self.pos += 1;
            }
            else {
                break;
            }
        }
        Ok(if digits > 0 { Some(value) } else { None })
    }

    fn parse_string_literal(&mut self) -> Result<String, XpathParseError> {
        let quote = self.peek().ok_or_else(|| XpathParseError::unexpected_eof("string literal", self.pos))?;
        if quote != '"' && quote != '\'' {
            return Err(XpathParseError::unsupported_predicate("expected string literal", self.pos));
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
        Err(XpathParseError::unclosed_string(start))
    }

    fn find_matching(&mut self, closing: char) -> Result<usize, XpathParseError> {
        let start = self.pos;
        let mut depth = 1usize;
        while let Some(ch) = self.peek() {
            self.pos += 1;
            if ch == '[' {
                depth += 1;
            }
            else if ch == closing {
                depth -= 1;
                if depth == 0 {
                    return Ok(self.pos - 1);
                }
            }
        }
        Err(XpathParseError::unexpected_eof("predicate", start))
    }

    fn expect_char(&mut self, expected: char) -> Result<(), XpathParseError> {
        if self.peek() == Some(expected) {
            self.pos += 1;
            Ok(())
        }
        else {
            Err(XpathParseError::unexpected_eof(format!("expected '{expected}'").as_str(), self.pos))
        }
    }

    fn expect_end(&mut self) -> Result<(), XpathParseError> {
        self.skip_ws();
        if self.at_end() { Ok(()) } else { Err(XpathParseError::unexpected_trailing(self.remaining(), self.pos)) }
    }

    fn skip_ws(&mut self) -> bool {
        let mut moved = false;
        while let Some(ch) = self.peek() {
            if ch.is_whitespace() {
                self.pos += 1;
                moved = true;
            }
            else {
                break;
            }
        }
        moved
    }

    fn peek(&self) -> Option<char> {
        self.source[self.pos..].chars().next()
    }

    fn peek_two(&self) -> Option<&'a str> {
        if self.source[self.pos..].starts_with("//") { Some("//") } else { None }
    }

    fn at_end(&self) -> bool {
        self.pos >= self.source.len()
    }

    fn remaining(&self) -> &str {
        self.source[self.pos..].trim()
    }
}
