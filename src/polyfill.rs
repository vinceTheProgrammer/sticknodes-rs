use crate::color::Color;
use raad::be::*;
use std::io::Read;

#[derive(Debug)]
pub struct Polyfill {
    anchor_node_id: i32,
    color: Color,
    use_polyfill_color: bool,
    number_of_attached_nodes: i32,
    attached_nodes: Vec<i32>
}

impl Polyfill {
    pub fn read(reader: &mut impl Read) -> std::io::Result<Self> {
        let anchor_node_id = reader.r::<i32>()?;
        let [alpha, blue, green, red] = reader.r::<[u8; 4]>()?;
        let use_polyfill_color = reader.r::<u8>()? != 0;
        let number_of_attached_nodes = reader.r::<i32>()?;

        let attached_nodes = (0..number_of_attached_nodes)
            .map(|_| reader.r::<i32>())
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Polyfill {
            anchor_node_id,
            color: Color { alpha, blue, green, red },
            use_polyfill_color,
            number_of_attached_nodes,
            attached_nodes 
        })
    }
}