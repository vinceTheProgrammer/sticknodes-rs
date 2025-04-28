use core::cell::RefCell;

use byteorder::{BigEndian, WriteBytesExt};
use core2::io::{Cursor, Write};
extern crate alloc;
use alloc::{format, rc::Rc, vec, vec::Vec};

use crate::{
    error::*,
    structs::{node::*, polyfill::*, stickfigure::*},
};

fn write_stickfigure_header(stickfigure: &Stickfigure) -> Result<Vec<u8>, StickfigureError> {
    let mut byte_vec = Vec::new();

    let mut buffer = [0u8; 4]; // fixed-size buffer. should be set to max size of bytes written in this section.
    let mut cursor = Cursor::new(&mut buffer[..]);
    cursor
        .write_i32::<BigEndian>(stickfigure.version)
        .or_else(|err| return Err(StickfigureError::Io(err)))?;
    byte_vec.append(&mut Vec::from(buffer));

    if stickfigure.version >= 403 {
        let mut buffer_ = [0u8; 4]; // fixed-size buffer. should be set to max size of bytes written in this section.
        let mut cursor_ = Cursor::new(&mut buffer_[..]);
        cursor_
            .write_i32::<BigEndian>(stickfigure.build)
            .or_else(|err| return Err(StickfigureError::Io(err)))?;
        byte_vec.append(&mut Vec::from(buffer_));
    }

    let mut buffer2 = [0u8; 8]; // fixed-size buffer. should be set to max size of bytes written in this section.
    let mut cursor2 = Cursor::new(&mut buffer2[..]);
    cursor2
        .write_f32::<BigEndian>(stickfigure.scale)
        .or_else(|err| return Err(StickfigureError::Io(err)))?;
    cursor2
        .write_all(&[
            stickfigure.color.alpha,
            stickfigure.color.blue,
            stickfigure.color.green,
            stickfigure.color.red,
        ])
        .or_else(|err| return Err(StickfigureError::Io(err)))?;
    byte_vec.append(&mut Vec::from(buffer2));

    Ok(byte_vec)
}

pub fn write_stickfigure(stickfigure: &Stickfigure) -> Result<Vec<u8>, LibraryError> {
    let mut byte_vec = Vec::new();

    byte_vec.append(&mut write_stickfigure_header(&stickfigure)?);

    byte_vec.append(&mut write_child_nodes(
        stickfigure.version,
        stickfigure.build,
        DrawOrderIndex(0),
        stickfigure,
    )?);

    if stickfigure.version >= 230 {
        byte_vec.append(&mut write_polyfill_header(&stickfigure)?);
    }

    Ok(byte_vec)
}

fn write_child_nodes(
    version: i32,
    build: i32,
    draw_index: DrawOrderIndex,
    stickfigure: &Stickfigure,
) -> Result<Vec<u8>, StickfigureError> {
    let mut byte_vec = Vec::new();

    if !stickfigure.all_draw_indices_exist(&vec![draw_index]) {
        return Err(StickfigureError::InvalidDrawIndex(
            draw_index.0,
            format!("Cannot finish reading this stickfigure file."),
        ));
    }

    let node_index = stickfigure.node_index_from_draw_order(draw_index);

    if let Some(node) = stickfigure.nodes.node_weight(node_index) {
        byte_vec.append(&mut write_node(version, build, node)?);

        let children = stickfigure.get_children(draw_index);
        let number_of_child_nodes = children.len() as i32;

        let mut buffer = [0u8; 4]; // fixed-size buffer. should be set to max size of bytes written in this section.
        let mut cursor = Cursor::new(&mut buffer[..]);
        cursor
            .write_i32::<BigEndian>(number_of_child_nodes)
            .or_else(|err| return Err(StickfigureError::Io(err)))?;
        byte_vec.append(&mut Vec::from(buffer));

        for child_draw_index in children.iter().rev() {
            byte_vec.append(&mut write_child_nodes(
                version,
                build,
                *child_draw_index,
                stickfigure,
            )?);
        }
    }

    Ok(byte_vec)
}

fn write_node(
    version: i32,
    build: i32,
    rc_node: &Rc<RefCell<Node>>,
) -> Result<Vec<u8>, StickfigureError> {
    let node = rc_node.borrow();

    let mut byte_vec = Vec::new();

    let mut buffer1 = [0u8; 7]; // fixed-size buffer. should be set to max size of bytes written in this section.
    let mut cursor1 = Cursor::new(&mut buffer1[..]);
    cursor1
        .write_i8(node.node_type.to_integer())
        .or_else(|err| return Err(StickfigureError::Io(err)))?;
    cursor1
        .write_i32::<BigEndian>(node.draw_order_index.0)
        .or_else(|err| return Err(StickfigureError::Io(err)))?;
    cursor1
        .write_u8(node.is_static as u8)
        .or_else(|err| return Err(StickfigureError::Io(err)))?;
    cursor1
        .write_u8(node.is_stretchy as u8)
        .or_else(|err| return Err(StickfigureError::Io(err)))?;
    byte_vec.append(&mut Vec::from(buffer1));

    if version >= 248 {
        let mut buffer_ = [0u8; 1]; // fixed-size buffer. should be set to max size of bytes written in this section.
        let mut cursor_ = Cursor::new(&mut buffer_[..]);
        cursor_
            .write_u8(node.is_smart_stretch as u8)
            .or_else(|err| return Err(StickfigureError::Io(err)))?;
        byte_vec.append(&mut Vec::from(buffer_));
    }
    if version >= 252 {
        let mut buffer_ = [0u8; 1]; // fixed-size buffer. should be set to max size of bytes written in this section.
        let mut cursor_ = Cursor::new(&mut buffer_[..]);
        cursor_
            .write_u8(node.do_not_apply_smart_stretch as u8)
            .or_else(|err| return Err(StickfigureError::Io(err)))?;
        byte_vec.append(&mut Vec::from(buffer_));
    }

    let mut buffer2 = [0u8; 1]; // fixed-size buffer. should be set to max size of bytes written in this section.
    let mut cursor2 = Cursor::new(&mut buffer2[..]);
    cursor2
        .write_u8(node.use_segment_color as u8)
        .or_else(|err| return Err(StickfigureError::Io(err)))?;
    byte_vec.append(&mut Vec::from(buffer2));

    if version >= 256 {
        let mut buffer_ = [0u8; 1]; // fixed-size buffer. should be set to max size of bytes written in this section.
        let mut cursor_ = Cursor::new(&mut buffer_[..]);
        cursor_
            .write_u8(node.use_circle_outline as u8)
            .or_else(|err| return Err(StickfigureError::Io(err)))?;
        byte_vec.append(&mut Vec::from(buffer_));
    }
    if version >= 403 && build >= 21 {
        let mut buffer_ = [0u8; 1]; // fixed-size buffer. should be set to max size of bytes written in this section.
        let mut cursor_ = Cursor::new(&mut buffer_[..]);
        cursor_
            .write_u8(node.circle_is_hollow as u8)
            .or_else(|err| return Err(StickfigureError::Io(err)))?;
        byte_vec.append(&mut Vec::from(buffer_));
    }
    if version >= 176 {
        let mut buffer_ = [0u8; 2]; // fixed-size buffer. should be set to max size of bytes written in this section.
        let mut cursor_ = Cursor::new(&mut buffer_[..]);
        cursor_
            .write_u8(node.use_gradient as u8)
            .or_else(|err| return Err(StickfigureError::Io(err)))?;
        cursor_
            .write_u8(node.reverse_gradient as u8)
            .or_else(|err| return Err(StickfigureError::Io(err)))?;
        byte_vec.append(&mut Vec::from(buffer_));
    }
    if version >= 403 && build >= 20 {
        let mut buffer_ = [0u8; 2]; // fixed-size buffer. should be set to max size of bytes written in this section.
        let mut cursor_ = Cursor::new(&mut buffer_[..]);
        cursor_
            .write_i16::<BigEndian>(node.gradient_mode)
            .or_else(|err| return Err(StickfigureError::Io(err)))?;
        byte_vec.append(&mut Vec::from(buffer_));
    }

    let mut buffer3 = [0u8; 29]; // fixed-size buffer. should be set to max size of bytes written in this section.
    let mut cursor3 = Cursor::new(&mut buffer3[..]);
    cursor3
        .write_u8(node.use_segment_scale as u8)
        .or_else(|err| return Err(StickfigureError::Io(err)))?;
    cursor3
        .write_f32::<BigEndian>(node.local_x)
        .or_else(|err| return Err(StickfigureError::Io(err)))?;
    cursor3
        .write_f32::<BigEndian>(node.local_y)
        .or_else(|err| return Err(StickfigureError::Io(err)))?;
    cursor3
        .write_f32::<BigEndian>(node.scale)
        .or_else(|err| return Err(StickfigureError::Io(err)))?;
    cursor3
        .write_f32::<BigEndian>(node.default_length)
        .or_else(|err| return Err(StickfigureError::Io(err)))?;
    cursor3
        .write_f32::<BigEndian>(node.length)
        .or_else(|err| return Err(StickfigureError::Io(err)))?;
    cursor3
        .write_i32::<BigEndian>(node.default_thickness)
        .or_else(|err| return Err(StickfigureError::Io(err)))?;
    cursor3
        .write_i32::<BigEndian>(node.thickness)
        .or_else(|err| return Err(StickfigureError::Io(err)))?;
    byte_vec.append(&mut Vec::from(buffer3));

    if version >= 320 {
        let mut buffer_ = [0u8; 4]; // fixed-size buffer. should be set to max size of bytes written in this section.
        let mut cursor_ = Cursor::new(&mut buffer_[..]);
        cursor_
            .write_i32::<BigEndian>(node.segment_curve_radius_and_default_curve_radius)
            .or_else(|err| return Err(StickfigureError::Io(err)))?;
        byte_vec.append(&mut Vec::from(buffer_));
    }
    if version >= 403 && build >= 20 {
        let mut buffer_ = [0u8; 1]; // fixed-size buffer. should be set to max size of bytes written in this section.
        let mut cursor_ = Cursor::new(&mut buffer_[..]);
        cursor_
            .write_u8(node.curve_circulization as u8)
            .or_else(|err| return Err(StickfigureError::Io(err)))?;
        byte_vec.append(&mut Vec::from(buffer_));
    }
    if version >= 403 && build >= 21 {
        let mut buffer_ = [0u8; 2]; // fixed-size buffer. should be set to max size of bytes written in this section.
        let mut cursor_ = Cursor::new(&mut buffer_[..]);
        cursor_
            .write_i16::<BigEndian>(node.segment_curve_polyfill_precision)
            .or_else(|err| return Err(StickfigureError::Io(err)))?;
        byte_vec.append(&mut Vec::from(buffer_));
    }
    if version >= 256 {
        let mut buffer_ = [0u8; 10]; // fixed-size buffer. should be set to max size of bytes written in this section.
        let mut cursor_ = Cursor::new(&mut buffer_[..]);
        cursor_
            .write_u8(node.half_arc as u8)
            .or_else(|err| return Err(StickfigureError::Io(err)))?;
        cursor_
            .write_i16::<BigEndian>(node.right_triangle_direction)
            .or_else(|err| return Err(StickfigureError::Io(err)))?;
        cursor_
            .write_u8(node.triangle_upside_down as u8)
            .or_else(|err| return Err(StickfigureError::Io(err)))?;
        cursor_
            .write_f32::<BigEndian>(node.trapezoid_top_thickness_ratio)
            .or_else(|err| return Err(StickfigureError::Io(err)))?;
        cursor_
            .write_i16::<BigEndian>(node.num_polygon_vertices)
            .or_else(|err| return Err(StickfigureError::Io(err)))?;
        byte_vec.append(&mut Vec::from(buffer_));
    }
    if version >= 248 {
        let mut buffer_ = [0u8; 4]; // fixed-size buffer. should be set to max size of bytes written in this section.
        let mut cursor_ = Cursor::new(&mut buffer_[..]);
        cursor_
            .write_f32::<BigEndian>(node.default_local_angle)
            .or_else(|err| return Err(StickfigureError::Io(err)))?;
        byte_vec.append(&mut Vec::from(buffer_));
    }

    let mut buffer4 = [0u8; 4]; // fixed-size buffer. should be set to max size of bytes written in this section.
    let mut cursor4 = Cursor::new(&mut buffer4[..]);
    cursor4
        .write_f32::<BigEndian>(node.local_angle)
        .or_else(|err| return Err(StickfigureError::Io(err)))?;
    byte_vec.append(&mut Vec::from(buffer4));

    if version >= 248 {
        let mut buffer_ = [0u8; 4]; // fixed-size buffer. should be set to max size of bytes written in this section.
        let mut cursor_ = Cursor::new(&mut buffer_[..]);
        cursor_
            .write_f32::<BigEndian>(node.default_angle)
            .or_else(|err| return Err(StickfigureError::Io(err)))?;
        byte_vec.append(&mut Vec::from(buffer_));
    }

    let mut buffer5 = [0u8; 4]; // fixed-size buffer. should be set to max size of bytes written in this section.
    let mut cursor5 = Cursor::new(&mut buffer5[..]);
    cursor5
        .write_all(&[
            node.color.alpha,
            node.color.blue,
            node.color.green,
            node.color.red,
        ])
        .or_else(|err| return Err(StickfigureError::Io(err)))?;
    byte_vec.append(&mut Vec::from(buffer5));

    if version >= 176 {
        let mut buffer_ = [0u8; 4]; // fixed-size buffer. should be set to max size of bytes written in this section.
        let mut cursor_ = Cursor::new(&mut buffer_[..]);
        cursor_
            .write_all(&[
                node.gradient_color.alpha,
                node.gradient_color.blue,
                node.gradient_color.green,
                node.gradient_color.red,
            ])
            .or_else(|err| return Err(StickfigureError::Io(err)))?;
        byte_vec.append(&mut Vec::from(buffer_));
    }
    if version >= 256 {
        let mut buffer_ = [0u8; 4]; // fixed-size buffer. should be set to max size of bytes written in this section.
        let mut cursor_ = Cursor::new(&mut buffer_[..]);
        cursor_
            .write_all(&[
                node.circle_outline_color.alpha,
                node.circle_outline_color.blue,
                node.circle_outline_color.green,
                node.circle_outline_color.red,
            ])
            .or_else(|err| return Err(StickfigureError::Io(err)))?;
        byte_vec.append(&mut Vec::from(buffer_));
    }

    Ok(byte_vec)
}

fn write_polyfill_header(stickfigure: &Stickfigure) -> Result<Vec<u8>, StickfigureError> {
    let mut byte_vec = Vec::new();

    let mut buffer = [0u8; 4]; // fixed-size buffer. should be set to max size of bytes written in this section.
    let mut cursor = Cursor::new(&mut buffer[..]);
    let number_of_polyfills = stickfigure.polyfills.len() as i32;
    cursor
        .write_i32::<BigEndian>(number_of_polyfills)
        .or_else(|err| return Err(StickfigureError::Io(err)))?;

    byte_vec.append(&mut Vec::from(buffer));

    for polyfill in &stickfigure.polyfills {
        byte_vec.append(&mut write_polyfill(&polyfill)?);
    }

    Ok(byte_vec)
}

fn write_polyfill(rc_polyfill: &Rc<RefCell<Polyfill>>) -> Result<Vec<u8>, StickfigureError> {
    let polyfill = rc_polyfill.borrow();

    let mut byte_vec = Vec::new();

    let mut buffer = [0u8; 13]; // fixed-size buffer. should be set to max size of bytes written in this section.
    let mut cursor = Cursor::new(&mut buffer[..]);
    cursor
        .write_i32::<BigEndian>(polyfill.anchor_node_draw_index.0)
        .or_else(|err| return Err(StickfigureError::Io(err)))?;
    cursor
        .write_all(&[
            polyfill.color.alpha,
            polyfill.color.blue,
            polyfill.color.green,
            polyfill.color.red,
        ])
        .or_else(|err| return Err(StickfigureError::Io(err)))?;
    cursor
        .write_u8(if polyfill.use_polyfill_color { 1 } else { 0 })
        .or_else(|err| return Err(StickfigureError::Io(err)))?;

    let number_of_attached_nodes = polyfill.attached_node_draw_indices.len() as i32;
    cursor
        .write_i32::<BigEndian>(number_of_attached_nodes)
        .or_else(|err| return Err(StickfigureError::Io(err)))?;

    byte_vec.append(&mut Vec::from(buffer));

    for draw_index in &polyfill.attached_node_draw_indices {
        let mut local_buffer = [0u8; 4]; // fixed-size buffer. should be set to max size of bytes written in this section.
        let mut local_cursor = Cursor::new(&mut local_buffer[..]);
        local_cursor
            .write_i32::<BigEndian>(draw_index.0)
            .or_else(|err| return Err(StickfigureError::Io(err)))?;
        byte_vec.append(&mut Vec::from(local_buffer));
    }

    Ok(byte_vec)
}
