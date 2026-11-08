// `FuzzySkin.cpp:345-412::fuzzy_extrusion_line`: preserve destination
// junction width/index and C++ coordinate truncation, using the task RNG.
use super::{Layer, config::Config};
use crate::{
    Point2, ProcessFuzzySkinMode, SlicerContext,
    arachne::ExtrusionJunction,
    geometry::Point,
    perimeters::{FuzzySkinNoiseType, coherent_value},
};

pub(super) fn fuzzify(
    junctions: &mut Vec<ExtrusionJunction>,
    config: Config,
    Layer {
        id: layer_id,
        slice_z,
        layer_height,
        scale,
    }: Layer,
    context: &mut SlicerContext,
    closed: bool,
) {
    if config.base.noise_type == FuzzySkinNoiseType::Ripple {
        if junctions.len() >= 3 {
            super::ripple::fuzzify(junctions, config, layer_id, scale);
        }
        return;
    }
    let minimum_step = config.point_distance as f64 * 3.0 / 4.0;
    let random_step = config.point_distance as f64 / 2.0;
    let min_extrusion_width =
        scale.scaled_delta(layer_height * (1.0 - 0.25 * std::f64::consts::PI) * 1.05);
    let mut left_over = context.rng().unit() * (minimum_step / 2.0);
    let mut previous = junctions[0];
    let mut output = Vec::with_capacity(junctions.len());
    for &next in junctions.iter() {
        if closed && previous.point == next.point {
            output.push(next);
            continue;
        }
        let dx = (next.point.x() - previous.point.x()) as f64;
        let dy = (next.point.y() - previous.point.y()) as f64;
        let length = (dx * dx + dy * dy).sqrt();
        let mut distance = left_over;
        while distance < length {
            let ratio = distance / length;
            let point = Point::new(
                previous.point.x() + (dx * ratio) as i64,
                previous.point.y() + (dy * ratio) as i64,
            );
            let noise = if config.base.noise_type == FuzzySkinNoiseType::Classic {
                context.rng().unit() * 2.0 - 1.0
            } else {
                coherent_value(
                    config.base.noise_type,
                    Point2::new(scale.unscale(point.x()), scale.unscale(point.y())),
                    slice_z,
                    config.base,
                )
            };
            let radius = noise * config.thickness as f64;
            let width = (next.width as f64 + radius + min_extrusion_width).max(min_extrusion_width);
            let (displacement, width) = match config.mode {
                ProcessFuzzySkinMode::Displacement => (radius, next.width),
                ProcessFuzzySkinMode::Extrusion => (0.0, width as i64),
                ProcessFuzzySkinMode::Combined => ((width - next.width as f64) / 2.0, width as i64),
            };
            output.push(ExtrusionJunction::new(
                Point::new(
                    point.x() + (-dy / length * displacement) as i64,
                    point.y() + (dx / length * displacement) as i64,
                ),
                width,
                next.perimeter_index,
            ));
            distance += minimum_step + context.rng().unit() * random_step;
        }
        left_over = distance - length;
        previous = next;
    }
    pad(&mut output, junctions);
    if junctions.last().expect("a fuzzy line has junctions").point == junctions[0].point {
        let last = *output.last().expect("padding supplies a junction");
        output[0].point = last.point;
        output[0].width = last.width;
    }
    if output.len() >= 3 {
        *junctions = output;
    }
}

pub(super) fn pad(output: &mut Vec<ExtrusionJunction>, source: &[ExtrusionJunction]) {
    while output.len() < 3 {
        // Upstream declares this index inside the loop: the same penultimate
        // junction is repeated, including its width and perimeter index.
        let index = source.len() - 2;
        output.push(source[index]);
        if index == 0 {
            break;
        }
    }
}
