#[derive(Default)]
pub(super) struct CategoryDepth {
    pub(super) layer: f32,
    pub(super) peak: f32,
    pub(super) ramming: bool,
    pub(super) reverse: bool,
}

pub(super) struct CategoryGap {
    pub(super) wipe: f32,
    pub(super) nozzle: f32,
}

impl CategoryDepth {
    pub(super) fn gap(
        &self,
        perimeter: f32,
        nozzle_perimeter: f32,
        extra: f32,
        height_spacing: f32,
    ) -> CategoryGap {
        let wipe = perimeter + extra;
        if !self.ramming {
            return CategoryGap { wipe, nozzle: wipe };
        }
        // WipeTower.hpp initializes m_tpu_fixed_spacing to two.
        let extra = if self.reverse {
            extra.max(perimeter)
        } else {
            extra
        };
        let nozzle = nozzle_perimeter + extra;
        let aligned = nozzle * height_spacing;
        if aligned < 2.0 * nozzle_perimeter {
            CategoryGap { wipe, nozzle }
        } else {
            CategoryGap {
                wipe: nozzle,
                nozzle,
            }
        }
    }
}
