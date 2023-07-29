use crate::{
    MatlabLanguage,
    ast::{Expression, Statement},
    builder::MatlabBuilder,
    lexer::token_type::MatlabTokenType,
    parser::element_type::MatlabElementType,
};
use oak_core::{OakError, RedNode, RedTree, Source};

#[derive(Clone, Copy, PartialEq, Eq)]
enum IfPhase {
    Cond,
    Then,
    ElseifCond,
    ElseifBody,
    Else,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum SwitchPhase {
    Disc,
    AfterDisc,
    CaseValue,
    CaseBody,
    Otherwise,
}

impl<'config> MatlabBuilder<'config> {
    pub(crate) fn build_stmt<S: Source + ?Sized>(&self, node: RedNode<'_, MatlabLanguage>, source: &S) -> Result<Statement, OakError> {
        let span = node.span();
        match node.element_type() {
            MatlabElementType::IfStmt => self.build_if(node, source),
            MatlabElementType::WhileStmt => self.build_while(node, source),
            MatlabElementType::ForStmt => self.build_for(node, source),
            MatlabElementType::SwitchStmt => self.build_switch(node, source),
            MatlabElementType::TryStmt => self.build_try(node, source),
            MatlabElementType::Error => Ok(Statement::Error { span }),
            kind if crate::builder::utils::is_expr_kind(kind) => Ok(Statement::Expr(self.build_expr(node, source)?)),
            other => Err(source.syntax_error(format!("Unexpected MATLAB statement kind: {other:?}"), span.start)),
        }
    }

    fn build_if<S: Source + ?Sized>(&self, node: RedNode<'_, MatlabLanguage>, source: &S) -> Result<Statement, OakError> {
        let span = node.span();
        let mut condition = None;
        let mut then_body = Vec::new();
        let mut elseifs = Vec::new();
        let mut else_body = Vec::new();
        let mut elseif_body = Vec::new();
        let mut pending_elseif_cond: Option<Expression> = None;
        let mut phase = IfPhase::Cond;

        for child in node.children() {
            match child {
                RedTree::Leaf(t) => match t.kind() {
                    MatlabTokenType::Elseif => {
                        if let Some(cond) = pending_elseif_cond.take() {
                            elseifs.push((cond, std::mem::take(&mut elseif_body)));
                        }
                        phase = IfPhase::ElseifCond;
                    }
                    MatlabTokenType::Else => {
                        if let Some(cond) = pending_elseif_cond.take() {
                            elseifs.push((cond, std::mem::take(&mut elseif_body)));
                        }
                        phase = IfPhase::Else;
                    }
                    _ => {}
                },
                RedTree::Node(n) => match phase {
                    IfPhase::Cond => {
                        condition = Some(self.build_expr(n, source)?);
                        phase = IfPhase::Then;
                    }
                    IfPhase::Then => then_body.push(self.build_stmt(n, source)?),
                    IfPhase::ElseifCond => {
                        pending_elseif_cond = Some(self.build_expr(n, source)?);
                        phase = IfPhase::ElseifBody;
                    }
                    IfPhase::ElseifBody => elseif_body.push(self.build_stmt(n, source)?),
                    IfPhase::Else => else_body.push(self.build_stmt(n, source)?),
                },
            }
        }
        if let Some(cond) = pending_elseif_cond.take() {
            elseifs.push((cond, elseif_body));
        }

        let condition = condition.ok_or_else(|| source.syntax_error("If missing condition".into(), span.start))?;
        Ok(Statement::If { condition, then_body, elseifs, else_body, span })
    }

    fn build_while<S: Source + ?Sized>(&self, node: RedNode<'_, MatlabLanguage>, source: &S) -> Result<Statement, OakError> {
        let span = node.span();
        let mut condition = None;
        let mut body = Vec::new();
        for child in node.children() {
            if let RedTree::Node(n) = child {
                if condition.is_none() {
                    condition = Some(self.build_expr(n, source)?);
                }
                else {
                    body.push(self.build_stmt(n, source)?);
                }
            }
        }
        let condition = condition.ok_or_else(|| source.syntax_error("While missing condition".into(), span.start))?;
        Ok(Statement::While { condition, body, span })
    }

    fn build_for<S: Source + ?Sized>(&self, node: RedNode<'_, MatlabLanguage>, source: &S) -> Result<Statement, OakError> {
        let span = node.span();
        let mut header = None;
        let mut body = Vec::new();
        for child in node.children() {
            if let RedTree::Node(n) = child {
                if header.is_none() {
                    header = Some(self.build_expr(n, source)?);
                }
                else {
                    body.push(self.build_stmt(n, source)?);
                }
            }
        }
        let header = header.ok_or_else(|| source.syntax_error("For missing header".into(), span.start))?;
        Ok(Statement::For { header, body, span })
    }

    fn build_switch<S: Source + ?Sized>(&self, node: RedNode<'_, MatlabLanguage>, source: &S) -> Result<Statement, OakError> {
        let span = node.span();
        let mut discriminant = None;
        let mut cases = Vec::new();
        let mut otherwise = Vec::new();
        let mut pending_case: Option<Expression> = None;
        let mut case_body = Vec::new();
        let mut phase = SwitchPhase::Disc;

        for child in node.children() {
            match child {
                RedTree::Leaf(t) => match t.kind() {
                    MatlabTokenType::Case => {
                        if let Some(val) = pending_case.take() {
                            cases.push((val, std::mem::take(&mut case_body)));
                        }
                        phase = SwitchPhase::CaseValue;
                    }
                    MatlabTokenType::Otherwise => {
                        if let Some(val) = pending_case.take() {
                            cases.push((val, std::mem::take(&mut case_body)));
                        }
                        phase = SwitchPhase::Otherwise;
                    }
                    _ => {}
                },
                RedTree::Node(n) => match phase {
                    SwitchPhase::Disc => {
                        discriminant = Some(self.build_expr(n, source)?);
                        phase = SwitchPhase::AfterDisc;
                    }
                    SwitchPhase::AfterDisc => {
                        // Unexpected node before first case — treat as error recovery skip.
                    }
                    SwitchPhase::CaseValue => {
                        pending_case = Some(self.build_expr(n, source)?);
                        phase = SwitchPhase::CaseBody;
                    }
                    SwitchPhase::CaseBody => case_body.push(self.build_stmt(n, source)?),
                    SwitchPhase::Otherwise => otherwise.push(self.build_stmt(n, source)?),
                },
            }
        }
        if let Some(val) = pending_case.take() {
            cases.push((val, case_body));
        }

        let discriminant = discriminant.ok_or_else(|| source.syntax_error("Switch missing discriminant".into(), span.start))?;
        Ok(Statement::Switch { discriminant, cases, otherwise, span })
    }

    fn build_try<S: Source + ?Sized>(&self, node: RedNode<'_, MatlabLanguage>, source: &S) -> Result<Statement, OakError> {
        let span = node.span();
        let mut body = Vec::new();
        let mut catch_body = Vec::new();
        let mut catch_name = None;
        let mut in_catch = false;
        let mut saw_catch_child = false;

        for child in node.children() {
            match child {
                RedTree::Leaf(t) => {
                    if t.kind() == MatlabTokenType::Catch {
                        in_catch = true;
                    }
                }
                RedTree::Node(n) => {
                    if !in_catch {
                        body.push(self.build_stmt(n, source)?);
                    }
                    else if !saw_catch_child {
                        saw_catch_child = true;
                        if crate::builder::utils::is_expr_kind(n.element_type()) {
                            let expr = self.build_expr(n, source)?;
                            if matches!(expr, Expression::Symbol(_)) {
                                catch_name = Some(expr);
                            }
                            else {
                                catch_body.push(Statement::Expr(expr));
                            }
                        }
                        else {
                            catch_body.push(self.build_stmt(n, source)?);
                        }
                    }
                    else {
                        catch_body.push(self.build_stmt(n, source)?);
                    }
                }
            }
        }

        Ok(Statement::Try { body, catch_name, catch_body, span })
    }
}
