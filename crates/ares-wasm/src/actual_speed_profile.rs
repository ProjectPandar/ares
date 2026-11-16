use crate::VertexBuffers;
use serde::Serialize;
use wasm_bindgen::prelude::*;

#[derive(Serialize)]
struct LevelView {
    value: f64,
    color: [f64; 4],
}

#[derive(Serialize)]
struct ItemView {
    pos: f64,
    speed: f64,
    internal: bool,
}

/// `SequentialView::ActualSpeedImguiWidget` data (`GCodeViewer.hpp:47-59`).
/// The f32 values are widened exactly to f64, as `colorRange` does.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ActualSpeedProfileView {
    y_range: [f64; 2],
    levels: Vec<LevelView>,
    data: Vec<ItemView>,
}

#[wasm_bindgen]
impl VertexBuffers {
    /// JSON `{yRange, levels: [{value, color}], data: [{pos, speed, internal}]}`
    /// of `SequentialView::update_actual_speed` after `setMovesRange` and
    /// `prepareVertexColors` (`GCodeViewer.cpp:1681-1753`). Without an update the
    /// previous profile is returned.
    #[wasm_bindgen(js_name = actualSpeedProfile)]
    pub fn actual_speed_profile(&mut self) -> String {
        self.sequential_view.update_actual_speed(&self.viewer);
        let profile = self.sequential_view.actual_speed();
        let view = ActualSpeedProfileView {
            y_range: profile.y_range.map(f64::from),
            levels: profile
                .levels
                .iter()
                .map(|&(value, color)| LevelView {
                    value: value.into(),
                    color: color.map(f64::from),
                })
                .collect(),
            data: profile
                .data
                .iter()
                .map(|item| ItemView {
                    pos: item.pos.into(),
                    speed: item.speed.into(),
                    internal: item.internal,
                })
                .collect(),
        };
        serde_json::to_string(&view).expect("actual speed profile serializes")
    }
}
