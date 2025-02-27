use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ChassisIntrusion {
    Normal,
    Intruded,
}
