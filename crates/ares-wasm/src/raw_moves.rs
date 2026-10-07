use ares_core::gcode_processor::MoveVertex;
use wasm_bindgen::prelude::*;

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
    pub(crate) fn pack(moves: &[MoveVertex]) -> Self {
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
