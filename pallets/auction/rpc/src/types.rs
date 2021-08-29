use jsonrpc_core::serde::{Deserializer, Serializer};
use node_primitives::Balance;
use serde::de::{self, Visitor};
use std::fmt::Formatter;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Price(pub Balance);

impl serde::Serialize for Price {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: Serializer,
	{
		serializer.serialize_str(&self.0.to_string())
	}
}

impl<'de> serde::Deserialize<'de> for Price {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		deserializer.deserialize_str(PriceVisitor)
	}
}

struct PriceVisitor;

impl<'de> Visitor<'de> for PriceVisitor {
	type Value = Price;

	fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
		formatter.write_str("a string can parsed to u128")
	}

	fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
	where
		E: de::Error,
	{
		let value: u128 = s
			.parse()
			.map_err(|e| E::custom(format!("parse to u128 failed: {}", e)))?;
		Ok(Price(value))
	}
}
