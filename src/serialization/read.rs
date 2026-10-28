use byteorder::{BigEndian, ByteOrder, LittleEndian, ReadBytesExt};
use miniz_oxide::inflate::decompress_to_vec_zlib;
use std::io::{Read, Seek, SeekFrom};
use std::{cell::RefCell, rc::Rc};

use crate::{
    error::{LibraryError, StickfigureError},
    structs::{
        node::{AngleLockMode, GradientMode, Node, NodeType, TriangleType},
        polyfill::Polyfill,
        stickfigure::{DrawOrderIndex, Stickfigure},
    },
    Color, ConnectorData, ConnectorMethod,
};

fn read_color(reader: &mut impl Read) -> Result<Color, StickfigureError> {
    let mut channels = [0; 4];
    reader
        .read_exact(&mut channels)
        .map_err(StickfigureError::Io)?;
    let [alpha, blue, green, red] = channels;
    Ok(Color {
        alpha,
        blue,
        green,
        red,
    })
}

fn validate_version(version: i32, build: i32) -> Result<(), LibraryError> {
    let supported = Stickfigure::default();

    if version > supported.version {
        return Err(LibraryError::UnsupportedVersion(version));
    }

    if version == supported.version && build > supported.build {
        return Err(LibraryError::UnsupportedBuild(version, build));
    }

    Ok(())
}

fn read_stickfigure_header<E: ByteOrder>(
    reader: &mut impl Read,
    stickfigure: &mut Stickfigure,
) -> Result<(), StickfigureError> {
    stickfigure.version = reader.read_i32::<E>().map_err(StickfigureError::Io)?;
    if stickfigure.version >= 403 {
        stickfigure.build = reader.read_i32::<E>().map_err(StickfigureError::Io)?;
    }
    stickfigure.scale = reader.read_f32::<E>().map_err(StickfigureError::Io)?;
    stickfigure.color = read_color(reader)?;

    Ok(())
}

pub fn read_stickfigure<R: Read + Seek>(reader: &mut R) -> Result<Stickfigure, LibraryError> {
    let mut stickfigure = Stickfigure::default();

    read_stickfigure_header::<BigEndian>(reader, &mut stickfigure)?;

    let mut version = stickfigure.version;
    let mut build = stickfigure.build;

    if version < 160 {
        reader
            .seek(SeekFrom::Start(0))
            .map_err(StickfigureError::Io)?;
        let mut compressed = Vec::new();
        reader
            .read_to_end(&mut compressed)
            .map_err(StickfigureError::Io)?;

        let decompressed = decompress_to_vec_zlib(&compressed)
            .map_err(|err| StickfigureError::DecompressError(format!("{err}")))?;

        let mut decompressed_str = decompressed.as_slice();

        stickfigure = Stickfigure::default();
        read_stickfigure_header::<LittleEndian>(&mut decompressed_str, &mut stickfigure)?;
        version = stickfigure.version;
        build = stickfigure.build;

        validate_version(version, build)?;

        read_child_nodes::<LittleEndian>(
            &mut decompressed_str,
            stickfigure.version,
            stickfigure.build,
            DrawOrderIndex(-1),
            1,
            &mut stickfigure,
            &[false],
        )?;
    } else {
        validate_version(version, build)?;

        read_child_nodes::<BigEndian>(
            reader,
            stickfigure.version,
            stickfigure.build,
            DrawOrderIndex(-1),
            1,
            &mut stickfigure,
            &[false],
        )?;

        if stickfigure.version >= 230 {
            stickfigure.polyfills = read_polyfill_header::<BigEndian>(reader)?;
        }

        if stickfigure.version >= 403 && stickfigure.build >= 38 {
            let number_of_connectors = reader
                .read_i32::<BigEndian>()
                .map_err(StickfigureError::Io)?;

            for _ in 0..number_of_connectors {
                let my_draw_index = reader
                    .read_i32::<BigEndian>()
                    .map_err(StickfigureError::Io)?;
                let end_draw_index = reader
                    .read_i32::<BigEndian>()
                    .map_err(StickfigureError::Io)?;

                let connector_node = stickfigure.get_node(DrawOrderIndex(my_draw_index)).ok_or_else(|| StickfigureError::InvalidDrawIndex(my_draw_index, "Attempted to get connector node defined in .nodes file that does not exist(?)".to_string()))?;
                if !stickfigure.draw_index_exists(DrawOrderIndex(end_draw_index)) {
                    Err(StickfigureError::InvalidDrawIndex(end_draw_index, "Attempted to get end connector node defined in .nodes file that does not exist(?)".to_string()))?;
                }

                let connector_data = &mut connector_node.borrow_mut().connector_data;

                let Some(data) = connector_data else {
                    return Err(StickfigureError::GenericError(format!(
                        "Attempted to get undefined connector data of node (while reading .nodes file). Node {my_draw_index:?}"
                    )))?;
                };
                data.end_node_draw_index = DrawOrderIndex(end_draw_index);
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
    connector_booleans_to_use: &[bool],
) -> Result<(), StickfigureError> {
    for (i, _) in (0..number_of_child_nodes_to_read).enumerate() {
        let is_connector = connector_booleans_to_use.get(i).unwrap_or(&false);

        let (node, number_of_child_nodes, connector_booleans) =
            read_node::<E>(reader, version, build, *is_connector)?;

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
            &connector_booleans,
        )?;
    }

    Ok(())
}

fn read_node<E: ByteOrder>(
    reader: &mut impl Read,
    version: i32,
    build: i32,
    is_connector: bool,
) -> Result<(Node, i32, Vec<bool>), StickfigureError> {
    let mut node = Node::new();

    if is_connector {
        node.connector_data = Some(read_connector_data::<E>(reader, build)?);
    }

    read_node_flags::<E>(reader, &mut node, version, build)?;
    read_node_geometry::<E>(reader, &mut node, version, build)?;
    read_node_colors(reader, &mut node, version)?;
    read_node_angle_lock::<E>(reader, &mut node, version, build)?;

    let (number_of_child_nodes, connector_booleans) =
        read_node_children_metadata::<E>(reader, version, build)?;

    initialize_node_derived_fields(&mut node);

    Ok((node, number_of_child_nodes, connector_booleans))
}

fn read_connector_data<E: ByteOrder>(
    reader: &mut impl Read,
    build: i32,
) -> Result<ConnectorData, StickfigureError> {
    let mut connector_data = ConnectorData {
        local_x: reader.read_f32::<E>().map_err(StickfigureError::Io)?,
        local_y: reader.read_f32::<E>().map_err(StickfigureError::Io)?,
        percent: reader.read_f32::<E>().map_err(StickfigureError::Io)?,
        ..Default::default()
    };

    if build >= 44 {
        connector_data.percent_default = reader.read_f32::<E>().map_err(StickfigureError::Io)?;
    }

    connector_data.percent_default = connector_data.percent;
    connector_data.value = reader.read_f32::<E>().map_err(StickfigureError::Io)?;

    let method = reader.read_i32::<E>().map_err(StickfigureError::Io)?;
    // Allow justified because truncation would not throw out any useful information. There are only two connector methods as of writing this.
    #[allow(clippy::cast_possible_truncation)]
    {
        connector_data.method = ConnectorMethod::from_integer(method as i8).unwrap_or_default();
    }

    connector_data.reversed = reader.read_u8().map_err(StickfigureError::Io)? != 0;
    if build >= 65 {
        connector_data.smart_stretch_ancestral_value =
            reader.read_f32::<E>().map_err(StickfigureError::Io)?;
    }

    connector_data.smart_stretch_ancestral_value = 1.0;

    Ok(connector_data)
}

fn read_node_flags<E: ByteOrder>(
    reader: &mut impl Read,
    node: &mut Node,
    version: i32,
    build: i32,
) -> Result<(), StickfigureError> {
    node.node_type =
        NodeType::from_integer(reader.read_i8().map_err(StickfigureError::Io)?).unwrap_or_default();
    node.draw_order_index.0 = reader.read_i32::<E>().map_err(StickfigureError::Io)?;
    node.is_static = reader.read_u8().map_err(StickfigureError::Io)? != 0;
    node.is_stretchy = reader.read_u8().map_err(StickfigureError::Io)? != 0;

    if version >= 403 && build >= 48 {
        node.is_floaty = reader.read_u8().map_err(StickfigureError::Io)? != 0;
    }
    if version >= 248 {
        node.is_smart_stretch = reader.read_u8().map_err(StickfigureError::Io)? != 0;
    }
    if version >= 252 {
        node.do_not_apply_smart_stretch = reader.read_u8().map_err(StickfigureError::Io)? != 0;
    }
    if version >= 403 && build >= 50 {
        node.smart_stretch_reset_impulse = reader.read_u8().map_err(StickfigureError::Io)? != 0;
    }
    node.use_segment_color = reader.read_u8().map_err(StickfigureError::Io)? != 0;

    if version >= 256 {
        node.use_circle_outline = reader.read_u8().map_err(StickfigureError::Io)? != 0;
    }
    if version >= 403 && build >= 21 {
        node.circle_is_hollow = reader.read_u8().map_err(StickfigureError::Io)? != 0;
    }
    if version >= 176 {
        node.use_gradient = reader.read_u8().map_err(StickfigureError::Io)? != 0;
        node.reverse_gradient = reader.read_u8().map_err(StickfigureError::Io)? != 0;
    }
    if version >= 403 && build >= 20 {
        let gradient_mode = reader.read_i16::<E>().map_err(StickfigureError::Io)?;
        // Allow justified because truncation would not throw out any useful information. There are only two gradient modes as of writing this.
        #[allow(clippy::cast_possible_truncation)]
        {
            node.gradient_mode =
                GradientMode::from_integer(gradient_mode as i8).unwrap_or_default();
        }
    }

    Ok(())
}

fn read_node_geometry<E: ByteOrder>(
    reader: &mut impl Read,
    node: &mut Node,
    version: i32,
    build: i32,
) -> Result<(), StickfigureError> {
    node.use_segment_scale = reader.read_u8().map_err(StickfigureError::Io)? != 0;
    node.local_x = reader.read_f32::<E>().map_err(StickfigureError::Io)?;
    node.local_y = reader.read_f32::<E>().map_err(StickfigureError::Io)?;
    node.scale = reader.read_f32::<E>().map_err(StickfigureError::Io)?;
    node.default_length = reader.read_f32::<E>().map_err(StickfigureError::Io)?;
    node.length = reader.read_f32::<E>().map_err(StickfigureError::Io)?;
    node.default_thickness = reader.read_i32::<E>().map_err(StickfigureError::Io)?;
    node.thickness = reader.read_i32::<E>().map_err(StickfigureError::Io)?;

    if version >= 320 {
        node.segment_curve_radius_and_default_curve_radius =
            reader.read_i32::<E>().map_err(StickfigureError::Io)?;
    }
    if version >= 403 && build >= 20 {
        node.curve_circulization = reader.read_u8().map_err(StickfigureError::Io)? != 0;
    }
    if version >= 403 && build >= 21 {
        node.segment_curve_polyfill_precision =
            reader.read_i16::<E>().map_err(StickfigureError::Io)?;
    }
    if version >= 256 {
        node.half_arc = reader.read_u8().map_err(StickfigureError::Io)? != 0;
        node.right_triangle_direction = reader.read_i16::<E>().map_err(StickfigureError::Io)?;
    }
    if version >= 300 {
        node.triangle_upside_down = reader.read_u8().map_err(StickfigureError::Io)? != 0;
    }
    if version >= 403 && build >= 36 {
        // Older builds serialize these values as integers; newer builds use floats.
        // Precision loss is inherent in the legacy integer representation.
        #[allow(clippy::cast_precision_loss)]
        if build < 64 {
            node.trapezoid_thickness_start =
                reader.read_i32::<E>().map_err(StickfigureError::Io)? as f32;
            node.trapezoid_thickness_end =
                reader.read_i32::<E>().map_err(StickfigureError::Io)? as f32;
        } else {
            node.trapezoid_thickness_start =
                reader.read_f32::<E>().map_err(StickfigureError::Io)?;
            node.trapezoid_thickness_end = reader.read_f32::<E>().map_err(StickfigureError::Io)?;
        }
    }
    if version >= 403 && build == 36 {
        reader.read_i32::<E>().map_err(StickfigureError::Io)?;
        reader.read_i32::<E>().map_err(StickfigureError::Io)?;
        node.use_trapezoid_thickness_start = reader.read_u8().map_err(StickfigureError::Io)? != 0;
        node.use_trapezoid_thickness_end = reader.read_u8().map_err(StickfigureError::Io)? != 0;
    }
    if version >= 256 && build != 36 {
        node.trapezoid_top_thickness_ratio =
            reader.read_f32::<E>().map_err(StickfigureError::Io)?;
    }
    if version >= 403 && build >= 36 {
        node.trapezoid_is_rounded_start = reader.read_u8().map_err(StickfigureError::Io)? != 0;
        node.trapezoid_is_rounded_end = reader.read_u8().map_err(StickfigureError::Io)? != 0;
    }
    if version >= 256 {
        node.num_polygon_vertices = reader.read_i16::<E>().map_err(StickfigureError::Io)?;
    }
    if version >= 248 {
        node.default_local_angle = reader.read_f32::<E>().map_err(StickfigureError::Io)?;
    }
    node.local_angle = reader.read_f32::<E>().map_err(StickfigureError::Io)?;
    if version >= 248 {
        node.default_angle = reader.read_f32::<E>().map_err(StickfigureError::Io)?;
    }

    Ok(())
}

fn read_node_colors(
    reader: &mut impl Read,
    node: &mut Node,
    version: i32,
) -> Result<(), StickfigureError> {
    node.color = read_color(reader)?;
    if version >= 176 {
        node.gradient_color = read_color(reader)?;
    }
    if version >= 256 {
        node.circle_outline_color = read_color(reader)?;
    }
    Ok(())
}

fn read_node_angle_lock<E: ByteOrder>(
    reader: &mut impl Read,
    node: &mut Node,
    version: i32,
    build: i32,
) -> Result<(), StickfigureError> {
    if version >= 403 && build >= 39 {
        node.is_angle_locked = reader.read_u8().map_err(StickfigureError::Io)? != 0;
        if build <= 50 {
            node.is_angle_locked = false;
        }
    }
    if version >= 403 && (39..=50).contains(&build) {
        reader.read_f32::<E>().map_err(StickfigureError::Io)?;
    }
    if version >= 403 && build >= 51 {
        node.angle_lock_is_main_node = reader.read_u8().map_err(StickfigureError::Io)? != 0;
        if build < 56 {
            node.angle_lock_is_main_node = !node.angle_lock_is_main_node;
        }
    }
    if version >= 403 && (51..=56).contains(&build) {
        node.angle_lock_offset_minuend = reader.read_f32::<E>().map_err(StickfigureError::Io)?;
        node.angle_lock_offset_subtrahend = reader.read_f32::<E>().map_err(StickfigureError::Io)?;
    }
    if version >= 403 && build >= 57 {
        node.angle_lock_offset = reader.read_f32::<E>().map_err(StickfigureError::Io)?;
    }
    if version >= 403 && build >= 63 {
        node.angle_lock_relative_start = reader.read_f32::<E>().map_err(StickfigureError::Io)?;
    }
    if version >= 403 && build >= 67 {
        node.angle_lock_stickfigure_start = reader.read_f32::<E>().map_err(StickfigureError::Io)?;
    }
    if version >= 403 && build >= 63 {
        node.angle_lock_relative_multiplier = reader.read_i8().map_err(StickfigureError::Io)?;
    }
    if version >= 403 && build >= 39 {
        node.is_drag_locked = if build <= 40 {
            reader.read_i16::<E>().map_err(StickfigureError::Io)? != 0
        } else {
            reader.read_u8().map_err(StickfigureError::Io)? != 0
        };
    }
    if version >= 403 && (41..=45).contains(&build) {
        reader.read_i16::<E>().map_err(StickfigureError::Io)?;
    }
    if version >= 403 && build >= 46 {
        node.drag_lock_angle = reader.read_f32::<E>().map_err(StickfigureError::Io)?;
    }
    if version >= 403 && build >= 41 {
        node.smart_stretch_multiplier = reader.read_f32::<E>().map_err(StickfigureError::Io)?;
    }
    if version >= 403 && (41..=45).contains(&build) {
        reader.read_u8().map_err(StickfigureError::Io)?;
    }

    Ok(())
}

fn read_node_children_metadata<E: ByteOrder>(
    reader: &mut impl Read,
    version: i32,
    build: i32,
) -> Result<(i32, Vec<bool>), StickfigureError> {
    let number_of_child_nodes = reader.read_i32::<E>().map_err(StickfigureError::Io)?;
    let mut connector_booleans = Vec::new();

    if version >= 403 && build > 38 {
        for _ in 0..number_of_child_nodes {
            connector_booleans.push(reader.read_i32::<E>().map_err(StickfigureError::Io)? != 0);
        }
    }

    Ok((number_of_child_nodes, connector_booleans))
}

fn initialize_node_derived_fields(node: &mut Node) {
    match node.right_triangle_direction {
        -1 => {
            node.triangle_type = TriangleType::RightTriangle;
            node.triangle_flipped = true;
        }
        0 => {
            node.triangle_type = TriangleType::Isosceles;
            node.triangle_flipped = false;
        }
        1 => {
            node.triangle_type = TriangleType::RightTriangle;
            node.triangle_flipped = false;
        }
        _ => {
            node.triangle_type = TriangleType::RightTriangle;
            node.triangle_flipped = node.right_triangle_direction < 0;
        }
    }

    node.angle_lock_mode = if !node.is_angle_locked {
        AngleLockMode::None
    } else if node.angle_lock_is_main_node {
        AngleLockMode::Absolute
    } else {
        AngleLockMode::Relative
    };
}

fn read_polyfill_header<E: ByteOrder>(
    reader: &mut impl Read,
) -> Result<Vec<Rc<RefCell<Polyfill>>>, StickfigureError> {
    let number_of_polyfills = reader.read_i32::<E>().map_err(StickfigureError::Io)?;

    // Allow justified because wrapping and truncation would only matter at absurdly large polyfill counts
    // Allow cast sign loss justified because number of polyfills should never be negative
    #[allow(
        clippy::cast_possible_wrap,
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss
    )]
    let mut polyfills = Vec::with_capacity(number_of_polyfills as usize);
    for _ in 0..number_of_polyfills {
        let polyfill = Rc::new(RefCell::new(read_polyfill::<E>(reader)?));
        polyfills.push(polyfill);
    }

    Ok(polyfills)
}

fn read_polyfill<E: ByteOrder>(reader: &mut impl Read) -> Result<Polyfill, StickfigureError> {
    let mut polyfill = Polyfill::default();

    polyfill.anchor_node_draw_index.0 = reader.read_i32::<E>().map_err(StickfigureError::Io)?;

    polyfill.color = read_color(reader)?;
    polyfill.use_polyfill_color = reader.read_u8().map_err(StickfigureError::Io)? != 0;
    let number_of_attached_nodes = reader.read_i32::<E>().map_err(StickfigureError::Io)?;

    let attached_node_draw_indices: Vec<DrawOrderIndex> = (0..number_of_attached_nodes)
        .map(|_| reader.read_i32::<E>().map(DrawOrderIndex))
        .collect::<Result<_, _>>()
        .map_err(StickfigureError::Io)?;

    polyfill.attached_node_draw_indices = attached_node_draw_indices;

    Ok(polyfill)
}
