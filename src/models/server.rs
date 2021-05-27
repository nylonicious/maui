use std::net::SocketAddr;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::Error;

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct ServerInfo {
    pub name: String,
    pub player_count: usize,
    pub max_player_count: usize,
    pub game_mode: String,
    pub map: String,
    pub rounds_played: usize,
    pub rounds_total: usize,
    pub num_of_teams: usize,
    pub team_scores: Vec<f32>,
    pub target_score: u32,
    pub online_state: String,
    pub ranked: bool,
    pub punkbuster: bool,
    pub game_password: bool,
    pub uptime: u64,
    pub round_time: u64,
    pub addr: SocketAddr,
    pub punkbuster_version: String,
    pub join_queue: bool,
    pub region: String,
    pub ping_site: String,
    pub country: String,
    pub blaze_player_count: usize,
    pub blaze_game_state: String,
}

impl ServerInfo {
    pub(crate) fn from_words(words: Vec<String>) -> Result<ServerInfo, Error> {
        let mut words = words.into_iter();

        let name = next!(words);
        let player_count = next_parse!(words);
        let max_player_count = next_parse!(words);
        let game_mode = next!(words);
        let map = next!(words);
        let rounds_played = next_parse!(words);
        let rounds_total = next_parse!(words);
        let num_of_teams: usize = next_parse!(words);

        let mut team_scores = Vec::with_capacity(num_of_teams);
        for _ in 0..num_of_teams {
            let team_score = next_parse!(words);
            team_scores.push(team_score);
        }

        Ok(ServerInfo {
            name,
            player_count,
            max_player_count,
            game_mode,
            map,
            rounds_played,
            rounds_total,
            num_of_teams,
            team_scores,
            target_score: next_parse!(words),
            online_state: next!(words),
            ranked: next_parse!(words),
            punkbuster: next_parse!(words),
            game_password: next_parse!(words),
            uptime: next_parse!(words),
            round_time: next_parse!(words),
            addr: next_parse!(words),
            punkbuster_version: next!(words),
            join_queue: next_parse!(words),
            region: next!(words),
            ping_site: next!(words),
            country: next!(words),
            blaze_player_count: next_parse!(words),
            blaze_game_state: next!(words),
        })
    }
}
