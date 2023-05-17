use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::broadcast;

use crate::frame::ServerFrames;

#[derive(Debug)]
pub(crate) struct DbHolder {
    /// The `Db` instance that will be shut down when this `DbHolder` struct
    /// is dropped.
    db: Db,
}

#[derive(Debug, Clone)]
pub(crate) struct Db {
    state: Arc<Mutex<State>>,
}

#[derive(Debug)]
struct State {
    // cameras: HashMap<(u32, u32), u32>,
    dispatchers: HashMap<Vec<u16>, broadcast::Sender<ServerFrames>>,
    plates: HashMap<String, u32>,
}

impl DbHolder {
    /// Create a new `DbHolder`, wrapping a `Db` instance. When this is dropped
    /// the `Db`'s purge task will be shut down.
    pub(crate) fn new() -> DbHolder {
        DbHolder { db: Db::new() }
    }

    /// Get the shared database. Internally, this is an
    /// `Arc`, so a clone only increments the ref count.
    pub(crate) fn db(&self) -> Db {
        self.db.clone()
    }
}

impl Db {
    pub(crate) fn new() -> Db {
        let state = Arc::new(Mutex::new(State {
            // cameras: HashMap::new(),
            dispatchers: HashMap::new(),
            plates: HashMap::new(),
        }));

        Db { state }
    }

    // pub(crate) fn new_camera(&self, road: u32, mile: u32, limit: u32) {}

    pub(crate) fn add_dispatcher(
        &self,
        roads: Vec<u16>,
        writer_stream: broadcast::Sender<ServerFrames>,
    ) {
        let mut state = self.state.lock().unwrap();
        state.dispatchers.insert(roads, writer_stream);
    }

    pub(crate) fn insert_plate(&self, plate: String, timestamp: u32) {
        let mut state = self.state.lock().unwrap();
        state.plates.insert(plate, timestamp);
    }
}
