use crate::polyfill::Polyfill;
use raad::be::*;
use std::io::Read;

#[derive(Debug)]
pub struct PolyfillData {
    number_of_polyfills: i32,
    polyfills:Vec<Polyfill>
}

impl PolyfillData {
    pub fn read(reader: &mut impl Read) -> std::io::Result<Self> {

        let number_of_polyfills = reader.r::<i32>()?;

        let mut polyfills = Vec::new();
        for _ in 0..number_of_polyfills {
            let polyfill = Polyfill::read(reader)?;
            polyfills.push(polyfill);
        }

        return Ok(PolyfillData {
            number_of_polyfills,
            polyfills
        })
    }
}