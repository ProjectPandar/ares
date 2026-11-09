// SeamContour.cpp::Resolver::select: bounds certificate, local witnesses,
// reverse certification and two-criterion dominance over immutable contours.
use super::{
    Contour, Resolver, Result,
    geometry::{box_distance, closest, dot, normals},
};

struct Witness {
    contour: usize,
    distance: f64,
    angle_degrees: f64,
}

impl Resolver {
    pub(super) fn select(
        &self,
        (layer, contour): (usize, usize),
        point: [f64; 2],
        (direction, radius): (isize, f64),
        check_reverse: bool,
    ) -> Result {
        let target = (layer as isize + direction) as usize;
        let source = &self.layers[layer][contour];
        let next = &self.layers[target];
        let bounds = source.nozzle * self.options.bounds_nozzle_factor;
        let noise = source.width * self.options.noise_width_factor;
        let reach = radius.max(bounds).max(noise);
        let candidates = self.indices[target].query(point, reach);
        if self.indices[layer]
            .query(point, reach)
            .into_iter()
            .any(|i| !self.layers[layer][i].reliable)
            || candidates.iter().any(|&i| !next[i].reliable)
        {
            return Result::Uncertain;
        }
        let mut fast = Vec::new();
        let mut nearby = Vec::new();
        for index in candidates {
            let candidate = &next[index];
            if candidate.points.len() < 3 {
                continue;
            }
            if box_distance(candidate, point) <= radius {
                nearby.push(index);
            }
            if candidate.hole == source.hole
                && (0..2).all(|axis| (candidate.min[axis] - source.min[axis]).abs() <= bounds)
                && (0..2).all(|axis| (candidate.max[axis] - source.max[axis]).abs() <= bounds)
            {
                fast.push(index);
            }
        }
        let source_normals = normals(source, point);
        let compatible_foot = |candidate: &Contour| {
            closest(
                candidate,
                point,
                -1.0,
                (candidate.hole == source.hole).then_some(source_normals.as_slice()),
                self.options.normal_cosine,
            )
        };
        let normal_agreement = |candidate: &Contour, foot| {
            let target_normals = normals(candidate, foot);
            source_normals
                .iter()
                .flat_map(|&a| target_normals.iter().map(move |&b| dot(a, b)))
                .fold(-1.0_f64, f64::max)
        };
        let bounds_result = 'bounds: {
            let [index] = fast.as_slice() else {
                break 'bounds None;
            };
            let index = *index;
            let foot = compatible_foot(&next[index]);
            if !(foot.distance <= noise
                && normal_agreement(&next[index], foot.point) >= self.options.normal_cosine)
            {
                break 'bounds None;
            }
            let competing = nearby
                .iter()
                .filter(|&&other| other != index && box_distance(&next[other], point) <= noise)
                .any(|&other| {
                    let alternative = compatible_foot(&next[other]);
                    alternative.distance <= noise
                        && normal_agreement(&next[other], alternative.point)
                            >= self.options.normal_cosine
                });
            if competing {
                break 'bounds None;
            }
            if !check_reverse {
                break 'bounds Some(Result::UniqueBounds(index));
            }
            let back = self.select((target, index), foot.point, (-direction, radius), false);
            if matches!(back, Result::Uncertain) {
                break 'bounds Some(back);
            }
            back.contours()
                .contains(&contour)
                .then_some(Result::UniqueBounds(index))
        };
        if let Some(result) = bounds_result {
            return result;
        }
        let mut witnessed = Vec::new();
        for index in nearby {
            let foot = compatible_foot(&next[index]);
            if foot.distance > radius {
                continue;
            }
            let agreement = normal_agreement(&next[index], foot.point);
            if agreement < self.options.normal_cosine {
                continue;
            }
            let band = foot.distance <= noise
                || [0.25, 0.5, 0.75].into_iter().all(|t| {
                    let probe = [
                        point[0] + t * (foot.point[0] - point[0]),
                        point[1] + t * (foot.point[1] - point[1]),
                    ];
                    self.in_material(layer, probe) != self.in_material(target, probe)
                        || self.near_boundary(layer, probe, noise)
                        || self.near_boundary(target, probe, noise)
                });
            if !band {
                continue;
            }
            let back = check_reverse
                .then(|| self.select((target, index), foot.point, (-direction, radius), false));
            if matches!(back, Some(Result::Uncertain)) {
                return Result::Uncertain;
            }
            if back
                .as_ref()
                .is_some_and(|back| !back.contours().contains(&contour))
            {
                continue;
            }
            witnessed.push(Witness {
                contour: index,
                distance: foot.distance,
                angle_degrees: agreement.clamp(-1.0, 1.0).acos() * 180.0 / std::f64::consts::PI,
            });
        }
        let distance_margin = (source.width * self.options.dominance_width_factor).max(1e-6);
        let angle_margin = self.options.dominance_angle_degrees.max(1e-6);
        let contours = witnessed
            .iter()
            .filter(|candidate| {
                !witnessed.iter().any(|other| {
                    other.distance + distance_margin <= candidate.distance
                        && other.angle_degrees + angle_margin <= candidate.angle_degrees
                })
            })
            .map(|candidate| candidate.contour)
            .collect::<Vec<_>>();
        match contours.len() {
            0 => Result::None,
            1 => Result::UniqueLocal(contours[0]),
            _ => Result::Multiple(contours),
        }
    }
}
