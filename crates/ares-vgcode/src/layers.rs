// Ports rendering-neutral data from AGPL-licensed OrcaSlicer `src/libvgcode/src/Layers.hpp` and `src/Layers.cpp`.

use crate::{GCodeExtrusionRole, Interval, MoveType, PathVertex, Range, TimeMode};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Layers {
    items: Vec<Item>,
    view_range: Range,
}

impl Layers {
    pub fn update(&mut self, vertex: &PathVertex, vertex_id: u32) {
        if self.items.is_empty() || vertex.layer_id as usize == self.items.len() {
            assert_eq!(vertex.layer_id as usize, self.items.len());
            let mut item = Item::default();
            if is_z_capturing_extrusion(vertex) {
                item.z = vertex.position[2];
            }
            item.range.set(vertex_id as usize, vertex_id as usize);
            item.times = vertex.times;
            item.contains_colorprint_options |= is_colorprint_option(vertex);
            self.items.push(item);
        } else {
            let item = self.items.last_mut().expect("layers has a last item");
            if is_z_capturing_extrusion(vertex) && item.z != vertex.position[2] {
                item.z = vertex.position[2];
            }
            item.range.set_max(vertex_id as usize);
            for i in 0..TimeMode::COUNT {
                item.times[i] += vertex.times[i];
            }
            item.contains_colorprint_options |= is_colorprint_option(vertex);
        }
    }

    pub fn reset(&mut self) {
        self.items.clear();
        self.view_range.reset();
    }

    pub fn empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn count(&self) -> usize {
        self.items.len()
    }

    pub fn get_times(&self, mode: TimeMode) -> Vec<f32> {
        self.items
            .iter()
            .map(|item| item.times[mode.index()])
            .collect()
    }

    pub fn get_zs(&self) -> Vec<f32> {
        self.items.iter().map(|item| item.z).collect()
    }

    pub fn get_layer_time(&self, mode: TimeMode, layer_id: usize) -> f32 {
        self.items
            .get(layer_id)
            .map_or(0.0, |item| item.times[mode.index()])
    }

    pub fn get_layer_z(&self, layer_id: usize) -> f32 {
        self.items.get(layer_id).map_or(0.0, |item| item.z)
    }

    pub fn get_layer_id_at(&self, z: f32) -> usize {
        let mut first = 0;
        let mut count = self.items.len();
        while count > 0 {
            let step = count / 2;
            let index = first + step;
            if self.items[index].z < z {
                count = step;
            } else {
                first = index + 1;
                count -= step + 1;
            }
        }
        first
    }

    pub fn get_view_range(&self) -> Interval {
        self.view_range.get()
    }

    pub fn set_view_range_interval(&mut self, range: Interval) {
        self.set_view_range(range[0], range[1]);
    }

    pub fn set_view_range(&mut self, min: usize, max: usize) {
        self.view_range.set(min, max);
    }

    pub fn layer_contains_colorprint_options(&self, layer_id: usize) -> bool {
        self.items
            .get(layer_id)
            .is_some_and(|item| item.contains_colorprint_options)
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
struct Item {
    z: f32,
    range: Range,
    times: [f32; TimeMode::COUNT],
    contains_colorprint_options: bool,
}

fn is_z_capturing_extrusion(vertex: &PathVertex) -> bool {
    vertex.move_type == MoveType::Extrude && vertex.role != GCodeExtrusionRole::Custom
}

fn is_colorprint_option(vertex: &PathVertex) -> bool {
    matches!(
        vertex.move_type,
        MoveType::PausePrint | MoveType::CustomGCode
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn vertex(
        layer_id: u32,
        z: f32,
        move_type: MoveType,
        role: GCodeExtrusionRole,
        times: [f32; 2],
    ) -> PathVertex {
        PathVertex {
            layer_id,
            position: [0.0, 0.0, z],
            move_type,
            role,
            times,
            ..PathVertex::default()
        }
    }

    #[test]
    fn update_accumulates_layer_times_z_and_view_range_data() {
        let mut layers = Layers::default();
        layers.update(
            &vertex(
                0,
                0.2,
                MoveType::Extrude,
                GCodeExtrusionRole::Perimeter,
                [1.0, 2.0],
            ),
            7,
        );
        layers.update(
            &vertex(
                0,
                0.2,
                MoveType::Travel,
                GCodeExtrusionRole::None,
                [3.0, 4.0],
            ),
            8,
        );
        layers.update(
            &vertex(
                1,
                0.4,
                MoveType::Extrude,
                GCodeExtrusionRole::InternalInfill,
                [5.0, 6.0],
            ),
            9,
        );

        assert_eq!(layers.count(), 2);
        assert_eq!(layers.get_zs(), vec![0.2, 0.4]);
        assert_eq!(layers.get_times(TimeMode::Normal), vec![4.0, 5.0]);
        assert_eq!(layers.get_times(TimeMode::Stealth), vec![6.0, 6.0]);
        assert_eq!(layers.get_layer_time(TimeMode::Normal, 0), 4.0);
        assert_eq!(layers.get_layer_z(1), 0.4);
    }

    #[test]
    fn update_detects_pause_and_custom_gcode_colorprint_options() {
        let mut layers = Layers::default();
        layers.update(
            &vertex(
                0,
                0.0,
                MoveType::PausePrint,
                GCodeExtrusionRole::None,
                [0.0, 0.0],
            ),
            0,
        );
        layers.update(
            &vertex(
                1,
                1.0,
                MoveType::CustomGCode,
                GCodeExtrusionRole::None,
                [0.0, 0.0],
            ),
            1,
        );

        assert!(layers.layer_contains_colorprint_options(0));
        assert!(layers.layer_contains_colorprint_options(1));
        assert!(!layers.layer_contains_colorprint_options(2));
    }

    #[test]
    fn custom_extrusion_does_not_capture_layer_z() {
        let mut layers = Layers::default();
        layers.update(
            &vertex(
                0,
                2.0,
                MoveType::Extrude,
                GCodeExtrusionRole::Custom,
                [0.0, 0.0],
            ),
            0,
        );
        assert_eq!(layers.get_layer_z(0), 0.0);
    }

    #[test]
    fn layer_lookup_returns_first_layer_with_z_at_or_above_query() {
        let mut layers = Layers::default();
        layers.update(
            &vertex(
                0,
                0.2,
                MoveType::Extrude,
                GCodeExtrusionRole::Perimeter,
                [0.0, 0.0],
            ),
            0,
        );
        layers.update(
            &vertex(
                1,
                0.4,
                MoveType::Extrude,
                GCodeExtrusionRole::Perimeter,
                [0.0, 0.0],
            ),
            1,
        );

        assert_eq!(layers.get_layer_id_at(0.1), 2);
        assert_eq!(layers.get_layer_id_at(0.2), 2);
        assert_eq!(layers.get_layer_id_at(0.3), 2);
        assert_eq!(layers.get_layer_id_at(0.4), 2);
        assert_eq!(layers.get_layer_id_at(0.5), 0);
    }

    #[test]
    fn reset_clears_layers_and_view_range() {
        let mut layers = Layers::default();
        layers.update(
            &vertex(
                0,
                0.2,
                MoveType::Extrude,
                GCodeExtrusionRole::Perimeter,
                [1.0, 1.0],
            ),
            3,
        );
        layers.set_view_range(1, 3);
        layers.reset();

        assert!(layers.empty());
        assert_eq!(layers.get_view_range(), [0, 0]);
    }
}
