//! `AdaptivePAInterpolator`: flow interpolation followed by acceleration.

use super::pchip::Pchip;

pub(super) struct Model(Vec<(f64, Pchip)>);

impl Model {
    pub(super) fn parse(csv: &str) -> Option<Self> {
        let mut rows = Vec::new();
        for line in csv.split_terminator('\n') {
            let mut row = [0.0; 3];
            for (slot, value) in row.iter_mut().zip(line.split_terminator(',')) {
                *slot = super::number::parse(value)?;
            }
            rows.push((row[2], row[1], row[0]));
        }
        rows.sort_by(|a, b| a.0.total_cmp(&b.0));
        let mut models = Vec::new();
        for group in rows.chunk_by(|a, b| a.0 == b.0) {
            if group.len() > 1 {
                models.push((
                    group[0].0,
                    Pchip::new(group.iter().map(|p| (p.1, p.2)).collect()),
                ));
            }
        }
        Some(Self(models))
    }

    pub(super) fn predict(&self, flow: f64, acceleration: f64) -> f64 {
        let values: Vec<_> = self
            .0
            .iter()
            .filter_map(|(accel, curve)| {
                let pa = curve.interpolate(flow);
                (pa != -1.0).then_some((*accel, pa))
            })
            .collect();
        let predicted = match values.as_slice() {
            [] => return -1.0,
            [(_, value)] => *value,
            _ => Pchip::new(values).interpolate(acceleration),
        };
        (predicted * 1000.0).round() / 1000.0
    }
}
