//! Model definitions for Unified Modeling Language (UML).
//!
//! This module provides the core data structures for representing UML elements,
//! including classes, interfaces, and their relationships.

/// Represents a top-level UML element.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum UmlElement {
    /// A class definition.
    Class(Class),
    /// An interface definition.
    Interface(Interface),
    /// A relationship between UML elements.
    Relation(Relation),
}

/// Represents a UML class.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Class {
    /// The name of the class.
    pub name: String,
    /// The members (fields and methods) of the class.
    pub members: Vec<Member>,
}

impl Class {
    /// Creates a new class with the given name.
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into(), members: Vec::new() }
    }

    /// Adds a member to the class.
    pub fn with_member(mut self, member: Member) -> Self {
        self.members.push(member);
        self
    }

    /// Adds a field to the class.
    pub fn with_field(self, name: impl Into<String>) -> Self {
        self.with_member(Member::new_field(name))
    }

    /// Adds a method to the class.
    pub fn with_method(self, name: impl Into<String>) -> Self {
        self.with_member(Member::new_method(name))
    }
}

/// Represents a UML interface.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Interface {
    /// The name of the interface.
    pub name: String,
    /// The members (methods) of the interface.
    pub members: Vec<Member>,
}

impl Interface {
    /// Creates a new interface with the given name.
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into(), members: Vec::new() }
    }

    /// Adds a member (method) to the interface.
    pub fn with_method(mut self, name: impl Into<String>) -> Self {
        self.members.push(Member::new_method(name));
        self
    }
}

/// Represents a member of a class or interface.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Member {
    /// The name of the member.
    pub name: String,
    /// The kind of the member (field or method).
    pub kind: MemberKind,
}

impl Member {
    /// Creates a new field member.
    pub fn new_field(name: impl Into<String>) -> Self {
        Self { name: name.into(), kind: MemberKind::Field }
    }

    /// Creates a new method member.
    pub fn new_method(name: impl Into<String>) -> Self {
        Self { name: name.into(), kind: MemberKind::Method }
    }
}

/// The kind of a UML member.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum MemberKind {
    /// A data field.
    Field,
    /// A behavior or method.
    Method,
}

/// Represents a relationship between UML elements.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Relation {
    /// The source element name.
    pub source: String,
    /// The target element name.
    pub target: String,
    /// The type of relationship.
    pub kind: RelationKind,
    /// An optional label for the relationship.
    pub label: Option<String>,
}

impl Relation {
    /// Creates a new relationship of the given kind.
    pub fn new(source: impl Into<String>, target: impl Into<String>, kind: RelationKind) -> Self {
        Self { source: source.into(), target: target.into(), kind, label: None }
    }

    /// Sets the label for the relationship.
    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Creates a new inheritance relationship.
    pub fn inheritance(source: impl Into<String>, target: impl Into<String>) -> Self {
        Self::new(source, target, RelationKind::Inheritance)
    }

    /// Creates a new association relationship.
    pub fn association(source: impl Into<String>, target: impl Into<String>) -> Self {
        Self::new(source, target, RelationKind::Association)
    }
}

/// A collection of UML elements forming a diagram.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct UmlDiagram {
    /// The elements in the diagram.
    pub elements: Vec<UmlElement>,
}

impl UmlDiagram {
    /// Creates a new empty UML diagram.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds an element to the diagram.
    pub fn with_element(mut self, element: impl Into<UmlElement>) -> Self {
        self.elements.push(element.into());
        self
    }

    /// Adds a class to the diagram.
    pub fn class(self, name: impl Into<String>) -> Class {
        Class::new(name)
    }

    /// Adds a class with fields/methods to the diagram.
    pub fn add_class(self, class: Class) -> Self {
        self.with_element(class)
    }

    /// Adds an interface to the diagram.
    pub fn interface(self, name: impl Into<String>) -> Interface {
        Interface::new(name)
    }

    /// Adds an interface with methods to the diagram.
    pub fn add_interface(self, interface: Interface) -> Self {
        self.with_element(interface)
    }

    /// Adds an inheritance relationship.
    pub fn inheritance(self, source: impl Into<String>, target: impl Into<String>) -> Self {
        self.with_element(Relation::inheritance(source, target))
    }

    /// Adds an association relationship.
    pub fn association(self, source: impl Into<String>, target: impl Into<String>, label: impl Into<String>) -> Self {
        self.with_element(Relation::association(source, target).with_label(label))
    }
}

impl From<Class> for UmlElement {
    fn from(c: Class) -> Self {
        UmlElement::Class(c)
    }
}

impl From<Interface> for UmlElement {
    fn from(i: Interface) -> Self {
        UmlElement::Interface(i)
    }
}

impl From<Relation> for UmlElement {
    fn from(r: Relation) -> Self {
        UmlElement::Relation(r)
    }
}

/// The type of relationship between UML elements.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum RelationKind {
    /// Inheritance relationship (is-a).
    Inheritance,
    /// General association between elements.
    Association,
    /// Strong ownership relationship (part-of).
    Composition,
    /// Weak ownership relationship (has-a).
    Aggregation,
    /// A dependency relationship between elements.
    Dependency,
}
