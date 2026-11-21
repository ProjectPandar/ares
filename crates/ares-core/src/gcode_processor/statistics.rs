//! `GCodeStatistics.cpp::gcode_statistics_json` over a completed processor result.

use std::{collections::BTreeMap, f64::consts::PI};

use serde::Serialize;

use super::{EMoveType, GCodeProcessorResult};
use crate::ExtrusionRole;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ObjectMass {
    pub name: String,
    pub instance: usize,
    pub volume: f64,
    pub mass: f64,
    pub support_volume: f64,
    pub support_mass: f64,
}

#[derive(Serialize)]
pub struct GCodeStatistics<'a> {
    version: u32,
    generator: &'static str,
    units: Units,
    summary: Summary,
    layers: Vec<Layer>,
    objects: Vec<Object<'a>>,
}

#[derive(Serialize)]
struct Units {
    time: &'static str,
    length: &'static str,
    volume: &'static str,
    weight: &'static str,
}

#[derive(Serialize)]
struct Summary {
    time: f64,
    prepare_time: f64,
    travel_time: f64,
    travel_distance: f64,
    layer_count: usize,
    volume: f64,
    weight: f64,
    filament_cost: f64,
    filament_changes: usize,
    filaments: Vec<Filament>,
    features: Features,
}

type Features = BTreeMap<&'static str, Usage>;

#[derive(Default, Serialize)]
struct Layer {
    z: f64,
    time: f64,
    travel_time: f64,
    volume: f64,
    weight: f64,
    features: Features,
}

#[derive(Serialize)]
struct Filament {
    id: usize,
    length: f64,
    volume: f64,
    weight: f64,
    cost: f64,
    model: Material,
    support: Material,
    prime_tower: Material,
    flush: Material,
}

#[derive(Serialize)]
struct Material {
    volume: f64,
    weight: f64,
}

#[derive(Default, Serialize)]
struct Usage {
    time: f64,
    volume: f64,
    weight: f64,
}

#[derive(Serialize)]
struct Object<'a> {
    name: &'a str,
    instance: usize,
    volume: f64,
    weight: f64,
    support_volume: f64,
    support_weight: f64,
}

fn rounded(value: f64, scale: f64) -> f64 {
    (value * scale).round() / scale
}

pub(crate) fn density(densities: &[f32], id: usize) -> f64 {
    f64::from(
        densities
            .get(id)
            .copied()
            .filter(|&d| d > 0.0)
            .unwrap_or(1.245),
    )
}

impl Usage {
    fn add(&mut self, time: f64, volume: f64, weight: f64) {
        self.time += time;
        self.volume += volume;
        self.weight += weight;
    }
}

fn round_features(features: &mut Features) {
    for usage in features.values_mut() {
        usage.time = rounded(usage.time, 1000.0);
        usage.volume = rounded(usage.volume, 1000.0);
        usage.weight = rounded(usage.weight, 10000.0);
    }
}

impl GCodeProcessorResult {
    /// Statistics from the recorded Normal-mode vertices and processor totals.
    pub fn statistics(&self) -> GCodeStatistics<'_> {
        let area = |id: usize| PI * (0.5 * f64::from(self.filament_diameters[id])).powi(2);
        let weight = |id, volume| 0.001 * volume * density(&self.filament_densities, id);
        let mut layers = Vec::<Layer>::new();
        let mut features = Features::new();
        let mut travel_time = 0.0;
        for vertex in &self.moves {
            let index = vertex.layer_id as usize;
            if layers.len() <= index {
                layers.resize_with(index + 1, Layer::default);
            }
            let layer = &mut layers[index];
            let time = f64::from(vertex.time);
            layer.time += time;
            if vertex.move_type == EMoveType::Travel {
                layer.travel_time += time;
                travel_time += time;
            }
            if vertex.move_type != EMoveType::Extrude || vertex.internal_only {
                continue;
            }
            if vertex.extrusion_role != ExtrusionRole::Custom {
                layer.z = f64::from(vertex.position[2]);
            }
            let id = usize::from(vertex.extruder_id);
            let volume = f64::from(vertex.delta_extruder) * area(id);
            let weight = weight(id, volume);
            layer.volume += volume;
            layer.weight += weight;
            let role = role_name(vertex.extrusion_role);
            layer
                .features
                .entry(role)
                .or_default()
                .add(time, volume, weight);
            features.entry(role).or_default().add(time, volume, weight);
        }
        for layer in &mut layers {
            layer.z = rounded(layer.z, 1000.0);
            layer.time = rounded(layer.time, 1000.0);
            layer.travel_time = rounded(layer.travel_time, 1000.0);
            layer.volume = rounded(layer.volume, 1000.0);
            layer.weight = rounded(layer.weight, 10000.0);
            round_features(&mut layer.features);
        }
        round_features(&mut features);
        let stats = &self.print_statistics;
        let (mut total_volume, mut total_weight, mut total_cost) = (0.0, 0.0, 0.0);
        let filaments = stats
            .total_volumes_per_extruder
            .iter()
            .copied()
            .enumerate()
            .filter(|&(_, volume)| volume != 0.0)
            .map(|(id, volume)| {
                let usage = |bucket: &[f64]| {
                    let used = bucket[id];
                    Material {
                        volume: rounded(used, 1000.0),
                        weight: rounded(weight(id, used), 10000.0),
                    }
                };
                let filament_weight = weight(id, volume);
                let cost = 0.001
                    * filament_weight
                    * f64::from(self.filament_costs.get(id).copied().unwrap_or_default());
                total_volume += volume;
                total_weight += filament_weight;
                total_cost += cost;
                Filament {
                    id: id + 1,
                    length: rounded(volume / area(id), 100.0),
                    volume: rounded(volume, 1000.0),
                    weight: rounded(filament_weight, 10000.0),
                    cost: rounded(cost, 100.0),
                    model: usage(&stats.model_volumes_per_extruder),
                    support: usage(&stats.support_volumes_per_extruder),
                    prime_tower: usage(&stats.wipe_tower_volumes_per_extruder),
                    flush: usage(&stats.flush_per_filament),
                }
            })
            .collect();
        GCodeStatistics {
            version: 1,
            generator: concat!("Ares ", env!("CARGO_PKG_VERSION")),
            units: Units {
                time: "s",
                length: "mm",
                volume: "mm3",
                weight: "g",
            },
            summary: Summary {
                time: rounded(f64::from(stats.normal_time), 1000.0),
                prepare_time: rounded(f64::from(stats.normal_prepare_time), 1000.0),
                travel_time: rounded(travel_time, 1000.0),
                travel_distance: rounded(f64::from(stats.total_travel_distance), 100.0),
                layer_count: layers.len(),
                volume: rounded(total_volume, 1000.0),
                weight: rounded(total_weight, 10000.0),
                filament_cost: rounded(total_cost, 100.0),
                filament_changes: stats.total_filament_changes,
                filaments,
                features,
            },
            layers,
            objects: self
                .object_masses
                .iter()
                .map(|object| Object {
                    name: &object.name,
                    instance: object.instance,
                    volume: rounded(object.volume, 1000.0),
                    weight: rounded(0.001 * object.mass, 10000.0),
                    support_volume: rounded(object.support_volume, 1000.0),
                    support_weight: rounded(0.001 * object.support_mass, 10000.0),
                })
                .collect(),
        }
    }
}

impl GCodeStatistics<'_> {
    pub fn to_json_bytes(&self) -> Result<Vec<u8>, serde_json::Error> {
        serde_json::to_vec_pretty(self)
    }
}

fn role_name(role: ExtrusionRole) -> &'static str {
    use ExtrusionRole::*;
    match role {
        None => "Undefined",
        Perimeter => "Inner wall",
        ExternalPerimeter => "Outer wall",
        OverhangPerimeter => "Overhang wall",
        InternalInfill => "Sparse infill",
        SolidInfill => "Internal solid infill",
        TopSolidInfill => "Top surface",
        BottomSurface => "Bottom surface",
        Ironing => "Ironing",
        BridgeInfill => "Bridge",
        InternalBridgeInfill => "Internal Bridge",
        GapFill => "Gap infill",
        Skirt => "Skirt",
        Brim => "Brim",
        SupportMaterial => "Support",
        SupportMaterialInterface => "Support interface",
        SupportTransition => "Support transition",
        WipeTower => "Prime tower",
        Custom => "Custom",
        Mixed => "Multiple",
    }
}
