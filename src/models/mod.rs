#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

mod event;
pub use self::event::Event;

mod player;
pub use self::player::{PlayerInfo, PlayerKind};

mod server;
pub use self::server::ServerInfo;

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub enum Subset {
    All,
    Squad { team_id: usize, squad_id: usize },
    Team { team_id: usize },
    Player { name: String },
}

impl Subset {
    pub(crate) fn into_words(self) -> Vec<String> {
        match self {
            Subset::All => vec!["all".to_owned()],
            Subset::Squad { team_id, squad_id } => vec![
                "squad".to_owned(),
                team_id.to_string(),
                squad_id.to_string(),
            ],
            Subset::Team { team_id } => vec!["team".to_owned(), team_id.to_string()],
            Subset::Player { name } => vec!["player".to_owned(), name],
        }
    }
}
