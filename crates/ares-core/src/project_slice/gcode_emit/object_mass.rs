//! Instance volume and mass from `GCode.cpp::set_mass_locator` and
//! `GCodeProcessor.cpp:7644–7715`. Body indices and inertia are not exported.

use crate::ExtrusionRole;
use crate::gcode_processor::ObjectMass;
use crate::geometry::{BoundingBox, CoordinateScale, ExPolygon, Line, Point};
use crate::project_slice::perimeters::classic::traversal::PreparedPostClassicTraversal;

pub(super) struct Locator<'a> {
    objects: Vec<Vec<Layer<'a>>>,
    instances: Vec<Instance>,
    scale: CoordinateScale,
    origin: (f64, f64),
}

struct Layer<'a> {
    z: f64,
    islands: Vec<(&'a ExPolygon, BoundingBox)>,
}

struct Instance {
    object: usize,
    index: usize,
    name: String,
    shift: Point,
    footprint: BoundingBox,
    crowded: bool,
}

fn mark_crowded(footprint: BoundingBox, earlier: &mut [Instance]) -> bool {
    let mut crowded = false;
    for other in earlier {
        if other.footprint.overlap(footprint) {
            crowded = true;
            other.crowded = true;
        }
    }
    crowded
}

fn support_distance(bb: BoundingBox, point: Point) -> (bool, i128) {
    let inside = bb.contains(point);
    let axes = [
        (point.x(), bb.min().x(), bb.max().x()),
        (point.y(), bb.min().y(), bb.max().y()),
    ];
    let distance = axes
        .into_iter()
        .map(|(p, min, max)| {
            let delta = if inside {
                p - (min + max) / 2
            } else {
                (min - p).max(p - max).max(0)
            } as i128;
            delta * delta
        })
        .sum();
    (!inside, distance)
}

impl Layer<'_> {
    fn distance(&self, point: Point, crowded: bool) -> (bool, f64) {
        let mut distance = f64::INFINITY;
        for &(slice, bounds) in &self.islands {
            if !bounds.contains(point) {
                continue;
            }
            if !crowded
                || slice.contour().contains(&point)
                    && !slice.holes().iter().any(|hole| hole.contains(&point))
            {
                return (true, 0.0);
            }
            let gap = slice
                .lines()
                .into_iter()
                .map(|edge: Line| edge.distance_to_squared(point))
                .fold(f64::INFINITY, f64::min);
            distance = distance.min(gap);
        }
        (false, distance)
    }
}

impl<'a> Locator<'a> {
    pub(super) fn new(traversal: &'a PreparedPostClassicTraversal) -> Self {
        let scale = traversal.scale;
        let margin = scale.checked_scale(1.0).unwrap();
        let mut instances = Vec::<Instance>::new();
        let mut objects = Vec::new();
        for (index, object) in traversal.objects.iter().enumerate() {
            let (resolved, group) = traversal.print_object_config(index);
            let model = &traversal.project.objects()[resolved.source_object_index];
            let layers = object
                .physical_layers()
                .iter()
                .enumerate()
                .map(|(index, layer)| Layer {
                    z: layer.print_z,
                    islands: object
                        .slices(index)
                        .unwrap_or_default()
                        .iter()
                        .filter_map(|slice| {
                            let bounds = BoundingBox::from_expolygon(slice)?;
                            let min = bounds.min();
                            let max = bounds.max();
                            Some((
                                slice,
                                BoundingBox::new(
                                    Point::new(min.x() - margin, min.y() - margin),
                                    Point::new(max.x() + margin, max.y() + margin),
                                ),
                            ))
                        })
                        .collect(),
                })
                .collect::<Vec<_>>();
            let Some(footprint) = BoundingBox::from_points(layers.iter().flat_map(|layer| {
                layer
                    .islands
                    .iter()
                    .flat_map(|(_, bb)| [bb.min(), bb.max()])
            })) else {
                continue;
            };
            let center = crate::project_slice::centering::center(model, group, scale)
                .expect("sliced object retains its raw center");
            for &member in &group.instance_indices {
                let shift =
                    super::footprint::instance_shift(&model.instances()[member], center, scale);
                let shifted = |p: Point| Point::new(p.x() + shift.x(), p.y() + shift.y());
                let footprint =
                    BoundingBox::new(shifted(footprint.min()), shifted(footprint.max()));
                let crowded = mark_crowded(footprint, &mut instances);
                instances.push(Instance {
                    object: objects.len(),
                    index: member,
                    name: model.name().to_owned(),
                    shift,
                    footprint,
                    crowded,
                });
            }
            objects.push(layers);
        }
        Self {
            objects,
            instances,
            scale,
            origin: traversal.project.plate_origin(),
        }
    }

    pub(super) fn masses(&self) -> Vec<ObjectMass> {
        self.instances
            .iter()
            .map(|instance| ObjectMass {
                name: instance.name.clone(),
                instance: instance.index,
                ..Default::default()
            })
            .collect()
    }

    fn locate(&self, [x, y, z]: [f64; 3], support: bool) -> Option<usize> {
        let point = Point::new(
            self.scale.checked_scale(x + self.origin.0)?,
            self.scale.checked_scale(y + self.origin.1)?,
        );
        if support {
            return self
                .instances
                .iter()
                .enumerate()
                .min_by_key(|(_, instance)| support_distance(instance.footprint, point))
                .map(|(index, _)| index);
        }
        let mut best = None;
        let mut distance = f64::INFINITY;
        for (index, instance) in self.instances.iter().enumerate() {
            if !instance.footprint.contains(point) {
                continue;
            }
            let layers = &self.objects[instance.object];
            let Some(layer) = layers.get(layers.partition_point(|layer| layer.z < z - 0.002))
            else {
                continue;
            };
            let local = Point::new(
                point.x() - instance.shift.x(),
                point.y() - instance.shift.y(),
            );
            let (inside, gap) = layer.distance(local, instance.crowded);
            if inside {
                return Some(index);
            }
            if gap < distance {
                distance = gap;
                best = Some(index);
            }
        }
        best
    }

    pub(super) fn add(
        &self,
        masses: &mut [ObjectMass],
        role: ExtrusionRole,
        point: [f64; 3],
        (volume, density): (f64, f64),
    ) {
        use ExtrusionRole::*;
        if volume <= 0.0 || matches!(role, None | Skirt | Brim | WipeTower | Custom | Mixed) {
            return;
        }
        let support = matches!(
            role,
            SupportMaterial | SupportMaterialInterface | SupportTransition
        );
        if let Some(id) = self.locate(point, support) {
            let mass = &mut masses[id];
            if support {
                mass.support_volume += volume;
                mass.support_mass += volume * density;
            } else {
                mass.volume += volume;
                mass.mass += volume * density;
            }
        }
    }
}
