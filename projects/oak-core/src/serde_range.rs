//! Serde support for `core::range::Range`.

#[cfg(feature = "serde")]
use core::range::Range;

#[cfg(feature = "serde")]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
struct RangeDef<T> {
    start: T,
    end: T,
}

#[cfg(feature = "serde")]
#[derive(serde::Serialize)]
struct RangeDefRef<'a, T> {
    start: &'a T,
    end: &'a T,
}

/// Serializes a `Range<T>`.
#[cfg(feature = "serde")]
pub fn serialize<S, T>(value: &Range<T>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
    T: serde::Serialize,
{
    serde::Serialize::serialize(&RangeDefRef { start: &value.start, end: &value.end }, serializer)
}

/// Deserializes a `Range<T>`.
#[cfg(feature = "serde")]
pub fn deserialize<'de, D, T>(deserializer: D) -> Result<Range<T>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: serde::Deserialize<'de>,
{
    let def = <RangeDef<T> as serde::Deserialize>::deserialize(deserializer)?;
    Ok(Range { start: def.start, end: def.end })
}

/// Serde support for `Vec<Range<T>>`.
#[cfg(feature = "serde")]
pub mod vec {
    use super::*;

    /// Serializes a slice of `Range<T>`.
    pub fn serialize<S, T>(value: &[Range<T>], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
        T: serde::Serialize,
    {
        use serde::ser::SerializeSeq;
        let mut seq = serializer.serialize_seq(Some(value.len()))?;
        for range in value {
            seq.serialize_element(&RangeDefRef { start: &range.start, end: &range.end })?;
        }
        seq.end()
    }

    /// Deserializes a `Vec<Range<T>>`.
    pub fn deserialize<'de, D, T>(deserializer: D) -> Result<Vec<Range<T>>, D::Error>
    where
        D: serde::Deserializer<'de>,
        T: serde::Deserialize<'de>,
    {
        let defs = <Vec<RangeDef<T>> as serde::Deserialize>::deserialize(deserializer)?;
        Ok(defs.into_iter().map(|def| Range { start: def.start, end: def.end }).collect())
    }
}

/// Serde support for `Option<Range<T>>`.
#[cfg(feature = "serde")]
pub mod option {
    use super::*;

    /// Serializes an `Option<Range<T>>`.
    pub fn serialize<S, T>(value: &Option<Range<T>>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
        T: serde::Serialize,
    {
        match value {
            Some(range) => serializer.serialize_some(&RangeDefRef { start: &range.start, end: &range.end }),
            None => serializer.serialize_none(),
        }
    }

    /// Deserializes an `Option<Range<T>>`.
    pub fn deserialize<'de, D, T>(deserializer: D) -> Result<Option<Range<T>>, D::Error>
    where
        D: serde::Deserializer<'de>,
        T: serde::Deserialize<'de>,
    {
        let opt = <Option<RangeDef<T>> as serde::Deserialize>::deserialize(deserializer)?;
        Ok(opt.map(|def| Range { start: def.start, end: def.end }))
    }
}
