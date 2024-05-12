use crate::color::Color;
use raad::be::*;
use std::io::Read;

#[derive(Debug)]
pub struct StickfigureHeader {
    version: i32,
    build: i32,
    scale: i32,
    color: Color
}

impl StickfigureHeader {
    pub fn read(reader: &mut impl Read) -> std::io::Result<Self> {

        let version = reader.r::<i32>()?;
        let build = reader.r::<i32>()?;
        let scale = reader.r::<i32>()?;
        let color = Color {
            alpha: reader.r::<u8>()?,
            blue: reader.r::<u8>()?,
            green: reader.r::<u8>()?,
            red: reader.r::<u8>()?,
        };

        return Ok(StickfigureHeader {
            version,
            build,
            scale,
            color
        })
    }
}