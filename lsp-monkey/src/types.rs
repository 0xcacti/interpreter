use serde::de::{self, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;
use std::marker::PhantomData;

pub type Integer = i32;
pub type UInteger = u32;
pub struct Decimal(f32);
pub type LSPAny = serde_json::Value;
pub type LSPObject = serde_json::Map<String, LSPAny>;
pub type LSPArray = Vec<LSPAny>;

impl Decimal {
    pub fn new(value: f32) -> Option<Self> {
        if value >= 0.0 && value <= 1.0 {
            Some(Decimal(value))
        } else {
            None
        }
    }

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
