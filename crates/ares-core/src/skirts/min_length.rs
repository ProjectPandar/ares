use crate::SliceError;

const MAX_GENERATED_SKIRT_LOOPS: u32 = 10_000;

#[derive(Clone, Copy)]
pub(super) struct Bounds {
    pub(super) min_x: f64,
    pub(super) min_y: f64,
    pub(super) max_x: f64,
    pub(super) max_y: f64,
}

pub(super) struct LoopCountInput {
    pub(super) configured_loops: u32,
    pub(super) min_skirt_length_mm: f64,
    pub(super) distance_mm: f64,
    pub(super) effective_line_width: f64,
    pub(super) bounds: Bounds,
    pub(super) skirt_extrusion_per_mm: f64,
    pub(super) apply_min_length: bool,
}

pub(super) fn loop_count(input: LoopCountInput) -> Result<u32, SliceError> {
    if input.configured_loops == 0 || !input.apply_min_length || input.min_skirt_length_mm <= 0.0 {
        return Ok(input.configured_loops);
    }

    let mut loops = input.configured_loops;
    let mut extruded = total_skirt_extrusion_mm(&input, loops);

    while extruded < input.min_skirt_length_mm {
        if loops >= MAX_GENERATED_SKIRT_LOOPS {
            return Err(SliceError::InvalidInput(
                "min_skirt_length would require more than 10000 skirt loops".to_owned(),
            ));
        }
        extruded += skirt_loop_extrusion_mm(&input, loops);
        loops += 1;
    }

    Ok(loops)
}

fn total_skirt_extrusion_mm(input: &LoopCountInput, loops: u32) -> f64 {
    (0..loops)
        .map(|loop_index| skirt_loop_extrusion_mm(input, loop_index))
        .sum()
}

fn skirt_loop_extrusion_mm(input: &LoopCountInput, loop_index: u32) -> f64 {
    let expand = input.distance_mm + f64::from(loop_index) * input.effective_line_width;
    let width = input.bounds.max_x - input.bounds.min_x + 2.0 * expand;
    let height = input.bounds.max_y - input.bounds.min_y + 2.0 * expand;
    2.0 * (width + height) * input.skirt_extrusion_per_mm
}
