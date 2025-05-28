use serde::{Deserialize, Serialize};

use crate::DrawOrderIndex;

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct ConnectorData {
    pub local_x: f32,
    pub local_y: f32,
    pub percent: f32,
    pub percent_default: f32,
    pub value: f32,
    pub method: ConnectorMethod,
    pub reversed: bool,
    pub smart_stretch_ancestral_value: f32,

    pub end_node_draw_index: DrawOrderIndex
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub enum ConnectorMethod {
    #[default]
    ByPercent = 0,
    ByValue = 1
} 

impl ConnectorMethod {
    pub fn from_integer(int: i8) -> Option<Self> {
        match int {
            0 => Some(ConnectorMethod::ByPercent),
            1 => Some(ConnectorMethod::ByValue),
            _ => None,
        }
    }
    pub fn to_integer(&self) -> i8 {
        self.clone() as i8
    }
}