use crate::frame::{self, Frame};

use bytes::{Buf, BytesMut};
use std::io::Cursor;
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufWriter};
use tokio::net::TcpStream;
use tracing::{debug, info};

#[derive(Debug)]
pub struct Connection {
    stream: BufWriter<TcpStream>,
    buffer: BytesMut,
}

impl Connection {
    pub fn new(socket: TcpStream) -> Connection {
        Connection {
            stream: BufWriter::new(socket),
            buffer: BytesMut::with_capacity(4 * 1024),
        }
    }

    pub async fn read_frame(&mut self) -> crate::Result<Option<Frame>> {
        loop {
            info!("Loop read_frame");
            if let Some(frame) = self.parse_frame()? {
                info!("Frame parsed");
                return Ok(Some(frame));
            }

            if 0 == self.stream.read_buf(&mut self.buffer).await? {
                if self.buffer.is_empty() {
                    return Ok(None);
                } else {
                    return Err("connection reset by peer".into());
                }
            }
        }
    }

    fn parse_frame(&mut self) -> crate::Result<Option<Frame>> {
        use frame::Error::Incomplete;

        let mut buf = Cursor::new(&self.buffer[..]);
        debug!(?buf);

        match Frame::check(&mut buf) {
            Ok(_) => {
                info!("Frame::check succesful");
                let len = buf.position() as usize;
                debug!(?len);
                buf.set_position(0);

                let frame = Frame::parse(&mut buf)?;
                self.buffer.advance(len);

                Ok(Some(frame))
            }
            Err(Incomplete) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    pub async fn write_frame(&mut self, frame: &Frame) -> crate::Result<()> {
        debug!(?frame);
        if let Frame::Response(mean) = frame {
            let _ = self.stream.write(&[mean.to_ne_bytes()[0]]).await?;
            return Ok(());
        }

        Err("Wrong frame".into())
    }
}
