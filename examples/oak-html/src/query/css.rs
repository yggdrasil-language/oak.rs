use super::view::{HtmlDocumentView, HtmlNodeData};
use crate::ast::Element;
use oak_core::query::{ElementRef, ElementView, QueryBudget, SelectorOutcome, SelectorResult};
use oak_css_selector::{AttributeOperator, Combinator, CompoundSelector, CssSelectorParseError, Selector, SelectorList, SimpleSelector, parse_css_selector};

/// Executes a CSS selector list against an HTML document view.
pub fn select_css<'a>(document: &HtmlDocumentView<'a>, selector: &str, budget: QueryBudget) -> Result<SelectorResult, CssSelectorParseError> {
    let list = parse_css_selector(selector)?;
    Ok(execute_css(document, &list, budget))
}

/// Executes CSS selectors and maps matches back to source AST elements.
pub fn select_css_elements<'a>(document: &HtmlDocumentView<'a>, selector: &str, budget: QueryBudget) -> Result<(SelectorResult, Vec<&'a Element>), CssSelectorParseError> {
    let result = select_css(document, selector, budget)?;
    let elements = document.source_elements(&result.matches);
    Ok((result, elements))
}

fn execute_css<'a>(document: &HtmlDocumentView<'a>, list: &SelectorList, budget: QueryBudget) -> SelectorResult {
    let mut matches = Vec::new();
    for reference in document.all_elements() {
        if matches.len() >= budget.max_matches {
            return SelectorResult { outcome: SelectorOutcome::BudgetExceeded, matches };
        }
        if list.selectors.iter().any(|selector| matches_selector(document, reference, selector)) {
            matches.push(reference);
        }
    }
    SelectorResult { outcome: SelectorOutcome::Matched, matches }
}

fn matches_selector<'a>(document: &HtmlDocumentView<'a>, reference: ElementRef, selector: &Selector) -> bool {
    if !matches_compound(document, reference, &selector.compound) {
        return false;
    }
    let mut current = reference;
    for (combinator, ancestor) in selector.ancestors.iter().rev() {
        let Some(parent) = find_relative_match(document, current, combinator, ancestor)
        else {
            return false;
        };
        current = parent;
    }
    true
}

fn find_relative_match<'a>(document: &HtmlDocumentView<'a>, reference: ElementRef, combinator: &Combinator, compound: &CompoundSelector) -> Option<ElementRef> {
    match combinator {
        Combinator::Child => document.element(reference.node_id).and_then(|view| view.parent()).filter(|parent| matches_compound(document, *parent, compound)),
        Combinator::Descendant => {
            let mut current = document.element(reference.node_id).and_then(|view| view.parent());
            while let Some(parent) = current {
                if matches_compound(document, parent, compound) {
                    return Some(parent);
                }
                current = document.element(parent.node_id).and_then(|view| view.parent());
            }
            None
        }
        Combinator::NextSibling => document.previous_element_sibling(reference).filter(|sibling| matches_compound(document, *sibling, compound)),
        Combinator::SubsequentSibling => {
            let mut current = document.previous_element_sibling(reference);
            while let Some(sibling) = current {
                if matches_compound(document, sibling, compound) {
                    return Some(sibling);
                }
                current = document.previous_element_sibling(sibling);
            }
            None
        }
    }
}

fn matches_compound<'a>(document: &HtmlDocumentView<'a>, reference: ElementRef, compound: &CompoundSelector) -> bool {
    let Some(node) = document.node(reference.node_id)
    else {
        return false;
    };
    compound.simple.iter().all(|simple| matches_simple(node, simple))
}

fn matches_simple(node: &HtmlNodeData, simple: &SimpleSelector) -> bool {
    match simple {
        SimpleSelector::Universal => true,
        SimpleSelector::Type(tag) => node.tag_name.eq_ignore_ascii_case(tag),
        SimpleSelector::Class(class) => node.classes.iter().any(|value| value == class),
        SimpleSelector::Id(id) => node.id.as_deref() == Some(id.as_str()),
        SimpleSelector::Attribute(attribute) => matches_attribute(node, attribute),
        SimpleSelector::PseudoClass(_) => false,
    }
}

fn matches_attribute(node: &HtmlNodeData, attribute: &oak_css_selector::AttributeSelector) -> bool {
    let value = node.attribute_value(&attribute.name);
    match attribute.operator {
        AttributeOperator::Present => value.is_some(),
        AttributeOperator::Equals => value == attribute.value.as_deref(),
        AttributeOperator::StartsWith => value.zip(attribute.value.as_deref()).is_some_and(|(left, right)| left.starts_with(right)),
        AttributeOperator::Suffix => value.zip(attribute.value.as_deref()).is_some_and(|(left, right)| left.ends_with(right)),
        AttributeOperator::Substring => value.zip(attribute.value.as_deref()).is_some_and(|(left, right)| left.contains(right)),
        AttributeOperator::Includes => value.zip(attribute.value.as_deref()).is_some_and(|(left, right)| left.split_whitespace().any(|token| token == right)),
        AttributeOperator::DashMatch => value.zip(attribute.value.as_deref()).is_some_and(|(left, right)| left == right || left.starts_with(&format!("{right}-"))),
    }
}
