use std::hash::Hash;

use serde::{de::DeserializeOwned, Serialize};

pub trait PeerId: Hash + Serialize + DeserializeOwned {}
