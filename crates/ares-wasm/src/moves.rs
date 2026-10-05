use ares_core::{ProcessedGCode, gcode_processor::MoveVertex};
use ares_vgcode::{GCodeInputData, convert::from_processor_result};
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
        Self {
            gcode: output.gcode,
            moves: MoveBuffers::pack(&output.result.moves),
            vertices: VertexBuffers::pack(data),
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
#[wasm_bindgen]
#[derive(Default)]
pub struct VertexBuffers {
    count: usize,
    spiral_vase_mode: bool,
    values: Vec<f32>,
    ids: Vec<u32>,
    tags: Vec<u8>,
    tool_colors: Vec<u8>,
    color_print_colors: Vec<u8>,
}

impl VertexBuffers {
    fn pack(data: GCodeInputData) -> Self {
        let mut out = Self {
            count: data.vertices.len(),
            spiral_vase_mode: data.spiral_vase_mode,
            values: Vec::with_capacity(16 * data.vertices.len()),
            ids: Vec::with_capacity(2 * data.vertices.len()),
            tags: Vec::with_capacity(4 * data.vertices.len()),
            tool_colors: data.tools_colors.into_iter().flatten().collect(),
            color_print_colors: data.color_print_colors.into_iter().flatten().collect(),
        };
        for v in data.vertices {
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
        }
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
}
