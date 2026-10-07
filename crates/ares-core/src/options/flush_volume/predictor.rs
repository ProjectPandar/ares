//! Immutable `FlushVolPredictor` datasets; no core filesystem/resource lookup.
use std::{collections::HashMap, sync::LazyLock};

use super::color_distance::distance;

// OrcaSlicer 8500fcdccaa10b5099ac20d252af3a7c560046f1:
// resources/flush/flush_data_standard.txt
// SHA256 9381953a49e231adc61aa90504c44a4a39211a4f6b034d3e8bffc77af4bc189a
const STANDARD: &str = include_str!("data/flush_data_standard.txt");
// resources/flush/flush_data_dual_standard.txt
// SHA256 beb8859336701fd03b5b409f6bc6f4f69fd3e3e68dd3f77a95cad45b93104586
const DUAL_STANDARD: &str = include_str!("data/flush_data_dual_standard.txt");
// resources/flush/flush_data_dual_highflow.txt
// SHA256 2a3499a5149ee79a5f05fed3637692bccd9355d4e684628af2dfde49c44359de
const DUAL_HIGHFLOW: &str = include_str!("data/flush_data_dual_highflow.txt");

struct Dataset {
    colors: Vec<[u8; 3]>,
    pairs: HashMap<[u8; 6], f32>,
}

impl Dataset {
    fn parse(text: &str) -> Self {
        let rgb = |text: &str| {
            std::array::from_fn(|i| {
                u8::from_str_radix(&text[1 + i * 2..3 + i * 2], 16)
                    .expect("embedded upstream RGB hex")
            })
        };
        let mut lines = text.lines();
        lines.next();
        let colors = lines
            .next()
            .expect("embedded upstream palette")
            .split_whitespace()
            .map(rgb)
            .collect();
        lines.next();
        let mut pairs = HashMap::new();
        for line in lines {
            let mut fields = line.split_whitespace();
            let from: [u8; 3] = rgb(fields.next().expect("embedded upstream source RGB"));
            let to: [u8; 3] = rgb(fields.next().expect("embedded upstream destination RGB"));
            let volume = fields
                .next()
                .expect("embedded upstream flush volume")
                .parse()
                .expect("embedded upstream numeric flush volume");
            pairs
                .entry([from[0], from[1], from[2], to[0], to[1], to[2]])
                .or_insert(volume);
        }
        Self { colors, pairs }
    }

    fn predict(&self, from: [u8; 3], to: [u8; 3]) -> Option<f32> {
        // Upstream picks the FIRST similar entry, not the nearest colour.
        let from = self
            .colors
            .iter()
            .find(|&&rgb| distance(rgb, from) <= 5.0)?;
        let to = self.colors.iter().find(|&&rgb| distance(rgb, to) <= 5.0)?;
        self.pairs
            .get(&[from[0], from[1], from[2], to[0], to[1], to[2]])
            .copied()
    }
}

static DATASETS: LazyLock<[Dataset; 3]> =
    LazyLock::new(|| [STANDARD, DUAL_STANDARD, DUAL_HIGHFLOW].map(Dataset::parse));

pub(super) fn predict(dataset: i32, from: [u8; 3], to: [u8; 3]) -> Option<f32> {
    match dataset {
        0 => DATASETS[0].predict(from, to),
        1 => DATASETS[1].predict(from, to),
        2 => DATASETS[2].predict(from, to),
        // Upstream has no resource file for other ids: prediction misses.
        _ => None,
    }
}
