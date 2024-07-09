use std::hash::Hash;

use serde::{de::DeserializeOwned, Serialize};

pub trait RoomId: Hash + Serialize + DeserializeOwned {}
