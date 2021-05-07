use std::{collections::HashMap, io};

use bytes::BytesMut;
use futures_util::{SinkExt, StreamExt};
use tokio::{
    net::TcpStream,
    sync::{broadcast, mpsc, oneshot},
};
use tokio_util::codec::{Decoder, Encoder, Framed};

use crate::{models::Event, Error, Packet};

pub(crate) type Request = (Vec<String>, oneshot::Sender<Response>);
pub(crate) type Response = (String, Vec<String>);

pub(crate) struct Connection {
    next_id: u32,
    stream: Framed<TcpStream, Codec>,
    event_tx: broadcast::Sender<Event>,
    request_rx: mpsc::UnboundedReceiver<Request>,
    pending_requests: HashMap<u32, oneshot::Sender<Response>>,
}

impl Connection {
    pub(crate) fn new(
        tcp_stream: TcpStream,
        event_tx: broadcast::Sender<Event>,
        request_rx: mpsc::UnboundedReceiver<Request>,
    ) -> Connection {
        Connection {
            next_id: 0,
            stream: Framed::new(tcp_stream, Codec),
            event_tx,
            request_rx,
            pending_requests: HashMap::new(),
        }
    }

    pub(crate) async fn run(mut self) -> Result<(), Error> {
        loop {
            tokio::select! {
                packet = self.stream.next() => {
                    match packet {
                        Some(packet) => self.handle_recv(packet?)?,
                        None => return Ok(()),
                    }
                }

                request = self.request_rx.recv() => {
                    match request {
                        Some((words, response_tx)) => {
                            let id = self.next_id;
                            self.next_id = self.next_id.wrapping_add(1);
                            self.stream
                                .send(Packet::new(id, false, false, words))
                                .await?;
                            self.pending_requests.insert(id, response_tx);
                        }
                        None => return Ok(()),
                    }
                }
            }
        }
    }

    fn handle_recv(&mut self, packet: Packet) -> Result<(), Error> {
        match (packet.is_response, packet.is_from_server) {
            (true, false) => {
                if let Some(response_tx) = self.pending_requests.remove(&packet.id) {
                    let mut words = packet.words;
                    if words.is_empty() {
                        return Err(Error::from(io::Error::new(
                            io::ErrorKind::InvalidData,
                            "empty response packet",
                        )));
                    }

                    let _ = response_tx.send((words.remove(0), words));
                }
            }
            (false, true) => {
                let _ = self.event_tx.send(Event::from_words(packet.words)?);
            }
            _ => unreachable!(),
        }

        Ok(())
    }
}

struct Codec;

impl Decoder for Codec {
    type Item = Packet;
    type Error = io::Error;

    fn decode(&mut self, buf: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        Packet::read(buf)
    }
}

impl Encoder<Packet> for Codec {
    type Error = io::Error;

    fn encode(&mut self, packet: Packet, buf: &mut BytesMut) -> Result<(), Self::Error> {
        packet.write(buf)
    }
}
