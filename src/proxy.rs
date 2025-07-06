use cote::prelude::error;
use cote::Result;
use tokio::sync::mpsc::channel;
use tokio::sync::mpsc::Receiver;
use tokio::sync::mpsc::Sender;

#[derive(Debug)]
pub struct Client<C, S> {
    send: Sender<S>,

    recv: Receiver<C>,
}

impl<S, C> Client<C, S> {
    pub fn req_sync(&mut self, msg: S) -> Result<C> {
        self.send
            .blocking_send(msg)
            .map_err(|e| error!("send msg failed: {e:?}"))?;

        self.recv
            .blocking_recv()
            .ok_or_else(|| error!("can not receive msg "))
    }

    pub async fn req(&mut self, msg: S) -> Result<C> {
        self.send
            .send(msg)
            .await
            .map_err(|e| error!("send msg failed: {e:?}"))?;

        self.recv
            .recv()
            .await
            .ok_or_else(|| error!("can not receive msg "))
    }
}

#[derive(Debug)]
pub struct Server<C, S> {
    pub send: Sender<C>,

    pub recv: Receiver<S>,
}

pub fn proxy<C, S>(size: usize) -> (Client<C, S>, Server<C, S>) {
    let (client_send, client_recv) = channel(size);
    let (server_send, server_recv) = channel(size);

    (
        Client {
            send: client_send,
            recv: server_recv,
        },
        Server {
            send: server_send,
            recv: client_recv,
        },
    )
}
