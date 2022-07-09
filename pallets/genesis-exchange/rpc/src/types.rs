use node_primitives::Balance;
use serde::de::{self, Visitor};
use serde::{Deserializer, Serializer};
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

#[cfg(test)]
mod tests {
	use crate::types::Price;
	use rmp_serde::{Deserializer, Serializer};
	use serde::{Deserialize, Serialize};
	use std::io::Cursor;

	#[test]
	fn price_serialize_deserialize_works() {
		let p1 = Price(123456789);
		let mut buf = Vec::new();
		p1.serialize(&mut Serializer::new(&mut buf).with_struct_map())
			.unwrap();

		let mut de = Deserializer::new(Cursor::new(buf));
		let p2: Price = Deserialize::deserialize(&mut de).unwrap();

		assert_eq!(p1, p2);
	}
}
