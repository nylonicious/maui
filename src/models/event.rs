#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::Error;

use super::{PlayerInfo, Subset};

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub enum Event {
    PlayerOnJoin {
        name: String,
        guid: String,
    },
    PlayerOnAuthenticated {
        name: String,
    },
    PlayerOnDisconnect {
        name: String,
        reason: String,
    },
    PlayerOnLeave(PlayerInfo),
    PlayerOnSpawn {
        name: String,
        team_id: usize,
    },
    PlayerOnKill {
        killer: String,
        victim: String,
        weapon: String,
        headshot: bool,
    },
    PlayerOnChat {
        source: String,
        message: String,
        subset: Subset,
    },
    PlayerOnSquadChange {
        name: String,
        team_id: usize,
        squad_id: usize,
    },
    PlayerOnTeamChange {
        name: String,
        team_id: usize,
        squad_id: usize,
    },
    PunkBusterOnMessage {
        message: String,
    },
    ServerOnRoundOver {
        team_id: usize,
    },
    ServerOnRoundOverPlayers(Vec<PlayerInfo>),
    ServerOnRoundOverTeamScores {
        num_of_teams: usize,
        team_scores: Vec<f32>,
        target_score: u32,
    },
    ServerOnLevelLoaded {
        map: String,
        gamemode: String,
        rounds_played: usize,
        rounds_total: usize,
    },
    ServerOnMaxPlayerCountChange {
        count: usize,
    },
}

impl Event {
    pub(crate) fn from_words(words: Vec<String>) -> Result<Event, Error> {
        let mut words = words.into_iter();

        match next!(words).as_ref() {
            "player.onJoin" => Ok(Event::PlayerOnJoin {
                name: next!(words),
                guid: next!(words),
            }),
            "player.onAuthenticated" => Ok(Event::PlayerOnAuthenticated { name: next!(words) }),
            "player.onDisconnect" => Ok(Event::PlayerOnDisconnect {
                name: next!(words),
                reason: next!(words),
            }),
            "player.onLeave" => {
                next!(words);
                let offset: usize = next!(words).parse().map_err(Error::new_parse)?;
                words.nth(offset);

                Ok(Event::PlayerOnLeave(PlayerInfo {
                    name: next!(words),
                    guid: next!(words),
                    team_id: next!(words).parse().map_err(Error::new_parse)?,
                    squad_id: next!(words).parse().map_err(Error::new_parse)?,
                    kills: next!(words).parse().map_err(Error::new_parse)?,
                    deaths: next!(words).parse().map_err(Error::new_parse)?,
                    score: next!(words).parse().map_err(Error::new_parse)?,
                    rank: next!(words).parse().map_err(Error::new_parse)?,
                    ping: next!(words).parse().map_err(Error::new_parse)?,
                    kind: next!(words).parse().map_err(Error::new_parse)?,
                }))
            }
            "player.onSpawn" => Ok(Event::PlayerOnSpawn {
                name: next!(words),
                team_id: next!(words).parse().map_err(Error::new_parse)?,
            }),
            "player.onKill" => Ok(Event::PlayerOnKill {
                killer: next!(words),
                victim: next!(words),
                weapon: next!(words),
                headshot: next!(words).parse().map_err(Error::new_parse)?,
            }),
            "player.onChat" => {
                let source = next!(words);
                let message = next!(words);

                let subset = match next!(words).as_ref() {
                    "all" => Subset::All,
                    "team" => Subset::Team {
                        team_id: next!(words).parse().map_err(Error::new_parse)?,
                    },
                    "squad" => Subset::Squad {
                        team_id: next!(words).parse().map_err(Error::new_parse)?,
                        squad_id: next!(words).parse().map_err(Error::new_parse)?,
                    },
                    "player" => Subset::Player { name: next!(words) },
                    other => {
                        return Err(Error::new_parse(format!(
                            "invalid player subset: {}",
                            other
                        )))
                    }
                };

                Ok(Event::PlayerOnChat {
                    source,
                    message,
                    subset,
                })
            }
            "player.onSquadChange" => Ok(Event::PlayerOnSquadChange {
                name: next!(words),
                team_id: next!(words).parse().map_err(Error::new_parse)?,
                squad_id: next!(words).parse().map_err(Error::new_parse)?,
            }),
            "player.onTeamChange" => Ok(Event::PlayerOnTeamChange {
                name: next!(words),
                team_id: next!(words).parse().map_err(Error::new_parse)?,
                squad_id: next!(words).parse().map_err(Error::new_parse)?,
            }),
            "punkBuster.onMessage" => Ok(Event::PunkBusterOnMessage {
                message: next!(words),
            }),
            "server.onRoundOverPlayers" => {
                let offset: usize = next!(words).parse().map_err(Error::new_parse)?;
                words.nth(offset - 1);

                let num_of_players: usize = next!(words).parse().map_err(Error::new_parse)?;
                let mut players = Vec::with_capacity(num_of_players);
                for _ in 0..num_of_players {
                    players.push(PlayerInfo {
                        name: next!(words),
                        guid: next!(words),
                        team_id: next!(words).parse().map_err(Error::new_parse)?,
                        squad_id: next!(words).parse().map_err(Error::new_parse)?,
                        kills: next!(words).parse().map_err(Error::new_parse)?,
                        deaths: next!(words).parse().map_err(Error::new_parse)?,
                        score: next!(words).parse().map_err(Error::new_parse)?,
                        rank: next!(words).parse().map_err(Error::new_parse)?,
                        ping: next!(words).parse().map_err(Error::new_parse)?,
                        kind: next!(words).parse().map_err(Error::new_parse)?,
                    })
                }

                Ok(Event::ServerOnRoundOverPlayers(players))
            }
            "server.onRoundOverTeamScores" => {
                let num_of_teams: usize = next!(words).parse().map_err(Error::new_parse)?;

                let mut team_scores = Vec::with_capacity(num_of_teams);
                for _ in 0..num_of_teams {
                    let team_score = next!(words).parse().map_err(Error::new_parse)?;
                    team_scores.push(team_score);
                }

                Ok(Event::ServerOnRoundOverTeamScores {
                    num_of_teams,
                    team_scores,
                    target_score: next!(words).parse().map_err(Error::new_parse)?,
                })
            }
            "server.onLevelLoaded" => Ok(Event::ServerOnLevelLoaded {
                map: next!(words),
                gamemode: next!(words),
                rounds_played: next!(words).parse().map_err(Error::new_parse)?,
                rounds_total: next!(words).parse().map_err(Error::new_parse)?,
            }),
            "server.onRoundOver" => Ok(Event::ServerOnRoundOver {
                team_id: next!(words).parse().map_err(Error::new_parse)?,
            }),
            "server.onMaxPlayerCountChange" => Ok(Event::ServerOnMaxPlayerCountChange {
                count: next!(words).parse().map_err(Error::new_parse)?,
            }),
            other => Err(Error::new_parse(format!("invalid event: {}", other))),
        }
    }
}
