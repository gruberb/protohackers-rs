use futures::{SinkExt, StreamExt};
use tokio::net::TcpStream;
use tokio_util::codec::{Framed, LinesCodec};
use tracing::{debug, error, info};

#[derive(Debug)]
pub struct Connection {
    stream: Framed<TcpStream, LinesCodec>,
}

impl Connection {
    pub fn new(socket: TcpStream) -> Connection {
        Connection {
            stream: Framed::new(socket, LinesCodec::new()),
        }
    }

    pub async fn read_frame(&mut self) -> crate::Result<Option<String>> {
        loop {
            info!("Read next frame");
            if let Some(Ok(frame)) = self.stream.next().await {
                info!("Frame parsed");
                return Ok(Some(frame));
            } else {
                return Err("connection reset by peer".into());
            }
        }
    }

    pub async fn write_frame(&mut self, response: String) -> crate::Result<()> {
        debug!(?response);
        if let Err(e) = self.stream.send(response.clone()).await {
            error!("Could not write frame to stream");
            return Err(e.to_string().into());
        }
        info!("Wrote to frame: {}", response);
        Ok(())
    }
}
