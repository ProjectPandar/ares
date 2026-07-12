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

#[cfg(test)]
mod tests {
    use super::*;

    fn input(min_skirt_length_mm: f64) -> LoopCountInput {
        LoopCountInput {
            configured_loops: 1,
            min_skirt_length_mm,
            distance_mm: 2.0,
            effective_line_width: 0.4,
            bounds: Bounds {
                min_x: -0.5,
                min_y: -0.5,
                max_x: 0.5,
                max_y: 0.5,
            },
            skirt_extrusion_per_mm: 0.05,
            apply_min_length: true,
        }
    }

    #[test]
    fn keeps_configured_loops_when_min_length_is_disabled() {
        assert_eq!(loop_count(input(0.0)).unwrap(), 1);
    }

    #[test]
    fn zero_configured_loops_disable_min_length_extension() {
        let input = LoopCountInput {
            configured_loops: 0,
            ..input(1.0)
        };

        assert_eq!(loop_count(input).unwrap(), 0);
    }

    #[test]
    fn extends_until_min_length_is_reached() {
        assert_eq!(loop_count(input(3.0)).unwrap(), 3);
    }

    #[test]
    fn rejects_requests_over_the_loop_bound() {
        let err = loop_count(input(1.0e12)).unwrap_err();

        assert!(
            matches!(err, SliceError::InvalidInput(message) if message.contains("min_skirt_length would require more than 10000 skirt loops"))
        );
    }
}
