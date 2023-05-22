use std::{
	collections::{HashMap, HashSet},
	net::SocketAddr,
};

use tokio::sync::mpsc;
use tracing::info;

use crate::frame::ServerFrames;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub(crate) struct DispatcherId(pub(crate) SocketAddr);

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub(crate) struct CameraId(pub(crate) SocketAddr);

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub(crate) struct Plate {
	pub(crate) plate: PlateName,
	pub(crate) timestamp: Timestamp,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub(crate) struct PlateName(pub(crate) String);

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

#[derive(Debug)]
pub(crate) struct Db {
	cameras: HashMap<CameraId, Camera>,
	dispatchers: HashMap<Road, Vec<(DispatcherId, mpsc::Sender<ServerFrames>)>>,
	plates: HashMap<(PlateName, Road), Vec<(Mile, Timestamp)>>,
	ticketed_plates_by_day: HashSet<(Timestamp, String)>,
	open_tickets: HashMap<Road, Vec<Ticket>>,
}

impl Db {
	pub(crate) fn new() -> Db {
		Db {
			cameras: HashMap::new(),
			dispatchers: HashMap::new(),
			plates: HashMap::new(),
			ticketed_plates_by_day: HashSet::new(),
			open_tickets: HashMap::new(),
		}
	}

	pub(crate) fn get_camera(&self, camera_id: CameraId) -> Option<Camera> {
		self.cameras.get(&camera_id).cloned()
	}

	pub(crate) fn add_camera(&mut self, camera_id: CameraId, camera: Camera) {
		self.cameras.insert(camera_id, camera);
	}

	pub(crate) fn add_dispatcher(
		&mut self,
		dispatcher_id: DispatcherId,
		roads: Vec<u16>,
		writer_stream: mpsc::Sender<ServerFrames>,
	) {
		info!("Adding new dispatcher for roads: {roads:?}");
		for r in roads.iter() {
			self.dispatchers
				.entry(Road(*r))
				.or_insert(Vec::new())
				.push((dispatcher_id.clone(), writer_stream.clone()));
		}
	}

	pub(crate) fn get_dispatcher_for_road(&self, road: Road) -> Option<mpsc::Sender<ServerFrames>> {
		let senders = self.dispatchers.get(&road);
		if senders.is_none() {
			return None;
		}

		senders.unwrap().first().map(|(_, s)| s.clone())
	}

	pub(crate) fn add_open_ticket(&mut self, ticket: Ticket) {
		info!("Adding open ticket: {ticket:?}");
		self.open_tickets
			.entry(Road(ticket.road))
			.or_insert(Vec::new())
			.push(ticket);
	}

	pub(crate) fn get_open_tickets(&self) -> Vec<Ticket> {
		self.open_tickets.values().flatten().cloned().collect()
	}

	pub(crate) fn remove_open_ticket(&mut self, road: Road, ticket: Ticket) -> bool {
		info!("Removing open ticket: {ticket:?}");
		if let Some(tickets) = self.open_tickets.get_mut(&road) {
			tickets.retain(|t| t.plate != ticket.plate);
			if tickets.is_empty() {
				self.open_tickets.remove(&road);
			}
			return true;
		}
		false
	}

	pub(crate) fn get_plates_by_road(
		&self,
		plate: Plate,
		road: Road,
	) -> Option<Vec<(Mile, Timestamp)>> {
		self.plates.get(&(plate.plate, road)).cloned()
	}

	pub(crate) fn add_plate(&mut self, camera_id: CameraId, plate: Plate) {
		//TODO: Check if the same plate was already added for the road AND MILE
		let camera = self.get_camera(camera_id).unwrap();

		match self
			.plates
			.get_mut(&(plate.plate.clone(), camera.road.clone()))
		{
			Some(v) => v.push((camera.mile, plate.timestamp)),
			None => {
				self.plates.insert(
					(plate.clone().plate, camera.road),
					vec![(camera.mile, plate.timestamp)],
				);
			}
		}
	}

	pub(crate) fn ticket_plate(&mut self, day: u32, plate_name: PlateName) {
		info!("Add {plate_name:?} for day:{day} ");
		self.ticketed_plates_by_day
			.insert((Timestamp(day), plate_name.0));
	}

	pub(crate) fn is_plate_ticketed_for_day(&self, day: u32, plate_name: PlateName) -> bool {
		info!(
			"Current ticketed plates, by day: {:?}",
			self.ticketed_plates_by_day
		);
		self.ticketed_plates_by_day
			.contains(&(Timestamp(day), plate_name.0))
	}
}
