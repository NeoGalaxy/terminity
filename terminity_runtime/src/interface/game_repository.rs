use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub type GameRepoLatest = GameRepoV1;
pub type GameDataLatest = GameDataV1;

impl GameRepo {
	pub fn open(self) -> GameRepoLatest {
		match self {
			GameRepo::V1(data) => data,
		}
	}
}

#[derive(Debug, Serialize, Deserialize)]
pub enum GameRepo {
	V1(GameRepoV1),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameDataV1 {
	pub subpath: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameRepoV1 {
	pub root_path: PathBuf,
	pub games: Vec<GameDataV1>,
}

impl GameRepoLatest {
	pub fn close(self) -> GameRepo {
		GameRepo::V1(self)
	}
}
