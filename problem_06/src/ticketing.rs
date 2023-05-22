use tracing::info;

use crate::db::{CameraId, Db, Plate, Road, Ticket};

pub(crate) async fn issue_possible_ticket(db: &mut Db, plate: Plate, camera_id: CameraId) {
	let camera = db.get_camera(camera_id.clone()).unwrap();
	let observed_plates = db.get_plates_by_road(plate.clone(), camera.road.clone());

	if observed_plates.is_none() {
		db.add_plate(camera_id, plate);
		return;
	}

	let mile = camera.mile;
	let limit = camera.limit;
	let road = camera.road;

	let plate_name = plate.clone().plate;
	let timestamp = plate.clone().timestamp;

	for (m, t) in observed_plates.unwrap().iter() {
		let distance = if mile > *m {
			mile.0 - m.0
		} else {
			m.0 - mile.0
		};

		let (time, mile1, timestamp1, mile2, timestamp2) = if timestamp > *t {
			(timestamp.0 - t.0, m.0, t.0, mile.0, timestamp.0)
		} else {
			(t.0 - timestamp.0, mile.0, timestamp.0, m.0, t.0)
		};

		let speed = (distance as u64 * 3600 * 100 / time as u64) as u16;

		if speed > limit.0 * 100 {
			let ticket = Ticket {
				plate: plate_name.clone().0,
				road: road.0,
				mile1,
				timestamp1,
				mile2,
				timestamp2,
				speed,
			};

			let day_start = timestamp1 / 86400;
			let day_end = timestamp2 / 86400;

			for day in day_start..=day_end {
				info!("Day {day} for {ticket:?}");
				if db.is_plate_ticketed_for_day(day, plate_name.clone()) {
					info!("Ticket already issued: {ticket:?}");
					continue;
				}

				let dispatcher = db.get_dispatcher_for_road(road.clone());

				if dispatcher.is_none() {
					info!("No dispatcher yet for this road: {ticket:?}");
					db.add_open_ticket(ticket.clone());
					continue;
				}

				info!("Sending ticket: {ticket:?}");
				let _ = dispatcher.unwrap().send(ticket.clone().into()).await;
				db.ticket_plate(day, plate_name.clone());
			}
		}
	}

	db.add_plate(camera_id, plate);
}

pub(crate) async fn send_out_waiting_tickets(db: Db) {
	let tickets = db.get_open_tickets();
	info!("Sending out waiting tickets: {tickets:?}");
	for ticket in tickets {
		if let Some(dispatcher) = db.get_dispatcher_for_road(Road(ticket.road)) {
			let _ = dispatcher.send(ticket.clone().into()).await;
			db.remove_open_ticket(Road(ticket.road), ticket);
		}
	}
}
