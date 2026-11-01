//! The instance edits of the plater session: moves, rotations, the scale and size fields and the
//! object menu's mirror (`Selection` and `GizmoObjectManipulation` through
//! `ares_core::PlaterSession`).

use ares_core::ScaleEdit;
use serde::Deserialize;
use wasm_bindgen::prelude::*;

use super::{PlaterSession, js_error};

/// A scale or size field edit as the web passes it (`ScaleEdit`).
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ScaleEditJson {
    axis: usize,
    value: f64,
    uniform: bool,
}

impl ScaleEditJson {
    fn parse(json: &str) -> Result<ScaleEdit, JsValue> {
        let edit: Self = serde_json::from_str(json).map_err(js_error)?;
        Ok(ScaleEdit {
            axis: edit.axis,
            value: edit.value,
            uniform: edit.uniform,
        })
    }
}

fn vector(values: &[f64]) -> Result<[f64; 3], JsValue> {
    values
        .try_into()
        .map_err(|_| js_error(format!("expected [x, y, z], got {} values", values.len())))
}

#[wasm_bindgen]
impl PlaterSession {
    /// A world move by `displacement` `[x, y, z]` (`Selection::translate`, `GLCanvas3D::do_move`).
    pub fn translate(
        &mut self,
        object: usize,
        instance: usize,
        displacement: Vec<f64>,
    ) -> Result<(), JsValue> {
        self.session
            .translate_instance(object, instance, vector(&displacement)?)
            .map_err(js_error)
    }

    /// A world position field edit (`GizmoObjectManipulation::change_position_value`).
    #[wasm_bindgen(js_name = setPosition)]
    pub fn set_position(
        &mut self,
        object: usize,
        instance: usize,
        axis: usize,
        value: f64,
    ) -> Result<(), JsValue> {
        self.session
            .set_instance_position(object, instance, axis, value)
            .map_err(js_error)
    }

    /// A world rotation by `radians` `[x, y, z]` (`Selection::rotate`, `GLCanvas3D::do_rotate`).
    pub fn rotate(
        &mut self,
        object: usize,
        instance: usize,
        radians: Vec<f64>,
    ) -> Result<(), JsValue> {
        self.session
            .rotate_instance(object, instance, vector(&radians)?)
            .map_err(js_error)
    }

    /// The absolute rotation fields in degrees (`GizmoObjectManipulation::update_settings_value`).
    #[wasm_bindgen(js_name = absoluteRotation)]
    pub fn absolute_rotation(&self, object: usize, instance: usize) -> Result<Vec<f64>, JsValue> {
        self.session
            .absolute_rotation(object, instance)
            .map(Vec::from)
            .map_err(js_error)
    }

    /// An absolute rotation field edit (`change_absolute_rotation_value`).
    #[wasm_bindgen(js_name = setAbsoluteRotation)]
    pub fn set_absolute_rotation(
        &mut self,
        object: usize,
        instance: usize,
        axis: usize,
        degrees: f64,
    ) -> Result<(), JsValue> {
        self.session
            .set_absolute_rotation(object, instance, axis, degrees)
            .map_err(js_error)
    }

    /// The scale (percent) and size (mm) fields, `[scale x, y, z, size x, y, z]`
    /// (`GizmoObjectManipulation::update_settings_value`).
    #[wasm_bindgen(js_name = scaleAndSize)]
    pub fn scale_and_size(&self, object: usize, instance: usize) -> Result<Vec<f64>, JsValue> {
        let (scale, size) = self
            .session
            .scale_and_size(object, instance)
            .map_err(js_error)?;
        Ok([scale, size].concat())
    }

    /// A "Scale ratios" field edit, `edit_json` `{axis, value, uniform}` (`change_scale_value`).
    #[wasm_bindgen(js_name = setScale)]
    pub fn set_scale(
        &mut self,
        object: usize,
        instance: usize,
        edit_json: &str,
    ) -> Result<(), JsValue> {
        let edit = ScaleEditJson::parse(edit_json)?;
        self.session
            .set_scale(object, instance, edit)
            .map_err(js_error)
    }

    /// A "Size" field edit, `edit_json` `{axis, value, uniform}` (`change_size_value`).
    #[wasm_bindgen(js_name = setSize)]
    pub fn set_size(
        &mut self,
        object: usize,
        instance: usize,
        edit_json: &str,
    ) -> Result<(), JsValue> {
        let edit = ScaleEditJson::parse(edit_json)?;
        self.session
            .set_size(object, instance, edit)
            .map_err(js_error)
    }

    /// The object menu's "Mirror" along `axis` (0 X, 1 Y, 2 Z) (`Plater::mirror`).
    pub fn mirror(&mut self, object: usize, instance: usize, axis: usize) -> Result<(), JsValue> {
        self.session
            .mirror(object, instance, axis)
            .map_err(js_error)
    }
}
