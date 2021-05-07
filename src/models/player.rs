#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::Error;

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct PlayerInfo {
    pub name: String,
    pub guid: String,
    pub team_id: usize,
    pub squad_id: usize,
    pub kills: i32,
    pub deaths: i32,
    pub score: u32,
    pub rank: i16,
    pub ping: u16,
    pub kind: PlayerKind,
}

impl PlayerInfo {
    pub fn new(name: String, guid: String) -> PlayerInfo {
        PlayerInfo {
            name,
            guid,
            team_id: 0,
            squad_id: 0,
            kills: 0,
            deaths: 0,
            score: 0,
            rank: 0,
            ping: 0,
            kind: PlayerKind::Player,
        }
    }
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub enum PlayerKind {
    Player,
    Spectator,
    Commander,
    MobileCommander,
}

impl std::str::FromStr for PlayerKind {
    type Err = Error;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input {
            "0" => Ok(PlayerKind::Player),
            "1" => Ok(PlayerKind::Spectator),
            "2" => Ok(PlayerKind::Commander),
            "3" => Ok(PlayerKind::MobileCommander),
            other => Err(Error::new_parse(format!("invaldid player kind: {}", other))),
        }
    }
}
