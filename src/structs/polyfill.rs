use hashbrown::HashSet;
use serde::Deserialize;
use serde::Serialize;
extern crate alloc;
use alloc::{format, vec::Vec};

use crate::{color::Color, Stickfigure, StickfigureError};

use super::stickfigure::DrawOrderIndex;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Polyfill {
    pub anchor_node_draw_index: DrawOrderIndex,
    pub color: Color,
    pub use_polyfill_color: bool,
    pub attached_node_draw_indices: Vec<DrawOrderIndex>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PolyfillOptions {
    pub anchor_node_draw_index: DrawOrderIndex,
    pub color: Color,
    pub use_polyfill_color: bool,
    pub attached_node_draw_indices: Vec<DrawOrderIndex>,
}

impl Default for Polyfill {
    fn default() -> Self {
        Self {
            anchor_node_draw_index: DrawOrderIndex(0),
            color: Color::default(),
            use_polyfill_color: false,
            attached_node_draw_indices: Vec::new(),
        }
    }
}

impl Default for PolyfillOptions {
    fn default() -> Self {
        Self {
            anchor_node_draw_index: DrawOrderIndex(0),
            color: Color::default(),
            use_polyfill_color: false,
            attached_node_draw_indices: Vec::new(),
        }
    }
}

impl Polyfill {
    pub fn from_options(
        options: PolyfillOptions,
        stickfigure: Stickfigure,
    ) -> Result<Self, StickfigureError> {
        let mut indices_to_check = options.attached_node_draw_indices.clone();
        indices_to_check.insert(0, options.anchor_node_draw_index);
        let missing_indices = stickfigure.missing_draw_indices(&indices_to_check);
        if missing_indices.iter().count() > 0 {
            return Err(StickfigureError::InvalidDrawIndices(
                format!("{:?}", missing_indices),
                format!("Cannot create polyfill from invalid draw order indices."),
            ));
        }

        if stickfigure.draw_index_is_polyfill_anchor(options.anchor_node_draw_index) {
            return Err(StickfigureError::NodeIsAlreadyAnchor(
                options.anchor_node_draw_index.0,
                format!("Cannot create polyfill with anchor that is already occupied."),
            ));
        }

        let mut polyfill = Polyfill::default();
        polyfill.anchor_node_draw_index = options.anchor_node_draw_index;
        polyfill.color = options.color;
        polyfill.use_polyfill_color = options.use_polyfill_color;
        polyfill.attached_node_draw_indices = options.attached_node_draw_indices;

        return Ok(polyfill);
    }

    pub fn to_options(&self) -> PolyfillOptions {
        PolyfillOptions {
            anchor_node_draw_index: self.anchor_node_draw_index,
            color: self.color,
            use_polyfill_color: self.use_polyfill_color,
            attached_node_draw_indices: self.attached_node_draw_indices.clone(),
        }
    }

    pub fn set_attached_node_draw_indices(
        &mut self,
        draw_indices: Vec<DrawOrderIndex>,
        stickfigure: Stickfigure,
    ) -> Result<(), StickfigureError> {
        let missing_indices = stickfigure.missing_draw_indices(&draw_indices);
        if missing_indices.iter().count() > 0 {
            return Err(StickfigureError::InvalidDrawIndices(
                format!("{:?}", missing_indices),
                format!("Cannot set attached node draw indices to invalid draw order indices."),
            ));
        }

        let mut seen = HashSet::new();
        let unique_draw_indices: Vec<DrawOrderIndex> = draw_indices
            .into_iter()
            .filter(|x| seen.insert(*x))
            .collect();

        self.attached_node_draw_indices = unique_draw_indices;

        Ok(())
    }

    pub fn set_anchor_node_draw_index(
        &mut self,
        draw_index: DrawOrderIndex,
        stickfigure: Stickfigure,
    ) -> Result<(), StickfigureError> {
        if !stickfigure.draw_index_exists(draw_index) {
            return Err(StickfigureError::InvalidDrawIndex(
                draw_index.0,
                format!("Cannot set anchor node draw index to invalid draw order index."),
            ));
        }

        if stickfigure.draw_index_is_polyfill_anchor(draw_index) {
            return Err(StickfigureError::NodeIsAlreadyAnchor(
                draw_index.0,
                format!("Cannot create polyfill with anchor that is already occupied."),
            ));
        }

        Ok(self.anchor_node_draw_index = draw_index)
    }

    pub fn insert_attached_node_draw_indices_after(
        &mut self,
        draw_indices: Vec<DrawOrderIndex>,
        insert_after_draw_index: DrawOrderIndex,
        stickfigure: Stickfigure,
    ) -> Result<(), StickfigureError> {
        let missing_indices = stickfigure.missing_draw_indices(&draw_indices);
        if missing_indices.iter().count() > 0 {
            return Err(StickfigureError::InvalidDrawIndices(
                format!("{:?}", missing_indices),
                format!("Cannot insert attached node draw indices that include invalid draw order indices."),
            ));
        }

        if let Some(vec_index) = self
            .attached_node_draw_indices
            .iter()
            .position(|index| *index == insert_after_draw_index)
        {
            self.attached_node_draw_indices
                .splice(vec_index + 1..vec_index + 1, draw_indices);
        } else {
            return Err(StickfigureError::InvalidDrawIndex(
                insert_after_draw_index.0,
                format!("Cannot insert attached node draw indices after a node draw index that is not an attached node draw index of this polyfill."),
            ));
        };

        Ok(())
    }

    pub fn insert_attached_node_draw_indices_before(
        &mut self,
        draw_indices: Vec<DrawOrderIndex>,
        insert_before_draw_index: DrawOrderIndex,
        stickfigure: Stickfigure,
    ) -> Result<(), StickfigureError> {
        let missing_indices = stickfigure.missing_draw_indices(&draw_indices);
        if missing_indices.iter().count() > 0 {
            return Err(StickfigureError::InvalidDrawIndices(
                format!("{:?}", missing_indices),
                format!("Cannot insert attached node draw indices that include invalid draw order indices."),
            ));
        }

        if let Some(vec_index) = self
            .attached_node_draw_indices
            .iter()
            .position(|index| *index == insert_before_draw_index)
        {
            self.attached_node_draw_indices
                .splice(vec_index..vec_index, draw_indices);
        } else {
            return Err(StickfigureError::InvalidDrawIndex(
                insert_before_draw_index.0,
                format!("Cannot insert attached node draw indices after a node draw index that is not an attached node draw index of this polyfill."),
            ));
        };

        Ok(())
    }

    pub fn remove_attached_node_draw_indices(
        &mut self,
        draw_indices: Vec<DrawOrderIndex>,
        stickfigure: Stickfigure,
    ) -> Result<(), StickfigureError> {
        let missing_indices = stickfigure.missing_draw_indices(&draw_indices);
        if missing_indices.iter().count() > 0 {
            return Err(StickfigureError::InvalidDrawIndices(
                format!("{:?}", missing_indices),
                format!("Cannot remove attached node draw indices that include invalid draw order indices."),
            ));
        }

        let indices: Vec<DrawOrderIndex> = self
            .attached_node_draw_indices
            .iter()
            .map(|i| *i)
            .filter(|draw_index| !draw_indices.contains(draw_index))
            .collect();

        self.attached_node_draw_indices = indices;

        Ok(())
    }

    pub fn try_set_attached_node_draw_indices(
        &mut self,
        draw_indices: Vec<DrawOrderIndex>,
        stickfigure: Stickfigure,
    ) -> Vec<DrawOrderIndex> {
        let missing_indices = stickfigure.missing_draw_indices(&draw_indices);

        let valid_indices = draw_indices
            .iter()
            .map(|i| *i)
            .filter(|index| !missing_indices.contains(index))
            .collect();

        self.attached_node_draw_indices = valid_indices;

        missing_indices
    }

    // below methods are commented out because I don't know if they'd be worth existing since there would have to be some kind of special return for if the provided index to insert after/before is invalid.. which kind of defeats the point of the method. At that point just handle the error of the non try versions lol
    // pub fn try_insert_attached_node_draw_indices_after(&mut self, draw_indices: Vec<DrawOrderIndex>, insert_after_draw_index: DrawOrderIndex, stickfigure: Stickfigure) -> Vec<DrawOrderIndex> {
    //     let missing_indices = stickfigure.missing_draw_indices(&draw_indices);

    //     let valid_indices: Vec<DrawOrderIndex> = draw_indices.iter().map(|i| *i).filter(|index| !missing_indices.contains(index)).collect();

    //     self.attached_node_draw_indices.splice(insert_after_draw_index + 1..insert_after_draw_index + 1, valid_indices);

    //     missing_indices
    // }

    // pub fn try_insert_attached_node_draw_indices_before(&mut self, draw_indices: Vec<DrawOrderIndex>, insert_before_draw_index: DrawOrderIndex, stickfigure: Stickfigure) -> Vec<DrawOrderIndex> {
    //     let missing_indices = stickfigure.missing_draw_indices(&draw_indices);

    //     let valid_indices: Vec<DrawOrderIndex> = draw_indices.iter().map(|i| *i).filter(|index| !missing_indices.contains(index)).collect();

    //     self.attached_node_draw_indices.splice(insert_before_draw_index..insert_before_draw_index, valid_indices);

    //     missing_indices
    // }

    pub fn try_remove_attached_node_draw_indices(
        &mut self,
        draw_indices: Vec<DrawOrderIndex>,
        stickfigure: Stickfigure,
    ) -> Vec<DrawOrderIndex> {
        let missing_indices = stickfigure.missing_draw_indices(&draw_indices);

        let valid_indices: Vec<DrawOrderIndex> = draw_indices
            .iter()
            .map(|i| *i)
            .filter(|index| !missing_indices.contains(index))
            .collect();

        let indices: Vec<DrawOrderIndex> = self
            .attached_node_draw_indices
            .iter()
            .map(|i| *i)
            .filter(|draw_index| !valid_indices.contains(draw_index))
            .collect();

        self.attached_node_draw_indices = indices;

        missing_indices
    }
}
