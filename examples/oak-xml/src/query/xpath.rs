use super::view::{XmlDocumentView, XmlElementView};
use super::view::XmlNodeData;
use oak_core::query::{ElementRef, ElementView, QueryBudget, SelectorOutcome, SelectorResult};
use oak_xpath::{
    parse_xpath, Axis, NodeTest, PathExpr, Predicate, Step, XpathExpr, XpathParseError,
};

/// Namespace bindings used to resolve prefixed names during XPath execution.
#[derive(Debug, Clone, Default)]
pub struct XmlNamespaceContext {
    prefixes: Vec<(String, String)>,
}

impl XmlNamespaceContext {
    /// Creates an empty namespace context.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Binds a prefix to a namespace URI.
    pub fn bind(&mut self, prefix: impl Into<String>, uri: impl Into<String>) {
        self.prefixes.push((prefix.into(), uri.into()));
    }
}

/// Executes an XPath subset against an XML document view.
pub fn select_xpath(
    document: &XmlDocumentView,
    selector: &str,
    budget: QueryBudget,
) -> Result<SelectorResult, XpathParseError> {
    let expr = parse_xpath(selector)?;
    Ok(execute_xpath(document, &expr, budget))
}

fn execute_xpath(document: &XmlDocumentView, expr: &XpathExpr, budget: QueryBudget) -> SelectorResult {
    let mut matches = Vec::new();
    let mut predicates_used = 0usize;

    for path in &expr.paths {
        let context = root_elements(document);

        let path_matches = evaluate_path(document, path, &context, budget, &mut predicates_used);
        for reference in path_matches {
            if matches.len() >= budget.max_matches {
                return SelectorResult {
                    outcome: SelectorOutcome::BudgetExceeded,
                    matches,
                };
            }
            if !matches.contains(&reference) {
                matches.push(reference);
            }
        }
    }

    SelectorResult {
        outcome: SelectorOutcome::Matched,
        matches,
    }
}

fn root_elements(document: &XmlDocumentView) -> Vec<ElementRef> {
    document
        .all_elements()
        .into_iter()
        .filter(|reference| document.node(reference.node_id).is_some_and(|node| node.parent.is_none()))
        .collect()
}

fn evaluate_path(
    document: &XmlDocumentView,
    path: &PathExpr,
    context: &[ElementRef],
    budget: QueryBudget,
    predicates_used: &mut usize,
) -> Vec<ElementRef> {
    if path.steps.is_empty() {
        return context.to_vec();
    }

    let mut current = context.to_vec();
    for (index, step) in path.steps.iter().enumerate() {
        let first_absolute = path.absolute && index == 0 && matches!(step.axis, Axis::Child);
        current = evaluate_step(document, step, &current, budget, predicates_used, first_absolute);
        if current.is_empty() {
            break;
        }
    }
    current
}

fn evaluate_step(
    document: &XmlDocumentView,
    step: &Step,
    context: &[ElementRef],
    budget: QueryBudget,
    predicates_used: &mut usize,
    first_absolute: bool,
) -> Vec<ElementRef> {
    if matches!(step.axis, Axis::Attribute) {
        return Vec::new();
    }

    let mut candidates = Vec::new();
    for reference in context {
        let next = if first_absolute {
            vec![*reference]
        } else {
            match step.axis {
            Axis::Child => document
                .element(reference.node_id)
                .map(|view| view.element_children())
                .unwrap_or_default(),
            Axis::Descendant => descendants(document, *reference),
            Axis::DescendantOrSelf => {
                let mut nodes = vec![*reference];
                nodes.extend(descendants(document, *reference));
                nodes
            }
            Axis::Attribute => Vec::new(),
            }
        };

        for child in next {
            let Some(node) = document.node(child.node_id) else {
                continue;
            };
            let Some(child_view) = document.element(child.node_id) else {
                continue;
            };
            if node_test_matches(node, &step.test)
                && predicates_match(document, &child_view, &step.predicates, budget, predicates_used)
            {
                candidates.push(child);
            }
        }
    }
    candidates
}

fn descendants(document: &XmlDocumentView, root: ElementRef) -> Vec<ElementRef> {
    let mut out = Vec::new();
    let mut stack = document
        .node(root.node_id)
        .map(|node| node.children.clone())
        .unwrap_or_default();
    while let Some(node_id) = stack.pop() {
        let reference = ElementRef {
            node_id,
            revision: document.revision(),
        };
        out.push(reference);
        if let Some(node) = document.node(node_id) {
            for child in node.children.iter().rev() {
                stack.push(*child);
            }
        }
    }
    out
}

fn node_test_matches(node: &XmlNodeData, test: &NodeTest) -> bool {
    match test {
        NodeTest::AnyElement => true,
        NodeTest::Name(qname) => node.matches_qname(qname),
        NodeTest::PrefixedWildcard(qname) => node.matches_qname(qname),
    }
}

fn predicates_match(
    document: &XmlDocumentView,
    view: &XmlElementView,
    predicates: &[Predicate],
    budget: QueryBudget,
    predicates_used: &mut usize,
) -> bool {
    for predicate in predicates {
        *predicates_used += 1;
        if *predicates_used > budget.max_predicates {
            return false;
        }
        match predicate {
            Predicate::Position(index) => {
                let siblings = sibling_elements(document, view.element_ref());
                let position = siblings.iter().position(|sibling| sibling.node_id == view.element_ref().node_id);
                if position != Some(index.saturating_sub(1) as usize) {
                    return false;
                }
            }
            Predicate::AttributeEquals(qname, value) => {
                let attr_name = match &qname.prefix {
                    Some(prefix) => format!("{}:{}", prefix, qname.local),
                    None => qname.local.clone(),
                };
                let name = oak_core::query::ExpandedName::local(attr_name);
                if view.attribute(&name).as_deref() != Some(value.as_str()) {
                    return false;
                }
            }
            Predicate::Unsupported(_) => return false,
        }
    }
    true
}

fn sibling_elements(document: &XmlDocumentView, reference: ElementRef) -> Vec<ElementRef> {
    let parent = document
        .element(reference.node_id)
        .and_then(|view| view.parent())
        .or_else(|| {
            if document.node(reference.node_id).is_some_and(|node| node.parent.is_none()) {
                None
            } else {
                None
            }
        });

    match parent {
        Some(parent_ref) => document
            .element(parent_ref.node_id)
            .map(|view| view.element_children())
            .unwrap_or_default(),
        None => root_elements(document),
    }
}
