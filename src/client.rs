use tokio::{
    net::{TcpStream, ToSocketAddrs},
    sync::{broadcast, mpsc, oneshot},
};

use crate::{
    models::{Event, PlayerInfo, ServerInfo},
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

    pub async fn enable_events(&self) -> Result<(), Error> {
        self.send(vec!["admin.eventsEnabled".to_owned(), true.to_string()])
            .await?;

        Ok(())
    }

    pub async fn disable_events(&self) -> Result<(), Error> {
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
    pub async fn get_players(&self) -> Result<Vec<PlayerInfo>, Error> {
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
}
