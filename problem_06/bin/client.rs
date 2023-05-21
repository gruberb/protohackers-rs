use problem_06::{DEFAULT_IP, DEFAULT_PORT};
use tokio::{
	io::{AsyncReadExt, AsyncWriteExt},
	net::{tcp::WriteHalf, TcpStream},
};
use tracing::{debug, error, info};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	tracing_subscriber::fmt::init();

	let mut stream = TcpStream::connect(format!("{DEFAULT_IP}:{DEFAULT_PORT}")).await?;
	let (mut read, mut write) = stream.split();

	// test_all_different_messages(&mut write).await?;
	// test_camera1_connection(&mut write).await?;
	// test_camera2_connection(&mut write).await?;
	test_dipatcher_connection(&mut write).await?;

	let mut buf: [u8; 4] = [0; 4];

	if let Ok(n) = read.read_exact(&mut buf).await {
		info!("Stream incoming...");

		if n == 0 {
			info!("End of stream");
			return Ok(());
		}

		let message = i32::from_be_bytes(buf);
		debug!(?message);
		return Ok(());
	}

	error!("Cannot read from socket");
	Err("Could not read from socket".into())
}

#[allow(dead_code)]
async fn test_all_different_messages(
	write: &mut WriteHalf<'_>,
) -> Result<(), Box<dyn std::error::Error>> {
	// 20                          Plate {
	// 07 52 45 30 35 42 4b 47         plate: "RE05BKG",
	// 00 01 e2 40                     timestamp: 123456
	//                             }
	let plate = [
		0x20, 0x07, 0x52, 0x45, 0x30, 0x35, 0x42, 0x4b, 0x47, 0x00, 0x01, 0xe2, 0x40,
	];

	// 40              WantHeartbeat{
	// 00 00 00 0a         interval: 10
	//                 }
	let want_heartbeat = [0x40, 0x00, 0x00, 0x00, 0x0a];

	// 80              IAmCamera{
	// 00 42               road: 66,
	// 00 64               mile: 100,
	// 00 3c               limit: 60,
	//                  }
	let i_am_camera = [0x80, 0x00, 0x42, 0x00, 0x64, 0x00, 0x3c];

	// 81              IAmDispatcher{
	// 03                  roads: [
	// 00 42                   66,
	// 01 70                   368,
	// 13 88                   5000
	//                     ]
	//                 }
	let i_am_dispatcher = [0x81, 0x03, 0x00, 0x42, 0x01, 0x70, 0x13, 0x88];

	write.write_all(&plate).await?;
	write.write_all(&want_heartbeat).await?;
	write.write_all(&i_am_camera).await?;
	write.write_all(&i_am_dispatcher).await?;

	Ok(())
}

#[allow(dead_code)]
async fn test_camera1_connection(
	write: &mut WriteHalf<'_>,
) -> Result<(), Box<dyn std::error::Error>> {
	// 80              IAmCamera{
	// 00 7b               road: 123,
	// 00 08               mile: 8,
	// 00 3c               limit: 60,
	//                  }
	let i_am_camera = [0x80, 0x00, 0x7b, 0x00, 0x08, 0x00, 0x3c];

	// 20                          Plate {
	// 04 55 4e 31 58                  plate: "UN1X",
	// 00 00 00 00                     timestamp: 0
	//                             }
	let plate = [0x20, 0x04, 0x55, 0x4e, 0x31, 0x58, 0x00, 0x00, 0x00, 0x00];

	write.write_all(&i_am_camera).await?;
	write.write_all(&plate).await?;

	Ok(())
}
#[allow(dead_code)]
async fn test_camera2_connection(
	write: &mut WriteHalf<'_>,
) -> Result<(), Box<dyn std::error::Error>> {
	// 80              IAmCamera{
	// 00 7b               road: 123,
	// 00 09               mile: 8,
	// 00 3c               limit: 60,
	//                  }
	let i_am_camera = [0x80, 0x00, 0x7b, 0x00, 0x09, 0x00, 0x3c];

	// 20                          Plate {
	// 04 55 4e 31 58                  plate: "UN1X",
	// 00 00 00 2d                     timestamp: 45
	//                             }
	let plate = [0x20, 0x04, 0x55, 0x4e, 0x31, 0x58, 0x00, 0x00, 0x00, 0x2d];

	write.write_all(&i_am_camera).await?;
	write.write_all(&plate).await?;

	Ok(())
}

#[allow(dead_code)]
async fn test_dipatcher_connection(
	write: &mut WriteHalf<'_>,
) -> Result<(), Box<dyn std::error::Error>> {
	// 81              IAmDispatcher{
	// 01                  roads: [
	// 00 7b                   123,
	//                     ]
	//                 }
	// let i_am_dispatcher = [0x81, 0x03, 0x00, 0x42, 0x01, 0x70, 0x13, 0x88];
	let i_am_dispatcher = [0x81, 0x01, 0x00, 0x7b];

	write.write_all(&i_am_dispatcher).await?;

	Ok(())
}
