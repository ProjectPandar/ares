//! `TreeSupport3D.cpp::RichInterfacePlacer`.

use std::collections::{HashSet, VecDeque};

use super::super::model_volumes::TreeModelVolumes;
use super::{
    interface_placer::InterfacePlacer,
    support_element::{AreaIncreaseSettings, StateBits, SupportElement, SupportElementState},
    tip_lines::{self, LineInformations, LineStatus},
};
use crate::geometry::{ClipperError, Coord, CoordinateScale, Point, Polygon};

pub(super) struct TipLayers {
    pub(super) insert_layer: usize,
    pub(super) roof_tip_layers: usize,
    pub(super) supports_roof_layers: usize,
    pub(super) dont_move_until: usize,
}

struct InfluenceTip {
    insert_layer: usize,
    dont_move_until: usize,
    recovery_depth: usize,
    skip_ovalisation: bool,
}

pub(super) struct RichInterfacePlacer<'a, 'v, 's> {
    pub(super) interface: &'a mut InterfacePlacer<'s>,
    pub(super) volumes: &'a mut TreeModelVolumes<'v>,
    pub(super) force_tip_to_roof: bool,
    move_bounds: &'a mut [VecDeque<SupportElement>],
    already_inserted: Vec<HashSet<(Coord, Coord)>>,
    base_radius: Coord,
    base_circle: Polygon,
}

impl<'a, 'v, 's> RichInterfacePlacer<'a, 'v, 's> {
    pub(super) fn new(
        interface: &'a mut InterfacePlacer<'s>,
        volumes: &'a mut TreeModelVolumes<'v>,
        force_tip_to_roof: bool,
        move_bounds: &'a mut [VecDeque<SupportElement>],
        scale: CoordinateScale,
    ) -> Self {
        let base_radius = scale.scaled_delta(0.01) as Coord;
        let base_circle = super::make_circle(base_radius as f64);
        let already_inserted = (0..move_bounds.len()).map(|_| HashSet::new()).collect();
        Self {
            interface,
            volumes,
            force_tip_to_roof,
            move_bounds,
            already_inserted,
            base_radius,
            base_circle,
        }
    }

    pub(super) fn add_points_along_lines(
        &mut self,
        mut lines: LineInformations,
        layers: TipLayers,
    ) -> Result<(), ClipperError> {
        let TipLayers {
            insert_layer,
            roof_tip_layers,
            supports_roof_layers,
            dont_move_until,
        } = layers;
        let mut roof_depth = 0;
        while roof_depth < roof_tip_layers && insert_layer - roof_depth >= 1 {
            let layer = insert_layer - roof_depth;
            let recovery_depth = roof_depth + supports_roof_layers;
            let config = self.interface.config;
            let (keep, mut points) = tip_lines::split(&lines, |point| {
                tip_lines::valid_on_next_layer(point, layer, self.volumes, config)
            })?;
            // Native evaluateRoofWillGenerate is unconditionally true.
            let (keep, free) = tip_lines::split(&keep, |_| Ok(true))?;
            lines = keep;
            points.extend(free);
            for point in points.into_iter().flatten() {
                self.add_point(
                    point,
                    InfluenceTip {
                        insert_layer: layer,
                        dont_move_until: roof_tip_layers - roof_depth,
                        recovery_depth,
                        skip_ovalisation: false,
                    },
                );
            }
            let factor = (config.min_radius / self.base_radius) as f64;
            let roofs = lines
                .iter()
                .flat_map(|line| line.iter().map(|&(point, _)| self.circle_at(point, factor)))
                .collect();
            self.interface.add_roof(roofs, layer, recovery_depth);
            roof_depth += 1;
        }
        let config = self.interface.config;
        for line in lines {
            let skip_ovalisation = config.min_radius < 3 * config.mesh.support_line_width
                && roof_tip_layers == 0
                && roof_depth == 0
                && line.len() > 5;
            for point in line {
                self.add_point(
                    point,
                    InfluenceTip {
                        insert_layer: insert_layer - roof_depth,
                        dont_move_until: dont_move_until.saturating_sub(roof_depth),
                        recovery_depth: roof_depth + supports_roof_layers,
                        skip_ovalisation,
                    },
                );
            }
        }
        Ok(())
    }

    fn add_point(&mut self, (point, status): (Point, LineStatus), layers: InfluenceTip) {
        let to_buildplate = matches!(
            status,
            LineStatus::ToBuildplate | LineStatus::ToBuildplateSafe
        );
        let config = self.interface.config;
        if !config.support_rests_on_model && !to_buildplate {
            return;
        }
        let circle = self.circle_at(point, 1.0);
        let step = (config.min_radius + 1) / 10;
        let key = (point.x() / step, point.y() / step);
        if !self.already_inserted[layers.insert_layer].insert(key) {
            return;
        }
        let pending_layers = if self.force_tip_to_roof {
            layers.dont_move_until as u32
        } else {
            0
        };
        let state = SupportElementState {
            bits: StateBits {
                to_buildplate,
                to_model_gracious: to_buildplate
                    || matches!(
                        status,
                        LineStatus::ToModelGracious | LineStatus::ToModelGraciousSafe
                    ),
                use_min_xy_dist: config.xy_distance > config.xy_min_distance,
                supports_roof: layers.recovery_depth > 0,
                can_use_safe_radius: matches!(
                    status,
                    LineStatus::ToBuildplateSafe | LineStatus::ToModelGraciousSafe
                ),
                skip_ovalisation: layers.skip_ovalisation,
                ..StateBits::default()
            },
            target_height: layers.insert_layer,
            target_position: point,
            next_position: point,
            layer_index: layers.insert_layer,
            effective_radius_height: 0,
            distance_to_top: 0,
            result_on_layer: Some(point),
            increased_to_model_radius: 0,
            elephant_foot_increases: 0.0,
            dont_move_until: layers.dont_move_until as u32,
            last_area_increase: AreaIncreaseSettings::default(),
            missing_roof_layers: pending_layers,
            roof_recovery_depth: if pending_layers > 0 {
                layers.recovery_depth as u32
            } else {
                0
            },
        };
        self.move_bounds[layers.insert_layer].push_back(SupportElement {
            state,
            parents: Vec::new(),
            influence_area: vec![circle],
        });
    }

    fn circle_at(&self, position: Point, factor: f64) -> Polygon {
        Polygon::new(
            self.base_circle
                .points()
                .iter()
                .map(|point| {
                    Point::new(
                        (point.x() as f64 * factor) as Coord + position.x(),
                        (point.y() as f64 * factor) as Coord + position.y(),
                    )
                })
                .collect(),
        )
    }
}
