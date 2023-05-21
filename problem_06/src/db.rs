use std::{
	collections::{HashMap, HashSet},
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
	pub(crate) timestamp: Timestamp,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub(crate) struct Camera {
	pub(crate) road: Road,
	pub(crate) mile: Mile,
	pub(crate) limit: Limit,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub(crate) struct Ticket {
	pub(crate) plate: String,
	pub(crate) road: u16,
	pub(crate) mile1: u16,
	pub(crate) timestamp1: u32,
	pub(crate) mile2: u16,
	pub(crate) timestamp2: u32,
	pub(crate) speed: u16,
}

impl From<Ticket> for ServerFrames {
	fn from(ticket: Ticket) -> Self {
		ServerFrames::Ticket {
			plate: ticket.plate,
			road: ticket.road,
			mile1: ticket.mile1,
			timestamp1: ticket.timestamp1,
			mile2: ticket.mile2,
			timestamp2: ticket.timestamp2,
			speed: ticket.speed,
		}
	}
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub(crate) struct Road(pub(crate) u16);

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub(crate) struct Limit(pub(crate) u16);

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub(crate) struct Timestamp(pub(crate) u32);

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub(crate) struct Mile(pub(crate) u16);

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
	dispatchers: HashMap<Road, Vec<(DispatcherId, mpsc::Sender<ServerFrames>)>>,
	plates: HashMap<(Plate, Road), Vec<(Mile, Timestamp)>>,
	ticketed_plates_by_day: HashSet<(Timestamp, String)>,
	open_tickets: HashMap<Road, Vec<Ticket>>,
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
			ticketed_plates_by_day: HashSet::new(),
			open_tickets: HashMap::new(),
		}));

		Db { state }
	}

	pub(crate) fn get_camera(&self, camera_id: CameraId) -> Option<Camera> {
		let state = self.state.lock().unwrap();
		state.cameras.get(&camera_id).cloned()
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
				.entry(Road(*r))
				.or_insert(Vec::new())
				.push((dispatcher_id.clone(), writer_stream.clone()));
		}

		debug!(?state);
	}

	pub(crate) fn get_dispatcher_for_road(&self, road: Road) -> Option<mpsc::Sender<ServerFrames>> {
		let state = self.state.lock().unwrap();
		let senders = state.dispatchers.get(&road);
		if senders.is_none() {
			return None;
		}

		senders.unwrap().first().map(|(_, s)| s.clone())
	}

	pub(crate) fn add_open_ticket(&self, ticket: Ticket) {
		let mut state = self.state.lock().unwrap();
		state
			.open_tickets
			.entry(Road(ticket.road))
			.or_insert(Vec::new())
			.push(ticket);
	}

	pub(crate) fn get_plates_by_road(
		&self,
		plate: Plate,
		road: Road,
	) -> Option<Vec<(Mile, Timestamp)>> {
		let state = self.state.lock().unwrap();
		state.plates.get(&(plate, road)).cloned()
	}

	pub(crate) fn add_plate(&self, camera_id: CameraId, plate: Plate) {
		let state = self.state.lock().unwrap();
		let camera = self.get_camera(camera_id).unwrap();

		match self
			.state
			.lock()
			.unwrap()
			.plates
			.get_mut(&(plate.clone(), camera.road.clone()))
		{
			Some(v) => v.push((camera.mile, plate.timestamp)),
			None => {
				self.state.lock().unwrap().plates.insert(
					(plate.clone(), camera.road),
					vec![(camera.mile, plate.timestamp)],
				);
			}
		}

		debug!(?state);
	}

	pub(crate) fn ticket_plate(&self, day: u32, plate_name: String) {
		let mut state = self.state.lock().unwrap();
		state
			.ticketed_plates_by_day
			.insert((Timestamp(day), plate_name));
		debug!(?state);
	}

	pub(crate) fn is_plate_ticketed_for_day(&self, day: u32, plate_name: String) -> bool {
		let state = self.state.lock().unwrap();
		state
			.ticketed_plates_by_day
			.contains(&(Timestamp(day), plate_name))
	}
}
