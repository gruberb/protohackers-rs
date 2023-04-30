use crate::{Message, Result, Username};
use futures::{SinkExt, StreamExt};
use tokio::net::TcpStream;
use tokio::sync::broadcast::Sender;
use tokio_util::codec::{Framed, LinesCodec};
use tracing::{debug, error, info};

#[derive(Clone, Debug, Default)]
pub struct BroadcastMessage {
    pub(crate) from: Username,
    pub(crate) message: Message,
}

impl BroadcastMessage {
    pub fn new(from: Username, message: Message) -> Self {
        BroadcastMessage { from, message }
    }
}

#[derive(Debug)]
pub struct Connection {
    pub stream: Framed<TcpStream, LinesCodec>,
    pub broadcast: Sender<BroadcastMessage>,
}

impl Connection {
    pub fn new(socket: TcpStream, sender: Sender<BroadcastMessage>) -> Connection {
        Connection {
            stream: Framed::new(socket, LinesCodec::new()),
            broadcast: sender,
        }
    }

    pub async fn red_next_frame(&mut self) -> Result<Option<String>> {
        return if let Some(Ok(frame)) = self.stream.next().await {
            info!("Frame for parsing the username parsed");
            Ok(Some(frame))
        } else {
            Err("connection reset by peer".into())
        };
    }

    pub async fn write_frame(&mut self, response: String) -> Result<()> {
        debug!(?response);
        if let Err(e) = self.stream.send(response.clone()).await {
            error!("Could not write frame to stream");
            return Err(e.to_string().into());
        }
        info!("Wrote to frame: {}", response);
        Ok(())
    }

    pub fn broadcast_message(&mut self, message: BroadcastMessage) -> Result<()> {
        match self.broadcast.send(message.clone()) {
            Ok(n) => info!("Sent broadcast: {n}"),
            Err(e) => error!("Could not send broadcast: {e}"),
        }
        Ok(())
    }
}
