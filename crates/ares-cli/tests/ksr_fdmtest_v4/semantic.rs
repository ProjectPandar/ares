use std::fmt::Debug;

#[path = "semantic/lifecycle.rs"]
mod lifecycle;
#[path = "semantic/parser.rs"]
mod parser;
#[cfg(test)]
#[path = "semantic/tests.rs"]
mod tests;
#[path = "semantic/travel.rs"]
mod travel;

use parser::{Deposition, SemanticGcode};

const MAX_FEED_DIFFERENCE: f64 = 12.0;
const MAX_FEED_RELATIVE_DIFFERENCE: f64 = 0.01;
const MAX_TIME_DIFFERENCE_SECONDS: u64 = 5;
const MAX_FILAMENT_LENGTH_DIFFERENCE_MM: f64 = 0.05;
const MAX_WIPE_EXTRUSION_DIFFERENCE: f64 = 0.000_011;
const MAX_EXTRUSION_DIFFERENCE: f64 = 0.000_011;
const MAX_COORDINATE_ROUNDING_DIFFERENCE_MM: f64 = 0.001_1;
const MAX_PATH_SEGMENTATION_DIFFERENCE_MM: f64 = 0.07;
const MAX_PATH_EXTRUSION_DIFFERENCE: f64 = 0.002_2;

pub(crate) fn compare(expected: &[u8], actual: &[u8]) -> Result<(), String> {
    let expected = parser::parse(expected)?;
    let actual = parser::parse(actual)?;

    compare_timing(&expected, &actual)?;
    compare_filament_lengths(&expected, &actual)?;
    exact("preamble", &expected.preamble, &actual.preamble)?;
    exact("postamble", &expected.postamble, &actual.postamble)?;
    if expected.layers.len() != actual.layers.len() {
        return Err(format!(
            "layer count differs: expected {}, actual {}",
            expected.layers.len(),
            actual.layers.len()
        ));
    }

    for (index, (expected, actual)) in expected.layers.iter().zip(&actual.layers).enumerate() {
        let layer = index + 1;
        exact_layer(layer, "metadata", &expected.metadata, &actual.metadata)?;
        compare_deposition(layer, &expected.deposition, &actual.deposition)?;
        lifecycle::compare(layer, &expected.lifecycles, &actual.lifecycles)?;
        travel::compare(layer, &expected.travels, &actual.travels)?;
        exact_layer(
            layer,
            "control events",
            &expected.controls,
            &actual.controls,
        )?;
    }
    Ok(())
}

fn compare_timing(expected: &SemanticGcode, actual: &SemanticGcode) -> Result<(), String> {
    for (name, expected, actual) in [
        (
            "model printing time",
            expected.timing.model,
            actual.timing.model,
        ),
        (
            "total estimated time",
            expected.timing.total,
            actual.timing.total,
        ),
        (
            "first layer printing time",
            expected.timing.first_layer,
            actual.timing.first_layer,
        ),
    ] {
        let difference = expected.abs_diff(actual);
        if difference > MAX_TIME_DIFFERENCE_SECONDS {
            return Err(format!(
                "{name} differs by {difference}s: expected {expected}s, actual {actual}s"
            ));
        }
    }
    Ok(())
}

fn compare_filament_lengths(
    expected: &SemanticGcode,
    actual: &SemanticGcode,
) -> Result<(), String> {
    if expected.filament_lengths.len() != actual.filament_lengths.len() {
        return Err(format!(
            "filament length count differs: expected {}, actual {}",
            expected.filament_lengths.len(),
            actual.filament_lengths.len()
        ));
    }
    for (index, (&expected, &actual)) in expected
        .filament_lengths
        .iter()
        .zip(&actual.filament_lengths)
        .enumerate()
    {
        if (expected - actual).abs() > MAX_FILAMENT_LENGTH_DIFFERENCE_MM {
            return Err(format!(
                "filament {} length differs: expected {expected:.2}mm, actual {actual:.2}mm",
                index + 1
            ));
        }
    }
    Ok(())
}

fn compare_deposition(
    layer: usize,
    expected: &[Deposition],
    actual: &[Deposition],
) -> Result<(), String> {
    let expected_segments = parse_segments(expected)?;
    let actual_segments = parse_segments(actual)?;
    let expected_paths = path_ranges(&expected_segments);
    let actual_paths = path_ranges(&actual_segments);
    if expected_paths.len() != actual_paths.len() {
        return Err(format!(
            "layer {layer} deposition path count differs: expected {}, actual {}",
            expected_paths.len(),
            actual_paths.len()
        ));
    }
    for (expected_range, actual_range) in expected_paths.into_iter().zip(actual_paths) {
        let expected_path = &expected_segments[expected_range.clone()];
        let actual_path = &actual_segments[actual_range];
        if exact_path_matches(expected_path, actual_path)
            || rounded_paths_match(expected_path, actual_path)
            || redistributed_paths_match(expected_path, actual_path)
            || segmented_paths_match(expected_path, actual_path)
        {
            continue;
        }
        let index = expected_range.start;
        let mismatch = expected_path
            .iter()
            .zip(actual_path)
            .position(|(expected, actual)| !rounded_segment_matches(expected, actual));
        return Err(format!(
            "layer {layer} deposition {} differs at path segment {:?}: expected {:?}, \
             actual {:?}; path sizes {} / {}, lengths {} / {}, extrusion {} / {}, \
             closure {} / {}, distance {} / {}",
            index + 1,
            mismatch,
            mismatch.and_then(|index| expected_path.get(index).map(|item| item.deposition)),
            mismatch.and_then(|index| actual_path.get(index).map(|item| item.deposition)),
            expected_path.len(),
            actual_path.len(),
            path_length(expected_path),
            path_length(actual_path),
            path_extrusion(expected_path),
            path_extrusion(actual_path),
            distance(expected_path[0].start, expected_path.last().unwrap().end),
            distance(actual_path[0].start, actual_path.last().unwrap().end),
            path_distance(expected_path, actual_path),
            path_distance(actual_path, expected_path)
        ));
    }
    Ok(())
}

struct Segment<'a> {
    deposition: &'a Deposition,
    fields: Vec<&'a str>,
    start: [f64; 3],
    end: [f64; 3],
    extrusion: f64,
}

fn parse_segments(deposition: &[Deposition]) -> Result<Vec<Segment<'_>>, String> {
    deposition
        .iter()
        .map(|deposition| {
            let fields = deposition.key.split('|').collect::<Vec<_>>();
            if fields.len() != 12 {
                return Err(format!("invalid deposition key {:?}", deposition.key));
            }
            let number = |index: usize| {
                let value = fields[index];
                if value.is_empty() {
                    Ok(0.0)
                } else {
                    value
                        .parse::<f64>()
                        .map_err(|_| format!("invalid deposition key {:?}", deposition.key))
                }
            };
            Ok(Segment {
                deposition,
                start: [number(3)?, number(4)?, number(5)?],
                end: [number(6)?, number(7)?, number(8)?],
                extrusion: number(11)?,
                fields,
            })
        })
        .collect()
}

fn path_ranges(segments: &[Segment<'_>]) -> Vec<std::ops::Range<usize>> {
    let mut ranges = Vec::new();
    let mut start = 0;
    for index in 1..segments.len() {
        if segments[index - 1].deposition.path != segments[index].deposition.path
            || segments[index - 1].fields[..2] != segments[index].fields[..2]
        {
            ranges.push(start..index);
            start = index;
        }
    }
    if start < segments.len() {
        ranges.push(start..segments.len());
    }
    ranges
}

fn exact_path_matches(expected: &[Segment<'_>], actual: &[Segment<'_>]) -> bool {
    expected.len() == actual.len()
        && expected
            .iter()
            .zip(actual)
            .all(|(expected, actual)| segment_matches(expected, actual))
}

fn segment_matches(expected: &Segment<'_>, actual: &Segment<'_>) -> bool {
    expected
        .fields
        .iter()
        .zip(&actual.fields)
        .enumerate()
        .all(|(index, (expected, actual))| {
            index == 11
                && (expected.parse::<f64>().unwrap() - actual.parse::<f64>().unwrap()).abs()
                    <= MAX_EXTRUSION_DIFFERENCE
                || index != 11 && expected == actual
        })
        && expected.deposition.acceleration == actual.deposition.acceleration
        && expected.deposition.fans == actual.deposition.fans
        && feed_matches(expected.deposition.feed, actual.deposition.feed)
}

fn feed_matches(expected: f64, actual: f64) -> bool {
    let absolute = (expected - actual).abs();
    let relative = absolute / expected.abs().max(f64::EPSILON);
    absolute <= MAX_FEED_DIFFERENCE && relative <= MAX_FEED_RELATIVE_DIFFERENCE
}
fn rounded_paths_match(expected: &[Segment<'_>], actual: &[Segment<'_>]) -> bool {
    expected.len() == actual.len()
        && expected
            .iter()
            .zip(actual)
            .all(|(expected, actual)| rounded_segment_matches(expected, actual))
}

fn rounded_segment_matches(expected: &Segment<'_>, actual: &Segment<'_>) -> bool {
    expected
        .fields
        .iter()
        .zip(&actual.fields)
        .enumerate()
        .all(|(index, (expected, actual))| match index {
            3..=8 => {
                (expected.parse::<f64>().unwrap() - actual.parse::<f64>().unwrap()).abs()
                    <= MAX_COORDINATE_ROUNDING_DIFFERENCE_MM
            }
            11 => {
                (expected.parse::<f64>().unwrap() - actual.parse::<f64>().unwrap()).abs()
                    <= MAX_EXTRUSION_DIFFERENCE
            }
            _ => expected == actual,
        })
        && expected.deposition.acceleration == actual.deposition.acceleration
        && expected.deposition.fans == actual.deposition.fans
        && feed_matches(expected.deposition.feed, actual.deposition.feed)
}
fn redistributed_paths_match(expected: &[Segment<'_>], actual: &[Segment<'_>]) -> bool {
    expected.len() == actual.len()
        && expected.iter().zip(actual).all(|(expected, actual)| {
            [0, 1, 2, 9, 10]
                .into_iter()
                .all(|index| expected.fields[index] == actual.fields[index])
                && expected.deposition.acceleration == actual.deposition.acceleration
                && expected.deposition.fans == actual.deposition.fans
                && feed_matches(expected.deposition.feed, actual.deposition.feed)
        })
        && (path_length(expected) - path_length(actual)).abs()
            <= MAX_PATH_SEGMENTATION_DIFFERENCE_MM
        && (path_extrusion(expected) - path_extrusion(actual)).abs()
            <= MAX_PATH_EXTRUSION_DIFFERENCE
        && path_distance(expected, actual) <= MAX_PATH_SEGMENTATION_DIFFERENCE_MM
        && path_distance(actual, expected) <= MAX_PATH_SEGMENTATION_DIFFERENCE_MM
}

fn segmented_paths_match(expected: &[Segment<'_>], actual: &[Segment<'_>]) -> bool {
    !expected.is_empty()
        && expected.len().abs_diff(actual.len()) <= 1
        && uniform_linear_path(expected)
        && uniform_linear_path(actual)
        && path_invariants_match(expected, actual)
        && (path_length(expected) - path_length(actual)).abs()
            <= MAX_PATH_SEGMENTATION_DIFFERENCE_MM
        && (path_extrusion(expected) - path_extrusion(actual)).abs()
            <= MAX_PATH_EXTRUSION_DIFFERENCE
        && path_distance(expected, actual) <= MAX_PATH_SEGMENTATION_DIFFERENCE_MM
        && path_distance(actual, expected) <= MAX_PATH_SEGMENTATION_DIFFERENCE_MM
}

fn uniform_linear_path(path: &[Segment<'_>]) -> bool {
    let first = &path[0];
    path.iter().all(|segment| {
        segment.fields[0] == first.fields[0]
            && segment.fields[1] == first.fields[1]
            && segment.fields[2] == "G1"
            && segment.deposition.acceleration == first.deposition.acceleration
            && segment.deposition.fans == first.deposition.fans
            && feed_matches(first.deposition.feed, segment.deposition.feed)
    })
}

fn path_invariants_match(expected: &[Segment<'_>], actual: &[Segment<'_>]) -> bool {
    expected[0].fields[..3] == actual[0].fields[..3]
        && expected[0].deposition.acceleration == actual[0].deposition.acceleration
        && expected[0].deposition.fans == actual[0].deposition.fans
        && feed_matches(expected[0].deposition.feed, actual[0].deposition.feed)
}

fn path_length(path: &[Segment<'_>]) -> f64 {
    path.iter()
        .map(|segment| distance(segment.start, segment.end))
        .sum()
}

fn path_extrusion(path: &[Segment<'_>]) -> f64 {
    path.iter().map(|segment| segment.extrusion).sum()
}

fn path_distance(from: &[Segment<'_>], to: &[Segment<'_>]) -> f64 {
    from.iter()
        .flat_map(|segment| {
            let midpoint =
                std::array::from_fn(|axis| (segment.start[axis] + segment.end[axis]) * 0.5);
            [segment.start, midpoint, segment.end]
        })
        .map(|point| {
            to.iter()
                .map(|segment| point_segment_distance(point, segment.start, segment.end))
                .fold(f64::INFINITY, f64::min)
        })
        .fold(0.0, f64::max)
}

fn point_segment_distance(point: [f64; 3], start: [f64; 3], end: [f64; 3]) -> f64 {
    let delta = std::array::from_fn::<_, 3, _>(|axis| end[axis] - start[axis]);
    let relative = std::array::from_fn::<_, 3, _>(|axis| point[axis] - start[axis]);
    let squared_length = delta.iter().map(|value| value * value).sum::<f64>();
    let projection = if squared_length > 0.0 {
        relative
            .iter()
            .zip(delta)
            .map(|(relative, delta)| relative * delta)
            .sum::<f64>()
            / squared_length
    } else {
        0.0
    }
    .clamp(0.0, 1.0);
    let nearest = std::array::from_fn(|axis| start[axis] + projection * (end[axis] - start[axis]));
    distance(point, nearest)
}

fn distance(left: [f64; 3], right: [f64; 3]) -> f64 {
    left.iter()
        .zip(right)
        .map(|(left, right)| (left - right).powi(2))
        .sum::<f64>()
        .sqrt()
}

fn exact<T: Debug + PartialEq>(name: &str, expected: &T, actual: &T) -> Result<(), String> {
    if expected == actual {
        Ok(())
    } else {
        Err(format!(
            "{name} differs: expected {expected:?}, actual {actual:?}"
        ))
    }
}

fn exact_layer<T: Debug + PartialEq>(
    layer: usize,
    name: &str,
    expected: &T,
    actual: &T,
) -> Result<(), String> {
    exact(&format!("layer {layer} {name}"), expected, actual)
}
