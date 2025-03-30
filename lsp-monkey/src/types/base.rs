use serde::de::{self, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;
use std::marker::PhantomData;

/// Defines an integer number in the range of -2^31 to 2^31 - 1.
pub type Integer = i32;

/// Defines an unsigned integer number in the range of 0 to 2^31 - 1.
pub type UInteger = u32;

/// Represents a decimal number, typically in the range [0, 1].
///
/// This type enforces range constraints during deserialization to
/// ensure valid decimal values according to the LSP specification.
pub struct Decimal(f32);

/// Represents any valid LSP value.
///
/// This can be an object, array, string, number, boolean, or null,
/// following the JSON data model used by the Language Server Protocol.
pub type LSPAny = serde_json::Value;

/// Represents a JSON object with string keys and arbitrary LSP values.
pub type LSPObject = serde_json::Map<String, LSPAny>;

/// Represents a JSON array containing arbitrary LSP values.
pub type LSPArray = Vec<LSPAny>;

impl Decimal {
    /// Creates a new Decimal if the value is within the valid range [0, 1].
    ///
    /// Returns None if the value is outside the valid range.
    pub fn new(value: f32) -> Option<Self> {
        if value >= 0.0 && value <= 1.0 {
            Some(Decimal(value))
        } else {
            None
        }
    }

    /// Returns the underlying f32 value.
    pub fn value(&self) -> f32 {
        self.0
    }
}

impl Serialize for Decimal {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Decimal {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct DecimalVisitor(PhantomData<fn() -> Decimal>);
        impl<'de> Visitor<'de> for DecimalVisitor {
            type Value = Decimal;
            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a decimal number between 0 and 1")
            }
            fn visit_f32<E>(self, value: f32) -> Result<Decimal, E>
            where
                E: de::Error,
            {
                if value >= 0.0 && value <= 1.0 {
                    Ok(Decimal(value))
                } else {
                    Err(E::custom(format!("decimal out of range: {}", value)))
                }
            }
        }
        deserializer.deserialize_f64(DecimalVisitor(PhantomData))
    }
}
