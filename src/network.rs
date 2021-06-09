use std::{
    net::{TcpListener, TcpStream},
    sync::mpsc::{channel, Receiver, Sender},
};

use crate::DEFAULT_PORT;

pub enum ChannelMessage {
    ConnectRequest,
    ConnectAccept,
    Message,
    File,
    Disconnect,
}

#[derive(Debug)]
pub struct Node {
    server: TcpListener,
    client: Option<TcpStream>,
    rx: Receiver<ChannelMessage>,
    tx: Sender<ChannelMessage>,
}

impl Node {
    pub fn new(arx: Receiver<ChannelMessage>) -> (Self, Receiver<ChannelMessage>) {
        let (stx, srx) = channel();
        (
            Self {
                server: TcpListener::bind((std::net::Ipv4Addr::UNSPECIFIED, DEFAULT_PORT)).unwrap(),
                client: None,
                rx: arx,
                tx: stx,
            },
            srx,
        )
    }
}
