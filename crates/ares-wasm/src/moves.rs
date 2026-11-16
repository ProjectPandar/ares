use crate::MoveBuffers;
use ares_vgcode::{
    CustomGCodeOverviewRow, FilamentLegend, GCodeExtrusionRole, MovesSlider, OptionType,
    SequentialView, ViewerImpl, move_type_to_option,
};
use wasm_bindgen::prelude::*;

/// Owned result of one core processed slice. Take methods empty their buffers.
#[wasm_bindgen]
pub struct SliceOutput {
    gcode: Vec<u8>,
    moves: MoveBuffers,
    pub(crate) vertices: VertexBuffers,
    print_statistics: Vec<u8>,
    estimation: String,
    extruder_colors: Vec<String>,
    filament_diameters: Vec<f32>,
    filament_densities: Vec<f32>,
    pub(crate) custom_gcode_overview: Vec<CustomGCodeOverviewRow>,
}

mod processed;

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

    /// `GCodeProcessorResult::print_statistics` as UTF-8 JSON bytes
    /// (`PrintEstimatedStatistics`, `GCodeProcessor.hpp:46-78`).
    #[wasm_bindgen(js_name = takePrintStatistics)]
    pub fn take_print_statistics(&mut self) -> Vec<u8> {
        std::mem::take(&mut self.print_statistics)
    }

    /// Legend estimation footer inputs as JSON (`GCodeViewer.cpp:4547-4644`).
    #[wasm_bindgen(getter)]
    pub fn estimation(&self) -> String {
        self.estimation.clone()
    }

    /// `GCodeProcessorResult::extruder_colors`
    /// (`GCodeProcessor.cpp:2321-2343`).
    #[wasm_bindgen(js_name = takeExtruderColors)]
    pub fn take_extruder_colors(&mut self) -> Vec<String> {
        std::mem::take(&mut self.extruder_colors)
    }

    /// `GCodeProcessorResult::filament_diameters` (`GCodeProcessor.hpp:252`).
    #[wasm_bindgen(js_name = takeFilamentDiameters)]
    pub fn take_filament_diameters(&mut self) -> Vec<f32> {
        std::mem::take(&mut self.filament_diameters)
    }

    /// `GCodeProcessorResult::filament_densities` (`GCodeProcessor.hpp:254`).
    #[wasm_bindgen(js_name = takeFilamentDensities)]
    pub fn take_filament_densities(&mut self) -> Vec<f32> {
        std::mem::take(&mut self.filament_densities)
    }
}

/// Converted libvgcode vertices and palettes; no G-code parsing in the adapter.
/// Keeps the vertices loaded in libvgcode `ViewerImpl` for `viewRange` and
/// the moves slider.
#[wasm_bindgen]
#[derive(Default)]
pub struct VertexBuffers {
    count: usize,
    spiral_vase_mode: bool,
    pub(crate) viewer: ViewerImpl,
    moves_slider: MovesSlider,
    pub(crate) sequential_view: SequentialView,
    values: Vec<f32>,
    ids: Vec<u32>,
    tags: Vec<u8>,
    options: Vec<u8>,
    move_type_counts: Vec<u32>,
    tool_colors: Vec<u8>,
    color_print_colors: Vec<u8>,
    layer_zs: Vec<f32>,
    pub(crate) filament_legend: FilamentLegend,
}

impl VertexBuffers {
    fn pack(
        mut viewer: ViewerImpl,
        (spiral_vase_mode, tool_colors, color_print_colors): (bool, Vec<u8>, Vec<u8>),
    ) -> Self {
        let count = viewer.vertices().len();
        // GCodeViewer::load_as_gcode (GCodeViewer.cpp:1128-1131) with the
        // `seq_top_layer_only` default "1" (AppConfig.cpp:202-203).
        viewer.toggle_top_layer_only_view_range();
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
            moves_slider: MovesSlider::new(&viewer),
            sequential_view: SequentialView::default(),
            filament_legend: FilamentLegend::default(),
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

    fn moves_thumbs(&self) -> [u32; 2] {
        [self.moves_slider.lower, self.moves_slider.higher].map(|stop| stop as u32)
    }

    fn visible_range(&self) -> Vec<u32> {
        let [first, last] = self.viewer.get_view_visible_range();
        [first, last, self.viewer.get_current_vertex_id()]
            .map(|id| id as u32)
            .to_vec()
    }

    /// Shows the extrusion roles with bit `i` of `visible_roles` set
    /// (`EGCodeExtrusionRole` order) and the options with bit `i` of
    /// `visible_options` set (`EOptionType` order), as GCodeViewer toggles the legend.
    pub(crate) fn set_visibility(&mut self, visible_roles: u32, visible_options: u16) {
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
    /// libvgcode `ViewerImpl` view range after `set_visibility` and
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
        self.set_visibility(visible_roles, visible_options);
        self.viewer
            .set_layers_view_range(min_layer as usize, max_layer as usize);
        self.visible_range()
    }
    /// The visibility as `viewRange` sets it, then `MovesSlider::set_layers_z_range`
    /// (GCodeViewer `set_layers_z_range` / `update_moves_slider`). Returns
    /// `[lower, higher, ...gcodeIds]`: the thumbs and the G-code line id of each stop.
    #[wasm_bindgen(js_name = movesSlider)]
    pub fn moves_slider(
        &mut self,
        min_layer: u32,
        max_layer: u32,
        visible_roles: u32,
        visible_options: u16,
    ) -> Vec<u32> {
        self.set_visibility(visible_roles, visible_options);
        self.moves_slider
            .set_layers_z_range(&mut self.viewer, [min_layer as usize, max_layer as usize]);
        self.moves_thumbs()
            .into_iter()
            .chain(self.moves_slider.alternate_values.iter().copied())
            .collect()
    }
    /// `MovesSlider::set_higher_value` (`IMSlider::SetHigherValue`). Returns
    /// the thumbs `[lower, higher]`.
    #[wasm_bindgen(js_name = setMovesHigher)]
    pub fn set_moves_higher(&mut self, value: i32) -> Vec<u32> {
        self.moves_slider.set_higher_value(value as isize);
        self.moves_thumbs().to_vec()
    }
    /// libvgcode `ViewerImpl::set_view_visible_range` between the moves slider
    /// thumbs (`GLCanvas3D.cpp:8263-8266`). Returns `[first, last, current]`
    /// as `viewRange`; fails before `movesSlider` has stops.
    #[wasm_bindgen(js_name = setMovesRange)]
    pub fn set_moves_range(&mut self) -> Result<Vec<u32>, JsError> {
        let Some([min, max]) = self.moves_slider.vertex_range() else {
            return Err(JsError::new("moves slider without stops"));
        };
        self.viewer.set_view_visible_range(min, max);
        Ok(self.visible_range())
    }
    /// `SequentialView::update` after `setMovesRange` and `vertexColors`:
    /// `[vertex, windowVertex]` of the shown tool marker, empty while hidden.
    #[wasm_bindgen(js_name = toolMarker)]
    pub fn tool_marker(&mut self) -> Vec<u32> {
        self.sequential_view
            .update(&self.viewer)
            .map_or_else(Vec::new, |marker| {
                vec![marker.vertex as u32, marker.window_vertex as u32]
            })
    }
    /// Float32Array `[elapsedSeconds, vertexSeconds]` for the position window.
    #[wasm_bindgen(js_name = markerTime)]
    pub fn marker_time(&self, window_vertex: u32) -> Result<Vec<f32>, JsError> {
        let id = window_vertex as usize;
        if id >= self.viewer.vertices().len() {
            return Err(JsError::new("marker vertex outside loaded toolpaths"));
        }
        let time = self.viewer.get_marker_time(id);
        Ok(vec![time.elapsed_seconds, time.vertex_seconds])
    }

    /// Uint32Array: vertex ids of libvgcode's enabled segments (`i` to `i + 1`).
    #[wasm_bindgen(js_name = enabledSegments)]
    pub fn enabled_segments(&mut self) -> Vec<u32> {
        self.viewer.enabled_segments().to_vec()
    }
    /// Uint32Array: vertex ids of libvgcode's enabled option markers.
    #[wasm_bindgen(js_name = enabledOptions)]
    pub fn enabled_options(&mut self) -> Vec<u32> {
        self.viewer.enabled_options().to_vec()
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
