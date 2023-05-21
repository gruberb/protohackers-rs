use std::{future::Future, sync::Arc};

use tokio::{
	net::{TcpListener, TcpStream},
	sync::{broadcast, mpsc, Semaphore},
	time::{self, Duration},
};
use tracing::{debug, error, info};

use crate::{
	connection::ConnectionType,
	db::{Camera, CameraId, Db, DbHolder, DispatcherId, Limit, Mile, Plate, Road, Timestamp},
	frame::{ClientFrames, ServerFrames},
	heartbeat::Heartbeat,
	ticketing::issue_possible_ticket,
	Connection, Shutdown,
};

struct Listener {
	listener: TcpListener,
	db_holder: DbHolder,
	limit_connections: Arc<Semaphore>,
	notify_shutdown: broadcast::Sender<()>,
	shutdown_complete_tx: mpsc::Sender<()>,
}

struct Handler {
	connection: Connection,
	connection_type: Option<ConnectionType>,
	db: Db,
	shutdown: Shutdown,
	_shutdown_complete: mpsc::Sender<()>,
}

const MAX_CONNECTIONS: usize = 1500;

pub async fn run(listener: TcpListener, shutdown: impl Future) -> crate::Result<()> {
	let (notify_shutdown, _) = broadcast::channel(1);
	let (shutdown_complete_tx, mut shutdown_complete_rx) = mpsc::channel(1);

	let mut server = Listener {
		listener,
		db_holder: DbHolder::new(),
		limit_connections: Arc::new(Semaphore::new(MAX_CONNECTIONS)),
		notify_shutdown,
		shutdown_complete_tx,
	};

	tokio::select! {
		res = server.run() => {
			if let Err(err) = res {
				error!(cause = %err, "failed to accept");
			}
		}
		_ = shutdown => {
			info!("shutting down");
		}
	}

	let Listener {
		shutdown_complete_tx,
		notify_shutdown,
		..
	} = server;

	drop(notify_shutdown);
	drop(shutdown_complete_tx);

	let _ = shutdown_complete_rx.recv().await;

	Ok(())
}

impl Listener {
	async fn run(&mut self) -> crate::Result<()> {
		info!("accepting inbound connections");

		loop {
			let permit = self
				.limit_connections
				.clone()
				.acquire_owned()
				.await
				.unwrap();

			let socket = self.accept().await?;
			let address = socket.peer_addr()?;

			let mut handler = Handler {
				connection: Connection::new(address, socket),
				connection_type: None,
				db: self.db_holder.db(),
				shutdown: Shutdown::new(self.notify_shutdown.subscribe()),
				_shutdown_complete: self.shutdown_complete_tx.clone(),
			};

			info!("Created new handler");

			tokio::spawn(async move {
				if let Err(err) = handler.run().await {
					error!(cause = ?err, "connection error");
				}
				drop(permit);
			});
		}
	}

	async fn accept(&mut self) -> crate::Result<TcpStream> {
		let mut backoff = 1;

		loop {
			match self.listener.accept().await {
				Ok((socket, _)) => return Ok(socket),
				Err(err) => {
					if backoff > 64 {
						return Err(err.into());
					}
				}
			}

			time::sleep(Duration::from_secs(backoff)).await;

			backoff *= 2;
		}
	}
}

impl Handler {
	async fn run(&mut self) -> crate::Result<()> {
		let (send_message, mut receive_message): (
			mpsc::Sender<ServerFrames>,
			mpsc::Receiver<ServerFrames>,
		) = mpsc::channel(1024);

		while !self.shutdown.is_shutdown() {
			tokio::select! {
				res = self.connection.read_frame() => {
					match res? {
						Some(frame) => {
							info!("Received frame");
						   let _ = self.handle_client_frame(self.db.clone(), frame, send_message.clone()).await;
						},
						None => return Ok(()),
					}

				}
				message = receive_message.recv() => {
					info!("Received a message through the channel");
					match message {
						Some(message) => {
							let _ = self.connection.write_frame(message).await;
						},
						None => (),
					}
				}
				_ = self.shutdown.recv() => {
					debug!("Shutdown");
					return Ok(());
				}
			};
		}

		Ok(())
	}

	fn set_connection_type(&mut self, connection_type: ConnectionType) {
		match connection_type {
			ConnectionType::Camera => {
				self.connection_type = Some(connection_type);
			}
			ConnectionType::Dispatcher => {
				self.connection_type = Some(connection_type);
			}
		}
	}

	async fn handle_client_frame(
		&mut self,
		mut db: Db,
		frame: ClientFrames,
		send_message: mpsc::Sender<ServerFrames>,
	) -> crate::Result<()> {
		debug!(?frame);
		match frame {
			ClientFrames::Plate { plate, timestamp } => {
				info!("Receive new plate {plate} {timestamp}");
				db.add_plate(
					CameraId(self.connection.get_address()),
					Plate {
						plate: plate.clone(),
						timestamp: Timestamp(timestamp),
					},
				);

				issue_possible_ticket(
					&mut db,
					Plate {
						plate,
						timestamp: Timestamp(timestamp),
					},
					CameraId(self.connection.get_address()),
				)
				.await;
			}
			ClientFrames::WantHeartbeat { interval } => {
				if interval > 0 {
					tokio::spawn(async move {
						let mut heartbeat = Heartbeat::new(interval, send_message.clone());
						heartbeat.start().await;
					});
				}
			}
			ClientFrames::IAmCamera { road, mile, limit } => {
				if self.connection_type.is_some() {
					return Err("Already connected".into());
				}
				self.set_connection_type(ConnectionType::Camera);
				info!("Set connection type to camera");

				db.add_camera(
					CameraId(self.connection.get_address()),
					Camera {
						road: Road(road),
						mile: Mile(mile),
						limit: Limit(limit),
					},
				);
			}
			ClientFrames::IAmDispatcher { roads } => {
				if self.connection_type.is_some() {
					return Err("Already connected".into());
				}

				self.set_connection_type(ConnectionType::Dispatcher);
				db.add_dispatcher(
					DispatcherId(self.connection.get_address()),
					roads.to_vec(),
					send_message.clone(),
				);
			}
		}

		Ok(())
	}
}
