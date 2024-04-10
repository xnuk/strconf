use std::collections::HashMap;
use std::fmt;
use std::hash::{BuildHasher, RandomState};
use std::marker::PhantomData;

use serde::de::{self, Deserialize, Deserializer, Visitor};
use serde::ser::{Serialize, Serializer};

type Dict<Hasher> = HashMap<String, Value<Hasher>, Hasher>;

#[derive(Debug)]
pub enum Value<Hasher> {
	String(String),
	List(Vec<Value<Hasher>>),
	Dict(Dict<Hasher>),
}

#[derive(Debug)]
#[repr(transparent)]
pub struct Config<Hasher = RandomState>(Dict<Hasher>);

impl<H> From<Config<H>> for Value<H> {
	fn from(value: Config<H>) -> Self {
		Value::Dict(value.0)
	}
}

impl<H> From<String> for Value<H> {
	fn from(value: String) -> Self {
		Value::String(value)
	}
}

impl<H> From<&str> for Value<H> {
	fn from(value: &str) -> Self {
		Value::String(value.to_owned())
	}
}

impl<H, T: Into<Value<H>>> From<Vec<T>> for Value<H> {
	fn from(value: Vec<T>) -> Self {
		Value::List(value.into_iter().map(T::into).collect())
	}
}

impl<H: BuildHasher + Default, T: Into<Value<H>>> From<HashMap<String, T, H>>
	for Value<H>
{
	fn from(value: HashMap<String, T, H>) -> Self {
		Value::Dict(value.into_iter().map(|(k, v)| (k, v.into())).collect())
	}
}

impl<H: BuildHasher + Default, T: Into<Value<H>>> From<HashMap<String, T, H>>
	for Config<H>
{
	fn from(value: HashMap<String, T, H>) -> Self {
		Config(value.into_iter().map(|(k, v)| (k, v.into())).collect())
	}
}

#[derive(Default)]
struct ValueVisit<H> {
	_hasher: PhantomData<H>,
}

impl<'de, H: BuildHasher + Default> Visitor<'de> for ValueVisit<H> {
	type Value = Value<H>;

	fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
		f.write_str("list, dict, or a string")
	}

	fn visit_string<E: de::Error>(self, v: String) -> Result<Self::Value, E> {
		Ok(v.into())
	}

	fn visit_str<E: de::Error>(self, v: &str) -> Result<Self::Value, E> {
		Ok(v.into())
	}

	fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
	where
		A: de::MapAccess<'de>,
	{
		let mut entries = match map.size_hint() {
			Some(hint) => HashMap::with_capacity_and_hasher(hint, H::default()),
			None => HashMap::default(),
		};

		while let Some((key, value)) = map.next_entry::<String, _>()? {
			if entries.insert(key.clone(), value).is_some() {
				return Err(de::Error::custom(format!(
					"Duplicated key {key:?} is not allowed"
				)));
			}
		}

		Ok(Value::Dict(entries))
	}

	fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
	where
		A: de::SeqAccess<'de>,
	{
		let mut arr = match seq.size_hint() {
			Some(hint) => Vec::with_capacity(hint),
			None => Vec::new(),
		};

		while let Some(item) = seq.next_element()? {
			arr.push(item)
		}

		Ok(Value::List(arr))
	}
}

impl<'de, H: BuildHasher + Default> Deserialize<'de> for Value<H> {
	fn deserialize<Der>(der: Der) -> Result<Self, Der::Error>
	where
		Der: Deserializer<'de>,
	{
		der.deserialize_any(ValueVisit::default())
	}
}

impl<'de, H: BuildHasher + Default> Deserialize<'de> for Config<H> {
	fn deserialize<Der>(der: Der) -> Result<Self, Der::Error>
	where
		Der: Deserializer<'de>,
	{
		match der.deserialize_map(ValueVisit::default())? {
			Value::Dict(dict) => Ok(Config(dict)),
			Value::List(_) => Err(de::Error::custom("Expected map, got list")),
			Value::String(_) => {
				Err(de::Error::custom("Expected map, got string"))
			}
		}
	}
}

impl<H> Serialize for Value<H> {
	fn serialize<Ser: Serializer>(
		&self,
		ser: Ser,
	) -> Result<Ser::Ok, Ser::Error> {
		match self {
			Value::Dict(dict) => dict.serialize(ser),
			Value::List(list) => list.serialize(ser),
			Value::String(s) => s.serialize(ser),
		}
	}
}

impl<H> Serialize for Config<H> {
	fn serialize<Ser: Serializer>(
		&self,
		ser: Ser,
	) -> Result<Ser::Ok, Ser::Error> {
		self.0.serialize(ser)
	}
}
