use std::sync::Arc;
use tokio::sync::RwLock;

use tracing::{debug, info};

use crate::Result;

#[derive(Debug, Clone)]
pub(crate) struct Db {
    users: Arc<RwLock<Vec<String>>>,
}

impl Db {
    pub fn new() -> Self {
        Db {
            users: Arc::new(RwLock::new(Vec::default())),
        }
    }

    pub async fn insert_user(&self, username: String) -> Result<()> {
        if !username.is_empty() && username.chars().all(char::is_alphabetic) {
            self.users.write().await.push(username);
            Ok(())
        } else {
            Err(format!("Cannot insert new user: {username}").into())
        }
    }

    pub async fn get_room_members(&self, username: String) -> Vec<String> {
        info!("Get room members: {:?}", self.users);
        self.users
            .read()
            .await
            .clone()
            .into_iter()
            .filter(|n| {
                info!("{n} is in the room");
                *n != username

            })
            .collect()
    }

    pub async fn remove(&self, username: String) -> Result<()> {
        self.users.write().await.retain(|n| *n != username);
        Ok(())
    }
}
