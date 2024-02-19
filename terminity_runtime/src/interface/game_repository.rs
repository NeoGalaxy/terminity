use serde::{
	de::{Unexpected, Visitor},
	Deserialize, Serialize,
};
use std::path::PathBuf;

type GameRepoLatest = GameRepoV1;

pub struct Tag<const V: u32>;

impl GameRepo {
	pub fn open(self) -> GameRepoLatest {
		match self {
			GameRepo::V1(_, data) => data,
		}
	}
}

impl<const V: u32> Serialize for Tag<V> {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		serializer.serialize_u32(V)
	}
}

impl<'de, const V: u32> Deserialize<'de> for Tag<V> {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: serde::Deserializer<'de>,
	{
		struct Visit<const V: u32>;

		impl<const V: u32> Visitor<'_> for Visit<V> {
			type Value = Tag<V>;

			fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
				write!(f, "the version tag {V}")
			}

			fn visit_u32<E>(self, v: u32) -> Result<Self::Value, E>
			where
				E: serde::de::Error,
			{
				if v == V {
					Ok(Tag)
				} else {
					Err(E::invalid_value(Unexpected::Unsigned(v as u64), &Self))
				}
			}
		}
		deserializer.deserialize_u32(Visit)
	}
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)] // Force to a stable tag, agnostic to the variant name
pub enum GameRepo {
	V1(Tag<1>, GameRepoV1),
}

#[derive(Serialize, Deserialize)]
pub struct GameDataV1 {
	pub subpath: PathBuf,
}

#[derive(Serialize, Deserialize)]
pub struct GameRepoV1 {
	pub root_path: PathBuf,
	pub games: Vec<GameDataV1>,
}
