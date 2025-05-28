use byteorder::{BigEndian, ByteOrder, LittleEndian, ReadBytesExt};
use core2::io::{self, Read, Seek, SeekFrom};
extern crate alloc;
use alloc::{rc::Rc, vec::Vec, vec, format};
use miniz_oxide::inflate::decompress_to_vec_zlib;

use core::{cell::RefCell};

use crate::{
    error::*,
    structs::{node::*, polyfill::*, stickfigure::*},
    Color, ConnectorData, ConnectorMethod,
};

fn read_stickfigure_header<E: ByteOrder>(
    reader: &mut impl Read,
    stickfigure: &mut Stickfigure,
) -> Result<(), StickfigureError> {
    stickfigure.version = reader
        .read_i32::<E>()
        .or_else(|err| return Err(StickfigureError::Io(err)))?;
    if stickfigure.version >= 403 {
        stickfigure.build = reader
            .read_i32::<E>()
            .or_else(|err| return Err(StickfigureError::Io(err)))?;
    }
    stickfigure.scale = reader
        .read_f32::<E>()
        .or_else(|err| return Err(StickfigureError::Io(err)))?;
    stickfigure.color = Color {
        alpha: reader
            .read_u8()
            .or_else(|err| return Err(StickfigureError::Io(err)))?,
        blue: reader
            .read_u8()
            .or_else(|err| return Err(StickfigureError::Io(err)))?,
        green: reader
            .read_u8()
            .or_else(|err| return Err(StickfigureError::Io(err)))?,
        red: reader
            .read_u8()
            .or_else(|err| return Err(StickfigureError::Io(err)))?,
    };

    Ok(())
}

pub fn read_stickfigure<R: Read + Seek>(reader: &mut R) -> Result<Stickfigure, LibraryError> {
    let mut stickfigure = Stickfigure::default();

    read_stickfigure_header::<BigEndian>(reader, &mut stickfigure)?;

    let mut version = stickfigure.version;
    let mut build = stickfigure.build;

    if version > Stickfigure::default().version || version < 160 {
        reader.seek(SeekFrom::Start(0));
        let mut compressed = Vec::new();
        reader.read_to_end(&mut compressed).map_err(StickfigureError::Io)?;

        let decompressed = decompress_to_vec_zlib(&compressed)
        .map_err(|_| StickfigureError::Io(io::Error::new(io::ErrorKind::Other, "")))?;

        let mut decompressed_str = decompressed.as_slice();

        stickfigure = Stickfigure::default();
        read_stickfigure_header::<LittleEndian>(&mut decompressed_str, &mut stickfigure)?;
        version = stickfigure.version;
        build = stickfigure.build;

        if version > Stickfigure::default().version {
            return Err(LibraryError::UnsupportedVersion(version));
        } else if version == Stickfigure::default().version && build > Stickfigure::default().build {
            return Err(LibraryError::UnsupportedBuild(version, build));
        }

        read_child_nodes::<LittleEndian>(
            &mut decompressed_str,
            stickfigure.version,
            stickfigure.build,
            DrawOrderIndex(-1),
            1,
            &mut stickfigure,
            Vec::from(vec![false])
        )?;
    } else {
        if version > Stickfigure::default().version {
            return Err(LibraryError::UnsupportedVersion(version));
        } else if version == Stickfigure::default().version && build > Stickfigure::default().build {
            return Err(LibraryError::UnsupportedBuild(version, build));
        }
    
        read_child_nodes::<BigEndian>(
            reader,
            stickfigure.version,
            stickfigure.build,
            DrawOrderIndex(-1),
            1,
            &mut stickfigure,
            Vec::from(vec![false])
        )?;
    
        if stickfigure.version >= 230 {
            stickfigure.polyfills = read_polyfill_header::<BigEndian>(reader)?;
        }

        if stickfigure.version >= 403 && stickfigure.build >= 38 {
            let number_of_connectors = reader
                .read_i32::<BigEndian>()
                .or_else(|err| return Err(StickfigureError::Io(err)))?;

            for _ in 0..number_of_connectors {
                let my_draw_index = reader
                    .read_i32::<BigEndian>()
                    .or_else(|err| return Err(StickfigureError::Io(err)))?;
                let end_draw_index = reader
                    .read_i32::<BigEndian>()
                    .or_else(|err| return Err(StickfigureError::Io(err)))?;
                
                let connector_node = stickfigure.get_node(DrawOrderIndex(my_draw_index)).ok_or_else(|| StickfigureError::InvalidDrawIndex(my_draw_index, format!("Attempted to get connector node defined in .nodes file that does not exist(?)")))?;
                if !stickfigure.draw_index_exists(DrawOrderIndex(end_draw_index)) {
                    return Err(StickfigureError::InvalidDrawIndex(end_draw_index, format!("Attempted to get end connector node defined in .nodes file that does not exist(?)")))?;
                }

                let connector_data= &mut connector_node.borrow_mut().connector_data;

                match connector_data {
                    Some(data) => {
                        data.end_node_draw_index = DrawOrderIndex(end_draw_index);
                    },
                    None => return Err(StickfigureError::GenericError(format!("Attempted to get undefined connector data of node (while reading .nodes file). Node {:?}", my_draw_index)))?,
                }
            }
        }
    }

    Ok(stickfigure)
}

fn read_child_nodes<E: ByteOrder>(
    reader: &mut impl Read,
    version: i32,
    build: i32,
    parent_draw_index: DrawOrderIndex,
    number_of_child_nodes_to_read: i32,
    stickfigure: &mut Stickfigure,
    connector_booleans_to_use: Vec<bool>
) -> Result<(), StickfigureError> {
    for i in 0..number_of_child_nodes_to_read {

        let is_connector = connector_booleans_to_use.get(i as usize).unwrap_or(&false);

        let (node, number_of_child_nodes, connector_booleans) = read_node::<E>(reader, version, build, *is_connector)?;

        let number_of_child_nodes = number_of_child_nodes;
        let draw_index = node.draw_order_index;

        if parent_draw_index.0 == -1 {
            stickfigure.add_root_node();
        } else {
            stickfigure.add_node_at_unique_index(node, parent_draw_index, draw_index)?;
        }

        read_child_nodes::<E>(
            reader,
            version,
            build,
            draw_index,
            number_of_child_nodes,
            stickfigure,
            connector_booleans
        )?;
    }

    Ok(())
}

fn read_node<E: ByteOrder>(
    reader: &mut impl Read,
    version: i32,
    build: i32, is_connector: bool
) -> Result<(Node, i32, Vec<bool>), StickfigureError> {
    let mut node = Node::new();

    if is_connector {
        let mut connector_data = ConnectorData::default();

        connector_data.local_x = reader
            .read_f32::<E>()
            .or_else(|err| return Err(StickfigureError::Io(err)))?;
        connector_data.local_y = reader
            .read_f32::<E>()
            .or_else(|err| return Err(StickfigureError::Io(err)))?;
        connector_data.percent = reader
            .read_f32::<E>()
            .or_else(|err| return Err(StickfigureError::Io(err)))?;
        if build >= 44 {
            connector_data.percent_default = reader
                .read_f32::<E>()
                .or_else(|err| return Err(StickfigureError::Io(err)))?;
        } else {
            connector_data.percent_default = connector_data.percent;
        }
        connector_data.value = reader
            .read_f32::<E>()
            .or_else(|err| return Err(StickfigureError::Io(err)))?;
        let method = reader
            .read_i32::<E>()
            .or_else(|err| return Err(StickfigureError::Io(err)))?;
        connector_data.method = ConnectorMethod::from_integer(method as i8).unwrap_or(ConnectorMethod::default());
        connector_data.reversed = reader
            .read_u8()
            .or_else(|err| return Err(StickfigureError::Io(err)))?
            != 0;
        if build >= 65 {
            connector_data.smart_stretch_ancestral_value = reader
                .read_f32::<E>()
                .or_else(|err| return Err(StickfigureError::Io(err)))?;
        } else {
            connector_data.smart_stretch_ancestral_value = 1.0;
        }
        node.connector_data = Some(connector_data);
    }

    node.node_type = NodeType::from_integer(
        reader
            .read_i8()
            .or_else(|err| return Err(StickfigureError::Io(err)))?,
    )
    .unwrap_or_default();
    node.draw_order_index.0 = reader
        .read_i32::<E>()
        .or_else(|err| return Err(StickfigureError::Io(err)))?;
    node.is_static = reader
        .read_u8()
        .or_else(|err| return Err(StickfigureError::Io(err)))?
        != 0;
    node.is_stretchy = reader
        .read_u8()
        .or_else(|err| return Err(StickfigureError::Io(err)))?
        != 0;
    if version >= 403 && build >= 48 {
        node.is_floaty = reader
            .read_u8()
            .or_else(|err| return Err(StickfigureError::Io(err)))?
            != 0;
    }
    if version >= 248 {
        node.is_smart_stretch = reader
            .read_u8()
            .or_else(|err| return Err(StickfigureError::Io(err)))?
            != 0;
    }
    if version >= 252 {
        node.do_not_apply_smart_stretch = reader
            .read_u8()
            .or_else(|err| return Err(StickfigureError::Io(err)))?
            != 0;
    }
    if version >= 403 && build >= 50 {
        node.smart_stretch_reset_impulse = reader
            .read_u8()
            .or_else(|err| return Err(StickfigureError::Io(err)))?
            != 0;
    }
    node.use_segment_color = reader
        .read_u8()
        .or_else(|err| return Err(StickfigureError::Io(err)))?
        != 0;

    if version >= 256 {
        node.use_circle_outline = reader
            .read_u8()
            .or_else(|err| return Err(StickfigureError::Io(err)))?
            != 0;
    }
    if version >= 403 && build >= 21 {
        node.circle_is_hollow = reader
            .read_u8()
            .or_else(|err| return Err(StickfigureError::Io(err)))?
            != 0;
    }
    if version >= 176 {
        node.use_gradient = reader
            .read_u8()
            .or_else(|err| return Err(StickfigureError::Io(err)))?
            != 0;
        node.reverse_gradient = reader
            .read_u8()
            .or_else(|err| return Err(StickfigureError::Io(err)))?
            != 0;
    }
    if version >= 403 && build >= 20 {
        let gradient_mode_short = reader
        .read_i16::<E>()
        .or_else(|err| return Err(StickfigureError::Io(err)))?;

        node.gradient_mode = GradientMode::from_integer(gradient_mode_short as i8).unwrap_or(GradientMode::default());
    }
    node.use_segment_scale = reader
        .read_u8()
        .or_else(|err| return Err(StickfigureError::Io(err)))?
        != 0;
    node.local_x = reader
        .read_f32::<E>()
        .or_else(|err| return Err(StickfigureError::Io(err)))?;
    node.local_y = reader
        .read_f32::<E>()
        .or_else(|err| return Err(StickfigureError::Io(err)))?;
    node.scale = reader
        .read_f32::<E>()
        .or_else(|err| return Err(StickfigureError::Io(err)))?;
    node.default_length = reader
        .read_f32::<E>()
        .or_else(|err| return Err(StickfigureError::Io(err)))?;
    node.length = reader
        .read_f32::<E>()
        .or_else(|err| return Err(StickfigureError::Io(err)))?;
    node.default_thickness = reader
        .read_i32::<E>()
        .or_else(|err| return Err(StickfigureError::Io(err)))?;
    node.thickness = reader
        .read_i32::<E>()
        .or_else(|err| return Err(StickfigureError::Io(err)))?;

    if version >= 320 {
        node.segment_curve_radius_and_default_curve_radius = reader
            .read_i32::<E>()
            .or_else(|err| return Err(StickfigureError::Io(err)))?;
    }
    if version >= 403 && build >= 20 {
        node.curve_circulization = reader
            .read_u8()
            .or_else(|err| return Err(StickfigureError::Io(err)))?
            != 0;
    }
    if version >= 403 && build >= 21 {
        node.segment_curve_polyfill_precision = reader
            .read_i16::<E>()
            .or_else(|err| return Err(StickfigureError::Io(err)))?;
    }
    if version >= 256 {
        node.half_arc = reader
            .read_u8()
            .or_else(|err| return Err(StickfigureError::Io(err)))?
            != 0;
        node.right_triangle_direction = reader
            .read_i16::<E>()
            .or_else(|err| return Err(StickfigureError::Io(err)))?;
    }
    if version >= 300 {
        node.triangle_upside_down = reader
            .read_u8()
            .or_else(|err| return Err(StickfigureError::Io(err)))?
            != 0;
    }
    if version >= 403 && build >= 36 {
        if build < 64 {
            node.trapezoid_thickness_start = reader
                .read_i32::<E>()
                .or_else(|err| return Err(StickfigureError::Io(err)))? as f32;
            node.trapezoid_thickness_end = reader
                .read_i32::<E>()
                .or_else(|err| return Err(StickfigureError::Io(err)))? as f32;
        } else {
            node.trapezoid_thickness_start = reader
                .read_f32::<E>()
                .or_else(|err| return Err(StickfigureError::Io(err)))?;
            node.trapezoid_thickness_end = reader
                .read_f32::<E>()
                .or_else(|err| return Err(StickfigureError::Io(err)))?;
        }
    }
    if version >= 403 && build == 36 {
        reader.read_i32::<E>();
        reader.read_i32::<E>();
        node.use_trapezoid_thickness_start = reader
            .read_u8()
            .or_else(|err| return Err(StickfigureError::Io(err)))?
            != 0;
        node.use_trapezoid_thickness_end = reader
            .read_u8()
            .or_else(|err| return Err(StickfigureError::Io(err)))?
            != 0;
    }
    if version >= 256 && build != 36{
        node.trapezoid_top_thickness_ratio = reader
            .read_f32::<E>()
            .or_else(|err| return Err(StickfigureError::Io(err)))?;
    }
    if version >= 403 && build >= 36 {
        node.trapezoid_is_rounded_start = reader
            .read_u8()
            .or_else(|err| return Err(StickfigureError::Io(err)))?
            != 0;
        node.trapezoid_is_rounded_end = reader
            .read_u8()
            .or_else(|err| return Err(StickfigureError::Io(err)))?
            != 0;
    }
    if version >= 256 {
        node.num_polygon_vertices = reader
            .read_i16::<E>()
            .or_else(|err| return Err(StickfigureError::Io(err)))?;
    }
    if version >= 248 {
        node.default_local_angle = reader
            .read_f32::<E>()
            .or_else(|err| return Err(StickfigureError::Io(err)))?;
    }
    node.local_angle = reader
        .read_f32::<E>()
        .or_else(|err| return Err(StickfigureError::Io(err)))?;
    if version >= 248 {
        node.default_angle = reader
            .read_f32::<E>()
            .or_else(|err| return Err(StickfigureError::Io(err)))?;
    }

    node.color = Color {
        alpha: reader
            .read_u8()
            .or_else(|err| return Err(StickfigureError::Io(err)))?,
        blue: reader
            .read_u8()
            .or_else(|err| return Err(StickfigureError::Io(err)))?,
        green: reader
            .read_u8()
            .or_else(|err| return Err(StickfigureError::Io(err)))?,
        red: reader
            .read_u8()
            .or_else(|err| return Err(StickfigureError::Io(err)))?,
    };

    if version >= 176 {
        node.gradient_color = Color {
            alpha: reader
                .read_u8()
                .or_else(|err| return Err(StickfigureError::Io(err)))?,
            blue: reader
                .read_u8()
                .or_else(|err| return Err(StickfigureError::Io(err)))?,
            green: reader
                .read_u8()
                .or_else(|err| return Err(StickfigureError::Io(err)))?,
            red: reader
                .read_u8()
                .or_else(|err| return Err(StickfigureError::Io(err)))?,
        };
    }
    if version >= 256 {
        node.circle_outline_color = Color {
            alpha: reader
                .read_u8()
                .or_else(|err| return Err(StickfigureError::Io(err)))?,
            blue: reader
                .read_u8()
                .or_else(|err| return Err(StickfigureError::Io(err)))?,
            green: reader
                .read_u8()
                .or_else(|err| return Err(StickfigureError::Io(err)))?,
            red: reader
                .read_u8()
                .or_else(|err| return Err(StickfigureError::Io(err)))?,
        };
    }

    if version >= 403 && build >= 39 {
        node.is_angle_locked = reader
            .read_u8()
            .or_else(|err| return Err(StickfigureError::Io(err)))?
            != 0;
        if build <= 50 {
            node.is_angle_locked = false;
        }
    }
    if version >= 403 && (build >= 39 && build <= 50) {
        reader.read_f32::<E>();
    }
    if version >= 403 && build >= 51 {
        node.angle_lock_is_main_node = reader
            .read_u8()
            .or_else(|err| return Err(StickfigureError::Io(err)))?
            != 0;
        if build < 56 {
            node.angle_lock_is_main_node = !node.angle_lock_is_main_node;
        }
    }
    if version >= 403 && (build >= 51 && build <= 56) {
        node.angle_lock_offset_minuend = reader
            .read_f32::<E>()
            .or_else(|err| return Err(StickfigureError::Io(err)))?;
        node.angle_lock_offset_subtrahend = reader
            .read_f32::<E>()
            .or_else(|err| return Err(StickfigureError::Io(err)))?;
    }
    if version >= 403 && build >= 57 {
        node.angle_lock_offset = reader
            .read_f32::<E>()
            .or_else(|err| return Err(StickfigureError::Io(err)))?;
    }
    if version >= 403 && build >= 63 {
        node.angle_lock_relative_start = reader
            .read_f32::<E>()
            .or_else(|err| return Err(StickfigureError::Io(err)))?;
    }
    if version >= 403 && build >= 67 {
        node.angle_lock_stickfigure_start = reader
            .read_f32::<E>()
            .or_else(|err| return Err(StickfigureError::Io(err)))?;
    }
    if version >= 403 && build >= 63 {
        node.angle_lock_relative_multiplier = reader
            .read_i8()
            .or_else(|err| return Err(StickfigureError::Io(err)))?;
    }
    if version >= 403 && build >= 39 {
        if build <= 40 {
            node.is_drag_locked = reader
            .read_i16::<E>()
            .or_else(|err| return Err(StickfigureError::Io(err)))?
            != 0;
        } else {
            node.is_drag_locked = reader
            .read_u8()
            .or_else(|err| return Err(StickfigureError::Io(err)))?
            != 0;
        }
    }
    if version >= 403 && (build >= 41 && build <= 45) {
        reader.read_i16::<E>();
    }
    if version >= 403 && build >= 46 {
        node.drag_lock_angle = reader
            .read_f32::<E>()
            .or_else(|err| return Err(StickfigureError::Io(err)))?;
    }

    if version >= 403 && build >= 41 {
        node.smart_stretch_multiplier = reader
            .read_f32::<E>()
            .or_else(|err| return Err(StickfigureError::Io(err)))?;
    }
    if version >= 403 && (build >= 41 && build <= 45) {
        reader.read_u8();
    }
    let number_of_child_nodes = reader
        .read_i32::<E>()
        .or_else(|err| return Err(StickfigureError::Io(err)))?;

    let mut connector_booleans = Vec::new();
    if version >= 403 && build > 38 {
        for _ in 0..number_of_child_nodes {
            let bool = reader
            .read_i32::<E>()
            .or_else(|err| return Err(StickfigureError::Io(err)))?
            != 0;
            connector_booleans.push(bool);
        }
    }

    match node.right_triangle_direction {
        -1 => {
            node.triangle_type = TriangleType::RightTriangle;
            node.triangle_flipped = true;
        },
        0 => {
            node.triangle_type = TriangleType::Isosceles;
            node.triangle_flipped = false;
        },
        1 => {
            node.triangle_type = TriangleType::RightTriangle;
            node.triangle_flipped = false;
        },
        _ => {
            node.triangle_type = TriangleType::RightTriangle;
            if node.right_triangle_direction < 0 {
                node.triangle_flipped = true;
            } else {
                node.triangle_flipped = false;
            }
        }
    }

    if node.is_angle_locked {
        if node.angle_lock_is_main_node {
            node.angle_lock_mode = AngleLockMode::Absolute;
        } else {
            node.angle_lock_mode = AngleLockMode::Relative
        }
    } else {
        node.angle_lock_mode = AngleLockMode::None;
    }

    Ok((node, number_of_child_nodes, connector_booleans))
}

fn read_polyfill_header<E: ByteOrder>(
    reader: &mut impl Read,
) -> Result<Vec<Rc<RefCell<Polyfill>>>, StickfigureError> {
    let number_of_polyfills = reader
        .read_i32::<E>()
        .or_else(|err| return Err(StickfigureError::Io(err)))?;

    let mut polyfills = Vec::with_capacity(number_of_polyfills as usize);
    for _ in 0..number_of_polyfills {
        let polyfill = Rc::new(RefCell::new(read_polyfill::<E>(reader)?));
        polyfills.push(polyfill);
    }

    return Ok(polyfills);
}

fn read_polyfill<E: ByteOrder>(reader: &mut impl Read) -> Result<Polyfill, StickfigureError> {
    let mut polyfill = Polyfill::default();

    polyfill.anchor_node_draw_index.0 = reader
        .read_i32::<E>()
        .or_else(|err| return Err(StickfigureError::Io(err)))?;

    let mut buf = [0u8; 4];
    reader
        .read_exact(&mut buf)
        .or_else(|err| return Err(StickfigureError::Io(err)))?;
    let [alpha, blue, green, red] = buf;

    polyfill.color = Color {
        alpha,
        blue,
        green,
        red,
    };
    polyfill.use_polyfill_color = reader
        .read_u8()
        .or_else(|err| return Err(StickfigureError::Io(err)))?
        != 0;
    let number_of_attached_nodes = reader
        .read_i32::<E>()
        .or_else(|err| return Err(StickfigureError::Io(err)))?;

    let attached_node_draw_indices: Vec<DrawOrderIndex> = (0..number_of_attached_nodes)
        .map(|_| reader.read_i32::<E>().map(DrawOrderIndex))
        .collect::<Result<_, _>>()
        .or_else(|err| return Err(StickfigureError::Io(err)))?;

    polyfill.attached_node_draw_indices = attached_node_draw_indices;

    Ok(polyfill)
}
