use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VapidResponse {
    pub key: Vec<u8>,
}
