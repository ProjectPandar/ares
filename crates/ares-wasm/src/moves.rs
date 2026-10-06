use ares_core::{ProcessedGCode, gcode_processor::MoveVertex};
use ares_vgcode::{
    GCodeExtrusionRole, GCodeInputData, MoveType, OptionType, ViewerImpl,
    convert::from_processor_result, move_type_to_option,
};
use wasm_bindgen::prelude::*;

/// Owned result of one core processed slice. Take methods empty their buffers.
#[wasm_bindgen]
pub struct SliceOutput {
    gcode: Vec<u8>,
    moves: MoveBuffers,
    vertices: VertexBuffers,
}

impl SliceOutput {
    pub fn from_processed(
        output: ProcessedGCode,
        tool_colors: &[String],
        color_print_colors: &[String],
    ) -> Self {
        let data = from_processor_result(&output.result, tool_colors, color_print_colors);
        let mut vertices = VertexBuffers::pack(data);
        // GCodeViewer::load_as_preview `m_move_type_counts` (GCodeViewer.cpp:1318-1324).
        vertices.move_type_counts = vec![0; MoveType::COUNT];
        for m in output.result.moves.iter().filter(|m| !m.internal_only) {
            vertices.move_type_counts[m.move_type as usize] += 1;
        }
        Self {
            gcode: output.gcode,
            moves: MoveBuffers::pack(&output.result.moves),
            vertices,
        }
    }
}

#[wasm_bindgen]
impl SliceOutput {
    #[wasm_bindgen(js_name = takeGcode)]
    pub fn take_gcode(&mut self) -> Vec<u8> {
        std::mem::take(&mut self.gcode)
    }

    #[wasm_bindgen(js_name = takeMoves)]
    pub fn take_moves(&mut self) -> MoveBuffers {
        std::mem::take(&mut self.moves)
    }

    #[wasm_bindgen(js_name = takeVertices)]
    pub fn take_vertices(&mut self) -> VertexBuffers {
        std::mem::take(&mut self.vertices)
    }
}

/// Raw moves in record order, including the dummy and internal split vertices.
#[wasm_bindgen]
#[derive(Default)]
pub struct MoveBuffers {
    count: usize,
    values: Vec<f32>,
    ids: Vec<u32>,
    tags: Vec<u8>,
}

impl MoveBuffers {
    fn pack(moves: &[MoveVertex]) -> Self {
        let mut out = Self {
            count: moves.len(),
            values: Vec::with_capacity(12 * moves.len()),
            ids: Vec::with_capacity(2 * moves.len()),
            tags: Vec::with_capacity(5 * moves.len()),
        };
        for m in moves {
            out.values.extend_from_slice(&[
                m.position[0],
                m.position[1],
                m.position[2],
                m.delta_extruder,
                m.feedrate,
                m.width,
                m.height,
                m.mm3_per_mm,
                m.travel_dist,
                m.fan_speed,
                m.temperature,
                m.time,
            ]);
            out.ids.extend_from_slice(&[m.gcode_id, m.layer_id]);
            out.tags.extend_from_slice(&[
                m.move_type as u8,
                m.extrusion_role as u8,
                m.extruder_id,
                m.cp_color_id,
                u8::from(m.internal_only),
            ]);
        }
        out
    }
}

#[wasm_bindgen]
impl MoveBuffers {
    #[wasm_bindgen(getter)]
    pub fn count(&self) -> usize {
        self.count
    }

    /// Float32Array stride12: x,y,z,deltaE,feedrate,width,height,mm3/mm,travel,fan,temp,time.
    #[wasm_bindgen(js_name = takeValues)]
    pub fn take_values(&mut self) -> Vec<f32> {
        std::mem::take(&mut self.values)
    }
    /// Uint32Array stride2: gcode_id,layer_id.
    #[wasm_bindgen(js_name = takeIds)]
    pub fn take_ids(&mut self) -> Vec<u32> {
        std::mem::take(&mut self.ids)
    }
    /// Uint8Array stride5: type,role,extruder,color,internal_only.
    #[wasm_bindgen(js_name = takeTags)]
    pub fn take_tags(&mut self) -> Vec<u8> {
        std::mem::take(&mut self.tags)
    }
}

/// Converted libvgcode vertices and palettes; no G-code parsing in the adapter.
/// Keeps the vertices loaded in libvgcode `ViewerImpl` for `viewRange`.
#[wasm_bindgen]
#[derive(Default)]
pub struct VertexBuffers {
    count: usize,
    spiral_vase_mode: bool,
    viewer: ViewerImpl,
    values: Vec<f32>,
    ids: Vec<u32>,
    tags: Vec<u8>,
    options: Vec<u8>,
    move_type_counts: Vec<u32>,
    tool_colors: Vec<u8>,
    color_print_colors: Vec<u8>,
    layer_zs: Vec<f32>,
}

impl VertexBuffers {
    fn pack(mut data: GCodeInputData) -> Self {
        let count = data.vertices.len();
        let tool_colors = std::mem::take(&mut data.tools_colors)
            .into_iter()
            .flatten()
            .collect();
        let color_print_colors = std::mem::take(&mut data.color_print_colors)
            .into_iter()
            .flatten()
            .collect();
        let spiral_vase_mode = data.spiral_vase_mode;
        let mut viewer = ViewerImpl::default();
        viewer.load(data);
        let mut out = Self {
            count,
            spiral_vase_mode,
            values: Vec::with_capacity(16 * count),
            ids: Vec::with_capacity(2 * count),
            tags: Vec::with_capacity(4 * count),
            options: Vec::with_capacity(count),
            move_type_counts: Vec::new(),
            tool_colors,
            color_print_colors,
            layer_zs: viewer.layers().get_zs(),
            viewer: ViewerImpl::default(),
        };
        for v in viewer.vertices() {
            out.values.extend_from_slice(&[
                v.position[0],
                v.position[1],
                v.position[2],
                v.height,
                v.width,
                v.feedrate,
                v.actual_feedrate,
                v.mm3_per_mm,
                v.fan_speed,
                v.temperature,
                v.times[0],
                v.times[1],
                v.layer_duration,
                v.pressure_advance,
                v.acceleration,
                v.jerk,
            ]);
            out.ids.extend_from_slice(&[v.gcode_id, v.layer_id]);
            out.tags.extend_from_slice(&[
                v.role as u8,
                v.move_type as u8,
                v.extruder_id,
                v.color_id,
            ]);
            out.options
                .push(move_type_to_option(v.move_type).map_or(u8::MAX, |option| option as u8));
        }
        out.viewer = viewer;
        out
    }
}

#[wasm_bindgen]
impl VertexBuffers {
    #[wasm_bindgen(getter)]
    pub fn count(&self) -> usize {
        self.count
    }
    #[wasm_bindgen(getter, js_name = spiralVaseMode)]
    pub fn spiral_vase_mode(&self) -> bool {
        self.spiral_vase_mode
    }
    /// Float32Array stride16: xyz,height,width,feedrate,actual_feedrate,mm3/mm,
    /// fan,temp,time_normal,time_stealth,layer_duration,PA,acceleration,jerk.
    #[wasm_bindgen(js_name = takeValues)]
    pub fn take_values(&mut self) -> Vec<f32> {
        std::mem::take(&mut self.values)
    }
    /// Uint32Array stride2: gcode_id,layer_id.
    #[wasm_bindgen(js_name = takeIds)]
    pub fn take_ids(&mut self) -> Vec<u32> {
        std::mem::take(&mut self.ids)
    }
    /// Uint8Array stride4: role,type,extruder,color.
    #[wasm_bindgen(js_name = takeTags)]
    pub fn take_tags(&mut self) -> Vec<u8> {
        std::mem::take(&mut self.tags)
    }
    /// Uint8Array: libvgcode `move_type_to_option` of each vertex as an
    /// `EOptionType` index (`optionTypes` order), 255 for none.
    #[wasm_bindgen(js_name = takeOptions)]
    pub fn take_options(&mut self) -> Vec<u8> {
        std::mem::take(&mut self.options)
    }
    /// Uint32Array: processed moves by `EMoveType`, internal split moves excluded
    /// (GCodeViewer `m_move_type_counts`).
    #[wasm_bindgen(js_name = takeMoveTypeCounts)]
    pub fn take_move_type_counts(&mut self) -> Vec<u32> {
        std::mem::take(&mut self.move_type_counts)
    }
    /// Bit `i` set when `EOptionType` `i` is visible in the loaded
    /// `ViewerImpl` (`Settings::options_visibility`).
    #[wasm_bindgen(getter, js_name = visibleOptions)]
    pub fn visible_options(&self) -> u16 {
        OptionType::ALL
            .into_iter()
            .filter(|&option| self.viewer.is_option_visible(option))
            .fold(0, |mask, option| mask | 1 << option.index())
    }
    /// Uint8Array RGB triples.
    #[wasm_bindgen(js_name = takeToolColors)]
    pub fn take_tool_colors(&mut self) -> Vec<u8> {
        std::mem::take(&mut self.tool_colors)
    }
    /// Uint8Array RGB triples.
    #[wasm_bindgen(js_name = takeColorPrintColors)]
    pub fn take_color_print_colors(&mut self) -> Vec<u8> {
        std::mem::take(&mut self.color_print_colors)
    }
    /// libvgcode `ViewerImpl` view range after showing extrusion roles with
    /// bit `i` of `visible_roles` set (`EGCodeExtrusionRole` order) and options
    /// with bit `i` of `visible_options` set (`EOptionType` order), then
    /// `set_layers_view_range(min_layer, max_layer)`, as GCodeViewer toggles the
    /// legend and moves the layer slider. Returns the visible range and the
    /// current vertex: `[first, last, current]`.
    #[wasm_bindgen(js_name = viewRange)]
    pub fn view_range(
        &mut self,
        min_layer: u32,
        max_layer: u32,
        visible_roles: u32,
        visible_options: u16,
    ) -> Vec<u32> {
        let viewer = &mut self.viewer;
        for role in GCodeExtrusionRole::ALL {
            if viewer.is_extrusion_role_visible(role) != (visible_roles >> role.index() & 1 == 1) {
                viewer.toggle_extrusion_role_visibility(role);
            }
        }
        for option in OptionType::ALL {
            if viewer.is_option_visible(option) != (visible_options >> option.index() & 1 == 1) {
                viewer.toggle_option_visibility(option);
            }
        }
        viewer.set_layers_view_range(min_layer as usize, max_layer as usize);
        let [first, last] = viewer.get_view_visible_range();
        [first, last, viewer.get_current_vertex_id()]
            .map(|id| id as u32)
            .to_vec()
    }
    /// Float32Array: libvgcode `Layers::get_zs`, one z per layer.
    #[wasm_bindgen(js_name = takeLayerZs)]
    pub fn take_layer_zs(&mut self) -> Vec<f32> {
        std::mem::take(&mut self.layer_zs)
    }
}

/// libvgcode `EOptionType` names in index order: the bits of `viewRange`
/// `visible_options` and the values of `VertexBuffers::takeOptions`.
#[wasm_bindgen(js_name = optionTypes)]
pub fn option_types() -> Vec<String> {
    OptionType::ALL
        .iter()
        .map(|option| format!("{option:?}"))
        .collect()
}
