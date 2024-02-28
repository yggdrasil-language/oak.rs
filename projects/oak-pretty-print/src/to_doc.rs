use crate::Document;
use alloc::{boxed::Box, string::String, vec::Vec};

/// A trait for types that can be converted to a document for pretty printing.
///
/// This trait is used to define how types should be formatted as documents.
/// It supports formatting parameters through the associated `Params` type,
/// which defaults to `()` if not specified.
///
/// # Example
/// ```rust
/// use oak_pretty_print::{AsDocument, Doc};
///
/// // Define a type with custom formatting parameters
/// struct MyType {
///     value: i32,
/// }
///
/// // Define formatting parameters for MyType
/// #[derive(Default)]
/// struct MyTypeParams {
///     indent: usize,
/// }
///
/// // Implement AsDocument for MyType with custom parameters
/// impl AsDocument for MyType {
///     type Params = MyTypeParams;
///
///     fn as_document(&self, config: &Self::Config) -> Doc<'_> {
///         // Use params to customize formatting
///         Doc::text(format!("MyType({}) with indent {}", self.value, params.indent))
///     }
/// }
///
/// // Create an instance of MyType
/// let my_type = MyType { value: 42 };
///
/// // Format it with custom parameters
/// let params = MyTypeParams { indent: 2 };
/// let doc = my_type.as_document(&params);
/// ```
pub trait AsDocument {
    /// The type of parameters used for formatting.
    /// Defaults to `()` if not specified.
    type Config = ();

    /// Converts this type to a document for pretty printing.
    ///
    /// # Parameters
    /// - `config`: Formatting parameters specific to this type.
    ///
    /// # Returns
    /// A `Document` representing the formatted type.
    fn as_document(&self, config: &Self::Config) -> Document<'_>;
}

/// A trait for types that can be converted to a document value, potentially consuming the input.
pub trait ToDocument<'a> {
    /// Converts this type to a document value.
    fn to_document(self) -> Document<'a>;
}

impl AsDocument for String {
    fn as_document(&self, _config: &Self::Config) -> Document<'_> {
        Document::Text(self.as_str().into())
    }
}

impl AsDocument for str {
    fn as_document(&self, _config: &Self::Config) -> Document<'_> {
        Document::Text(self.into())
    }
}

impl<'a> AsDocument for Document<'a> {
    fn as_document(&self, _config: &Self::Config) -> Document<'_> {
        self.clone()
    }
}

impl<T: AsDocument> AsDocument for Vec<T> {
    type Config = T::Config;

    fn as_document(&self, config: &Self::Config) -> Document<'_> {
        Document::Concat(self.iter().map(|t| t.as_document(config)).collect())
    }
}

impl<T: AsDocument> AsDocument for Option<T> {
    type Config = T::Config;

    fn as_document(&self, config: &Self::Config) -> Document<'_> {
        match self {
            Some(t) => t.as_document(config),
            None => Document::Nil,
        }
    }
}

impl<T: AsDocument + ?Sized> AsDocument for &T {
    type Config = T::Config;

    fn as_document(&self, config: &Self::Config) -> Document<'_> {
        (**self).as_document(config)
    }
}

impl<T: AsDocument + ?Sized> AsDocument for Box<T> {
    type Config = T::Config;

    fn as_document(&self, config: &Self::Config) -> Document<'_> {
        self.as_ref().as_document(config)
    }
}

impl<'a> ToDocument<'a> for Document<'a> {
    fn to_document(self) -> Document<'a> {
        self
    }
}

impl<'a, T: AsDocument + ?Sized> ToDocument<'a> for &'a T
where
    T::Config: Default,
{
    fn to_document(self) -> Document<'a> {
        self.as_document(&T::Config::default())
    }
}

impl<'a> ToDocument<'a> for String {
    fn to_document(self) -> Document<'a> {
        Document::Text(self.into())
    }
}
