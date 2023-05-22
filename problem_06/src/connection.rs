use std::{io::Cursor, net::SocketAddr};

use bytes::{Buf, BytesMut};
use tokio::{
	io::{AsyncReadExt, AsyncWriteExt, BufWriter},
	net::TcpStream,
};

use crate::frame::{self, ClientFrames, ServerFrames};

#[derive(PartialEq)]
pub(crate) enum ConnectionType {
	Camera,
	Dispatcher,
}

#[derive(Debug)]
pub struct Connection {
	pub address: SocketAddr,
	buffer: BytesMut,
	pub(crate) stream: BufWriter<TcpStream>,
}

impl Connection {
	pub fn new(address: SocketAddr, socket: TcpStream) -> Connection {
		Connection {
			address,
			buffer: BytesMut::with_capacity(4 * 1024),
			stream: BufWriter::new(socket),
		}
	}

	pub fn get_address(&self) -> SocketAddr {
		self.address.clone()
	}

	pub async fn read_frame(&mut self) -> crate::Result<Option<ClientFrames>> {
		loop {
			if let Some(frame) = self.parse_frame()? {
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

	fn parse_frame(&mut self) -> crate::Result<Option<ClientFrames>> {
		use frame::Error::Incomplete;

		let mut buf = Cursor::new(&self.buffer[..]);

		match ClientFrames::check(&mut buf) {
			Ok(_) => {
				let len = buf.position() as usize;
				buf.set_position(0);

				let frame = ClientFrames::parse(&mut buf)?;
				self.buffer.advance(len);

				Ok(Some(frame))
			}
			Err(Incomplete) => Ok(None),
			Err(e) => Err(e.into()),
		}
	}

	pub async fn write_frame(&mut self, frame: ServerFrames) -> tokio::io::Result<()> {
		let _ = self.stream.write_all(&frame.convert_to_bytes()).await;
		self.stream.flush().await?;
		Ok(())
	}
}
