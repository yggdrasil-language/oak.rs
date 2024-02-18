/// Keywords or soft keywords for the Valkyrie language.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ValkyrieKeywords {
    /// Declare a namespace in Valkyrie.
    ///
    /// ```v
    /// namespace path { ... }
    /// ```
    Namespace,
    /// Import a declaration from another namespace.
    ///
    /// ```v
    /// using package::module::path;
    /// ```
    Using,
    /// Declare a class.
    ///
    /// ```v
    /// class Point { x: f64, y: f64 }
    /// ```
    Class,
    /// Declare an abstract class or abstract method.
    ///
    /// ```v
    /// abstract class Shape { ... }
    /// ```
    ///
    /// ```v
    /// abstract micro method()
    /// ```
    Abstract,
    /// Declare a sealed class (restricted inheritance).
    ///
    /// Sealed classes restrict which classes can inherit from them.
    /// All subclasses must be declared in the same file as the sealed class.
    ///
    /// ```v
    /// sealed class Shape { ... }
    /// ```
    Sealed,
    /// Declare a final class or final method that cannot be inherited or overridden.
    ///
    /// ```v
    /// final class Constants { ... }
    /// ```
    ///
    /// ```v
    /// final micro get_value() -> i32 { ... }
    /// ```
    Final,
    /// Declare a structure (deprecated, use `structure` instead).
    ///
    /// ```v
    /// struct Point { x: f64, y: f64 }
    /// ```
    Struct,
    /// Declare a value type (immutable structure).
    ///
    /// ```v
    /// structure Point { x: f64, y: f64 }
    /// ```
    Structure,
    /// Declare a singleton.
    ///
    /// ```v
    /// singleton GlobalConfig { ... }
    /// ```
    Singleton,
    /// Declare a trait.
    ///
    /// ```v
    /// trait Show { micro show(self) }
    /// ```
    Trait,
    /// Declare an inherent or trait implementation block.
    ///
    /// ```v
    /// imply Owner { micro answer(self) -> i32 { return 1 } }
    /// ```
    Imply,
    /// Declare flags.
    ///
    /// ```v
    /// flags Permissions { Read, Write, Execute }
    /// ```
    Flags,
    /// Declare enums.
    ///
    /// ```v
    /// enums Priority { High = 3, Normal = 2, Low = 1 }
    /// ```
    Enums,
    /// Declare an enum (deprecated, use `unity` instead).
    ///
    /// ```v
    /// enum Priority { High, Normal, Low }
    /// ```
    Enum,
    /// Declare a unite (preferred alternative to enum).
    ///
    /// ```v
    /// unite Option<T> { Some(T), None }
    /// ```
    Unity,
    /// Declare a union.
    ///
    /// ```v
    /// union Option<T> { Some(T), None }
    /// ```
    Union,
    /// Declare a micro function.
    ///
    /// ```v
    /// micro add(x: i32, y: i32) -> i32 { x + y }
    /// ```
    Micro,
    /// Declare a mezzo function.
    ///
    /// ```v
    /// mezzo add(x: i32, y: i32) -> i32 { x + y }
    /// ```
    Mezzo,
    /// Declare a macro.
    ///
    /// ```v
    /// macro log(msg: string) { ... }
    /// ```
    Macro,
    /// Declare a widget.
    ///
    /// ```v
    /// widget Button { ... }
    /// ```
    Widget,
    /// Declare a variable.
    ///
    /// ```v
    /// let x = 42;
    /// ```
    Let,
    /// If expression.
    ///
    /// ```v
    /// if condition { ... } else { ... }
    /// ```
    If,
    /// Else clause.
    Else,
    /// Match expression.
    ///
    /// ```v
    /// match x { 1 => 2, _ => 0 }
    /// ```
    Match,
    /// Match case.
    Case,
    /// Match guard.
    ///
    /// ```v
    /// match x { y when y > 0 => y }
    /// ```
    When,
    /// Try expression.
    ///
    /// ```v
    /// try expression catch { pattern => expression }
    /// ```
    Try,
    /// Lambda expression.
    ///
    /// ```v
    /// lambda(x) { x + 1 }
    /// ```
    Lambda,
    /// Catch clause.
    Catch,
    /// While loop.
    ///
    /// ```v
    /// while true { ... }
    /// ```
    While,
    /// Loop.
    ///
    /// ```v
    /// loop { ... }
    /// loop i in list { ... }
    /// ```
    Loop,
    /// For loop.
    ///
    /// ```v
    /// for i in 0..10 { ... }
    /// ```
    For,
    /// Return from function.
    ///
    /// ```v
    /// return x;
    /// ```
    Return,
    /// Break from loop.
    ///
    /// ```v
    /// break @label value;
    /// ```
    Break,
    /// Continue to next iteration.
    ///
    /// ```v
    /// continue @label;
    /// ```
    Continue,
    /// Boolean true.
    True,
    /// Boolean false.
    False,
    /// Null value.
    Null,
    /// Yield value from generator.
    ///
    /// ```v
    /// yield x;
    /// ```
    Yield,
    /// In clause for loops.
    ///
    /// ```v
    /// for i in 0..10 { ... };
    /// ```
    In,
    /// Raise an error.
    ///
    /// ```v
    /// raise error;
    /// ```
    Raise,
    /// Effect declaration.
    ///
    /// ```v
    /// effect Console { micro log(msg: string) }
    /// ```
    Effect,
    /// Resume from an effect operation.
    ///
    /// ```v
    /// resume value;
    /// ```
    Resume,
    /// From keyword for yield from.
    ///
    /// ```v
    /// yield from list;
    /// ```
    From,
    /// Mutable variable.
    ///
    /// ```v
    /// let mut x = 1;
    /// ```
    Mut,
    /// Type check.
    ///
    /// ```v
    /// if x is T { ... }
    /// ```
    Is,
    /// Type pattern in match.
    ///
    /// ```v
    /// match x { type T => ... }
    /// ```
    ///
    /// ```v
    /// match x { type T => ... }
    /// ```
    Type,
    /// Type casting.
    ///
    /// ```v
    /// x as T
    /// ```
    As,
    /// Getter property.
    ///
    /// ```v
    /// get area(self) -> f64 { self.width * self.height }
    /// ```
    Get,
    /// Setter property.
    ///
    /// ```v
    /// set width(mut self, value: f64) { self.width = value }
    /// ```
    Set,
    /// Self type reference.
    ///
    /// Used in trait implementations and associated type paths.
    ///
    /// ```v
    /// trait Iterator {
    ///     type Item
    ///     micro next(self) -> Self::Item?
    /// }
    /// ```
    SelfType,
    /// Type alias declaration.
    ///
    /// ```v
    /// type Point = (f64, f64)
    /// ```
    TypeAlias,
    /// Implement a trait for a type.
    ///
    /// ```v
    /// impl Show for Point { ... }
    /// ```
    Impl,
    /// Where clause for generic constraints.
    ///
    /// ```v
    /// micro foo<T>(x: T) where T: Clone { ... }
    /// ```
    Where,
    /// Super reference for parent class constructor calls.
    ///
    /// ```v
    /// class Derived(Base) {
    ///     initiate(mut self, x: i32) {
    ///         super.initiate(x)  // Call parent constructor
    ///     }
    /// }
    /// ```
    Super,
    /// Override modifier for method overriding.
    ///
    /// ```v
    /// class Derived: Base {
    ///     override micro method(self) { ... }
    /// }
    /// ```
    Override,
    /// Virtual method declaration.
    ///
    /// ```v
    /// class Base {
    ///     virtual micro method(self) { ... }
    /// }
    /// ```
    Virtual,
    /// Readonly field modifier.
    ///
    /// ```v
    /// class Point {
    ///     readonly x: f64
    /// }
    /// ```
    Readonly,
    /// Initiate constructor method.
    ///
    /// ```v
    /// class Point {
    ///     initiate(mut self, x: f64, y: f64) {
    ///         self.x = x
    ///         self.y = y
    ///     }
    /// }
    /// ```
    Initiate,
    /// Declare a shader.
    ///
    /// ```gs
    /// shader StandardShader by PBR {
    ///     // shader definition
    /// }
    /// ```
    Shader,
    /// By keyword for shader definition.
    ///
    /// ```gs
    /// shader StandardShader by PBR {
    ///     // shader definition
    /// }
    /// ```
    By,
    /// Declare a component for ECS (Entity Component System).
    ///
    /// ```valkyrie
    /// component Position {
    ///     x: f32,
    ///     y: f32
    /// }
    /// ```
    Component,
    /// Declare a system for ECS (Entity Component System).
    ///
    /// ```valkyrie
    /// system MovementSystem {
    ///     micro execute(world: World): Result<()> {
    ///         // system logic
    ///     }
    /// }
    /// ```
    System,
    /// Declare events within a component.
    ///
    /// ```valkyrie
    /// component Player {
    ///     events on_health_change: micro(current: i32, max: i32) -> (),
    ///     events on_death: micro() -> ()
    /// }
    /// ```
    Events,
}
