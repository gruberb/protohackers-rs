use std::{
	collections::HashMap,
	net::SocketAddr,
	sync::{Arc, Mutex},
};

use tokio::sync::mpsc;
use tracing::debug;

use crate::frame::ServerFrames;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub(crate) struct DispatcherId(pub(crate) SocketAddr);

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub(crate) struct CameraId(pub(crate) SocketAddr);

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub(crate) struct Plate {
	pub(crate) plate: String,
	pub(crate) timestamp: u32,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub(crate) struct Camera {
	pub(crate) road: u16,
	pub(crate) mile: u16,
	pub(crate) limit: u16,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub(crate) struct Road(u16);

pub(crate) struct DbHolder {
	db: Db,
}

#[derive(Clone)]
pub(crate) struct Db {
	state: Arc<Mutex<State>>,
}

#[derive(Debug)]
struct State {
	cameras: HashMap<CameraId, Camera>,
	dispatchers: HashMap<Road, (DispatcherId, mpsc::Sender<ServerFrames>)>,
	plates: HashMap<CameraId, Plate>,
}

impl DbHolder {
	pub(crate) fn new() -> DbHolder {
		DbHolder { db: Db::new() }
	}

	pub(crate) fn db(&self) -> Db {
		self.db.clone()
	}
}

impl Db {
	pub(crate) fn new() -> Db {
		let state = Arc::new(Mutex::new(State {
			cameras: HashMap::new(),
			dispatchers: HashMap::new(),
			plates: HashMap::new(),
		}));

		Db { state }
	}

	pub(crate) fn add_camera(&self, camera_id: CameraId, camera: Camera) {
		let mut state = self.state.lock().unwrap();
		state.cameras.insert(camera_id, camera);
		debug!(?state);
	}

	pub(crate) fn add_dispatcher(
		&self,
		dispatcher_id: DispatcherId,
		roads: Vec<u16>,
		writer_stream: mpsc::Sender<ServerFrames>,
	) {
		let mut state = self.state.lock().unwrap();

		for r in roads.iter() {
			state
				.dispatchers
				.insert(Road(*r), (dispatcher_id.clone(), writer_stream.clone()));
		}

		debug!(?state);
	}

	pub(crate) fn insert_plate(&self, camera_id: CameraId, plate: Plate) {
		let mut state = self.state.lock().unwrap();
		state.plates.insert(camera_id, plate);
		debug!(?state);
	}
}
