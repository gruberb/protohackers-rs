use crate::db::{CameraId, Db, Plate, Ticket};

pub(crate) fn issue_possible_ticket(db: &mut Db, plate: Plate, camera_id: CameraId) {
	let camera = db.get_camera(camera_id).unwrap();
	let observed_plates = db
		.get_plates_by_road(plate.clone(), camera.road.clone())
		.unwrap();

	let mile = camera.mile;
	let limit = camera.limit;
	let road = camera.road;

	let plate_name = plate.plate;
	let timestamp = plate.timestamp;

	for (m, t) in observed_plates.iter() {
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

		let speed = distance * 3600 * 100 / time as u16;

		if speed > limit.0 * 100 {
			let ticket = Ticket {
				plate: plate_name.clone(),
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
				if db.is_plate_ticketed_for_day(day, plate_name.clone()) {
					continue;
				}

				let dispatcher = db.get_dispatcher_for_road(road.clone());

				if dispatcher.is_none() {
					db.add_open_ticket(ticket);
					continue;
				}

				dispatcher.unwrap().send(ticket).await;
				db.ticket_plate(day, plate_name.clone());
			}
		}
	}
}
