use tokio::{
    net::{TcpStream, ToSocketAddrs},
    sync::{broadcast, mpsc, oneshot},
};

use crate::{
    models::{Event, PlayerInfo, ServerInfo, Subset},
    Connection, Error, Request,
};

#[derive(Clone, Debug)]
pub struct Client {
    event_tx: broadcast::Sender<Event>,
    request_tx: mpsc::UnboundedSender<Request>,
}

impl Client {
    pub async fn connect<A: ToSocketAddrs>(addr: A) -> Result<Client, Error> {
        let tcp_stream = TcpStream::connect(addr).await?;
        let (event_tx, _) = broadcast::channel(1000);
        let (request_tx, request_rx) = mpsc::unbounded_channel();
        let connection = Connection::new(tcp_stream, event_tx.clone(), request_rx);
        tokio::spawn(connection.run());

        Ok(Client {
            event_tx,
            request_tx,
        })
    }

    /// Completes when the connection to the remote host has been lost or terminated.
    pub async fn closed(&self) {
        self.request_tx.closed().await
    }

    /// Checks if the connection to the remote host has been lost or terminated.
    pub fn is_closed(&self) -> bool {
        self.request_tx.is_closed()
    }

    pub fn subscribe(&self) -> broadcast::Receiver<Event> {
        self.event_tx.subscribe()
    }

    pub async fn send(&self, words: Vec<String>) -> Result<Vec<String>, Error> {
        let (response_tx, response_rx) = oneshot::channel();
        if self.request_tx.send((words, response_tx)).is_err() {
            return Err(Error::new_connection_lost());
        }

        match response_rx.await {
            Ok(res) => {
                if &res.0 == "OK" {
                    Ok(res.1)
                } else {
                    Err(Error::new_status(res.0))
                }
            }
            Err(_) => Err(Error::new_connection_lost()),
        }
    }

    pub async fn login(&self, password: String) -> Result<(), Error> {
        self.send(vec!["login.plainText".to_owned(), password])
            .await?;

        Ok(())
    }

    pub async fn events_enable(&self) -> Result<(), Error> {
        self.send(vec!["admin.eventsEnabled".to_owned(), true.to_string()])
            .await?;

        Ok(())
    }

    pub async fn events_disable(&self) -> Result<(), Error> {
        self.send(vec!["admin.eventsEnabled".to_owned(), false.to_string()])
            .await?;

        Ok(())
    }

    pub async fn get_server_info(&self) -> Result<ServerInfo, Error> {
        let words = self.send(vec!["serverInfo".to_owned()]).await?;
        let server_info = ServerInfo::from_words(words)?;

        Ok(server_info)
    }

    /// Returns list of all players currently on the server.
    pub async fn players_get(&self) -> Result<Vec<PlayerInfo>, Error> {
        let mut words = self
            .send(vec!["admin.listPlayers".to_owned(), "all".to_owned()])
            .await?
            .into_iter();

        let offset: usize = next_parse!(words);
        words.nth(offset - 1);
        let num_of_players: usize = next_parse!(words);
        let mut players = Vec::with_capacity(num_of_players);
        for _ in 0..num_of_players {
            players.push(PlayerInfo {
                name: next!(words),
                guid: next!(words),
                team_id: next_parse!(words),
                squad_id: next_parse!(words),
                kills: next_parse!(words),
                deaths: next_parse!(words),
                score: next_parse!(words),
                rank: next_parse!(words),
                ping: next_parse!(words),
                kind: next_parse!(words),
            });
        }

        Ok(players)
    }

    pub async fn say(&self, message: String, subset: Subset) -> Result<(), Error> {
        let mut words = vec!["admin.say".to_owned(), message];
        words.extend(subset.into_words());

        self.send(words).await?;

        Ok(())
    }

    pub async fn yell(&self, message: String, duration: u64, subset: Subset) -> Result<(), Error> {
        let mut words = vec!["admin.yell".to_owned(), message, duration.to_string()];
        words.extend(subset.into_words());

        self.send(words).await?;

        Ok(())
    }

    pub async fn players_move(
        &self,
        name: String,
        team_id: usize,
        squad_id: usize,
        force_kill: bool,
    ) -> Result<(), Error> {
        self.send(vec![
            "admin.movePlayer".to_owned(),
            name,
            team_id.to_string(),
            squad_id.to_string(),
            force_kill.to_string(),
        ])
        .await?;

        Ok(())
    }

    pub async fn players_kill(&self, name: String) -> Result<(), Error> {
        self.send(vec!["admin.killPlayer".to_owned(), name]).await?;

        Ok(())
    }

    pub async fn players_kick(&self, name: String, reason: Option<String>) -> Result<(), Error> {
        self.send(vec![
            "admin.kickPlayer".to_owned(),
            name,
            reason.unwrap_or_default(),
        ])
        .await?;

        Ok(())
    }

    pub async fn maps_get(&self) -> Result<Vec<(String, String, usize)>, Error> {
        let mut maps = Vec::new();
        let mut offset: usize = 0;
        let mut words = self
            .send(vec!["mapList.list".to_owned(), offset.to_string()])
            .await?
            .into_iter();

        loop {
            let num_of_maps: usize = next_parse!(words);
            assert_eq!(next!(words), "3");
            maps.reserve(num_of_maps);

            for _ in 0..num_of_maps {
                maps.push((next!(words), next!(words), next_parse!(words)));
            }

            if num_of_maps >= 100 {
                offset += 100;
            } else {
                return Ok(maps);
            }
        }
    }

    pub async fn maps_remove(&self, index: usize) -> Result<(), Error> {
        self.send(vec!["mapList.remove".to_owned(), index.to_string()])
            .await?;

        Ok(())
    }

    pub async fn maps_clear(&self) -> Result<(), Error> {
        self.send(vec!["mapList.clear".to_owned()]).await?;

        Ok(())
    }

    pub async fn maps_load(&self) -> Result<(), Error> {
        self.send(vec!["mapList.load".to_owned()]).await?;

        Ok(())
    }

    pub async fn maps_save(&self) -> Result<(), Error> {
        self.send(vec!["mapList.save".to_owned()]).await?;

        Ok(())
    }

    pub async fn maps_get_indexes(&self) -> Result<(usize, usize), Error> {
        let mut words = self
            .send(vec!["mapList.getMapIndices".to_owned()])
            .await?
            .into_iter();

        Ok((next_parse!(words), next_parse!(words)))
    }

    pub async fn maps_set_next_index(&self, index: usize) -> Result<(), Error> {
        self.send(vec![
            "mapList.setNextMapIndex".to_owned(),
            index.to_string(),
        ])
        .await?;

        Ok(())
    }

    /// Ends the current round, declaring specified team as the winner.
    pub async fn maps_end_round(&self, team_id: usize) -> Result<(), Error> {
        self.send(vec!["mapList.endRound".to_owned(), team_id.to_string()])
            .await?;

        Ok(())
    }

    /// Restarts the current round, without going through end screen.
    pub async fn maps_restart_round(&self) -> Result<(), Error> {
        self.send(vec!["mapList.restartRound".to_owned()]).await?;

        Ok(())
    }

    /// Runs the next round, without going through end screen.
    pub async fn maps_run_next_round(&self) -> Result<(), Error> {
        self.send(vec!["mapList.runNextRound".to_owned()]).await?;

        Ok(())
    }
}
