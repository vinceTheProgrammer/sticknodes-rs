use crate::stickfigure_header::StickfigureHeader;
use crate::polyfill_data::PolyfillData;
use crate::node_data::NodeData;
use std::io::Read;

#[derive(Debug)]
pub struct Stickfigure {
    header: StickfigureHeader,
    nodes: NodeData,
    polyfills: PolyfillData 
}

impl Stickfigure {
    pub fn read(reader: &mut impl Read) -> std::io::Result<Self> {

        let header = StickfigureHeader::read(reader)?;
    
        let nodes = NodeData::read(reader)?;
    
        let polyfills = PolyfillData::read(reader)?;
    
        return Ok(Stickfigure {
            header,
            nodes,
            polyfills
        });
    }
    
    fn write(_stickfigure: Stickfigure) -> Vec<u8> {
        todo!()
    }
}