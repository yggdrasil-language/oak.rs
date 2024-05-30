use oak_core::Range;
use oak_uml::model::UmlElement;

/// Represents the root of a PlantUML document.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PlantUmlRoot {
    /// The UML elements contained in the document.
    pub elements: Vec<UmlElement>,
    /// The text range covered by this root node.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Range<usize>,
}

impl PlantUmlRoot {
    /// Creates a new empty PlantUML document.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a UML element to the document.
    pub fn with_element(mut self, element: impl Into<UmlElement>) -> Self {
        self.elements.push(element.into());
        self
    }

    /// Sets the span of the document.
    pub fn with_span(mut self, span: Range<usize>) -> Self {
        self.span = span;
        self
    }
}

impl From<oak_uml::model::UmlDiagram> for PlantUmlRoot {
    fn from(diagram: oak_uml::model::UmlDiagram) -> Self {
        Self { elements: diagram.elements, span: Default::default() }
    }
}

impl std::fmt::Display for PlantUmlRoot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "@startuml")?;
        for element in &self.elements {
            match element {
                UmlElement::Class(c) => {
                    writeln!(f, "class {} {{", c.name)?;
                    for member in &c.members {
                        let symbol = match member.kind {
                            oak_uml::model::MemberKind::Field => "",
                            oak_uml::model::MemberKind::Method => "()",
                        };
                        writeln!(f, "    {}{}", member.name, symbol)?;
                    }
                    writeln!(f, "}}")?;
                }
                UmlElement::Interface(i) => {
                    writeln!(f, "interface {} {{", i.name)?;
                    for member in &i.members {
                        writeln!(f, "    {}()", member.name)?;
                    }
                    writeln!(f, "}}")?;
                }
                UmlElement::Relation(r) => {
                    let arrow = match r.kind {
                        oak_uml::model::RelationKind::Inheritance => "<|--",
                        oak_uml::model::RelationKind::Association => "-->",
                        oak_uml::model::RelationKind::Composition => "*--",
                        oak_uml::model::RelationKind::Aggregation => "o--",
                        oak_uml::model::RelationKind::Dependency => "..>",
                    };
                    if let Some(label) = &r.label {
                        writeln!(f, "{} {} {} : {}", r.source, arrow, r.target, label)?;
                    }
                    else {
                        writeln!(f, "{} {} {}", r.source, arrow, r.target)?;
                    }
                }
            }
        }
        writeln!(f, "@enduml")?;
        Ok(())
    }
}
