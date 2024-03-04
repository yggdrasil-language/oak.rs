use super::*;

/// A component declaration for ECS (Entity Component System).
///
/// ```valkyrie
/// component Position {
///     x: f32,
///     y: f32
/// }
///
/// component Player {
///     name: String,
///     health: i32,
///     events on_health_change: micro(current: i32, max: i32) -> (),
///     events on_death: micro() -> ()
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ComponentDeclaration {
    /// Annotations applied to the component.
    pub annotations: Vec<Attribute>,
    /// The component name.
    pub name: Identifier,
    /// Fields declared in the component.
    pub fields: Vec<FieldDeclaration>,
    /// Events declared in the component.
    pub events: Vec<EventDeclaration>,
    /// The source code span.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Span,
}

/// A system declaration for ECS (Entity Component System).
///
/// ```valkyrie
/// system MovementSystem {
///     micro execute(world: World): Result<()> {
///         // system logic
///         return Ok()
///     }
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SystemDeclaration {
    /// Annotations applied to the system.
    pub annotations: Vec<Attribute>,
    /// The system name.
    pub name: Identifier,
    /// Methods declared in the system (typically just execute).
    pub methods: Vec<MethodDeclaration>,
    /// The source code span.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Span,
}

/// An event declaration within a component.
///
/// ```valkyrie
/// event  on_click: Action<()>,
/// events on_death: micro() -> ()
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct EventDeclaration {
    /// Annotations applied to the event.
    pub annotations: Vec<Attribute>,
    /// The event kind.
    pub kind: EventKind,
    /// The event name.
    pub name: Identifier,
    /// The event signature (usually function type).
    pub signature: TypeExpression,
    /// The source code span.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Span,
}

/// The kind of event declared.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum EventKind {
    /// A single event, aka. `callback`.
    #[default]
    Event,
    /// A broadcast event.
    Events,
}
