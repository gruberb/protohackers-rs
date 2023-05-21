use crate::frame::ServerFrames;
use std::time::Duration;
use tokio::sync::mpsc;

pub(crate) struct Heartbeat {
	is_running: bool,
	interval: Duration,
	message: mpsc::Sender<ServerFrames>,
}

impl Heartbeat {
	pub(crate) fn new(interval: u32, message: mpsc::Sender<ServerFrames>) -> Self {
		Self {
			is_running: false,
			interval: Duration::from_millis((interval * 100) as u64),
			message,
		}
	}

	pub(crate) async fn start(&mut self) {
		if self.is_running {
			let _ = self.message.send(ServerFrames::Error {
				msg: "Heartbeat alreadt exists".to_string(),
			});
			return;
		}

		self.is_running = true;

		let mut interval = tokio::time::interval(self.interval);

		interval.tick().await;

		loop {
			interval.tick().await;
			let _ = self.message.send(ServerFrames::Heartbeat);
		}
	}
}
