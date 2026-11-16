use super::VertexBuffers;
use ares_vgcode::move_type_to_option;
use std::ops::Range;
use wasm_bindgen::prelude::*;

impl VertexBuffers {
    pub(crate) fn checked_range(
        first: u32,
        count: u32,
        total: usize,
    ) -> Result<Range<usize>, JsError> {
        if count > 1_048_576 {
            return Err(JsError::new("transfer range exceeds 1048576 records"));
        }
        let first = first as usize;
        let end = first
            .checked_add(count as usize)
            .filter(|&end| end <= total)
            .ok_or_else(|| JsError::new("transfer range outside loaded data"))?;
        Ok(first..end)
    }

    pub(crate) fn vertex_range(&self, first: u32, count: u32) -> Result<Range<usize>, JsError> {
        Self::checked_range(first, count, self.count)
    }
}

#[wasm_bindgen]
impl VertexBuffers {
    /// Float32Array stride16: xyz,height,width,feedrate,actual_feedrate,mm3/mm,
    /// fan,temp,time_normal,time_stealth,layer_duration,PA,acceleration,jerk.
    #[wasm_bindgen(js_name = valuesRange)]
    pub fn values_range(&self, first: u32, count: u32) -> Result<Vec<f32>, JsError> {
        let range = self.vertex_range(first, count)?;
        let mut values = Vec::with_capacity(16 * range.len());
        for v in &self.viewer.vertices()[range] {
            values.extend_from_slice(&[
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
        }
        Ok(values)
    }

    /// Uint32Array stride2: global gcode_id,layer_id.
    #[wasm_bindgen(js_name = idsRange)]
    pub fn ids_range(&self, first: u32, count: u32) -> Result<Vec<u32>, JsError> {
        let range = self.vertex_range(first, count)?;
        Ok(self.viewer.vertices()[range]
            .iter()
            .flat_map(|v| [v.gcode_id, v.layer_id])
            .collect())
    }

    /// Uint8Array stride4: role,type,extruder,color.
    #[wasm_bindgen(js_name = tagsRange)]
    pub fn tags_range(&self, first: u32, count: u32) -> Result<Vec<u8>, JsError> {
        let range = self.vertex_range(first, count)?;
        Ok(self.viewer.vertices()[range]
            .iter()
            .flat_map(|v| [v.role as u8, v.move_type as u8, v.extruder_id, v.color_id])
            .collect())
    }

    /// Uint8Array: `EOptionType` index, 255 for none.
    #[wasm_bindgen(js_name = optionsRange)]
    pub fn options_range(&self, first: u32, count: u32) -> Result<Vec<u8>, JsError> {
        let range = self.vertex_range(first, count)?;
        Ok(self.viewer.vertices()[range]
            .iter()
            .map(|v| move_type_to_option(v.move_type).map_or(u8::MAX, |option| option as u8))
            .collect())
    }

    /// Refresh the retained enabled-entity caches once; return their counts.
    #[wasm_bindgen(js_name = prepareEnabledIndices)]
    pub fn prepare_enabled_indices(&mut self) -> Vec<u32> {
        vec![
            self.viewer.enabled_segments().len() as u32,
            self.viewer.enabled_options().len() as u32,
        ]
    }

    #[wasm_bindgen(js_name = enabledSegmentsRange)]
    pub fn enabled_segments_range(&mut self, first: u32, count: u32) -> Result<Vec<u32>, JsError> {
        let enabled = self.viewer.enabled_segments();
        let range = Self::checked_range(first, count, enabled.len())?;
        Ok(enabled[range].to_vec())
    }

    #[wasm_bindgen(js_name = enabledOptionsRange)]
    pub fn enabled_options_range(&mut self, first: u32, count: u32) -> Result<Vec<u32>, JsError> {
        let enabled = self.viewer.enabled_options();
        let range = Self::checked_range(first, count, enabled.len())?;
        Ok(enabled[range].to_vec())
    }

    #[wasm_bindgen(js_name = sliderGcodeIdsRange)]
    pub fn slider_gcode_ids_range(&self, first: u32, count: u32) -> Result<Vec<u32>, JsError> {
        let values = &self.moves_slider.alternate_values;
        let range = Self::checked_range(first, count, values.len())?;
        Ok(values[range].to_vec())
    }
}
