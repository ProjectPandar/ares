//! The bed-exclusion regions of a print config at v2.5.0-beta (`345efec64`): `BedExcludeRegion`
//! (`PrintConfig.hpp:2545-2558`) and their resolution (`PrintConfig.cpp:13887-14369`). See
//! `docs/architecture/web-plater-arrange-imex.md`, "A2b". The belt projection of material
//! keep-outs (`project_bed_material_keepouts`, `:14089-14109`) is deferred with belts.

use crate::geometry::{Point, Polygon};
use crate::{BedExcludeVolumeMode, DynamicPrintConfig};

mod intersection;

pub use intersection::model_part_intersects_region;

/// `BedExcludeRegion::Purpose`.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Purpose {
    #[default]
    MaterialKeepout,
    CollisionVolume,
}

/// `BedExcludeRegion` (`PrintConfig.hpp:2545-2558`): a bed polygon kept clear between `z_min` and
/// `z_max` mm.
#[derive(Clone, Debug, PartialEq)]
pub struct BedExcludeRegion {
    pub polygon: Polygon,
    pub z_min: f64,
    pub z_max: f64,
    pub purpose: Purpose,
    pub has_z_range: bool,
}

impl BedExcludeRegion {
    pub fn is_collision_volume(&self) -> bool {
        self.purpose == Purpose::CollisionVolume
    }
}

/// `scale_(v)` into `Point(double, double)`, which rounds.
fn scaled_point(x: f64, y: f64) -> Point {
    Point::new((x / 1e-6).round() as i64, (y / 1e-6).round() as i64)
}

/// `Polygon::make_counter_clockwise`.
fn counter_clockwise(mut polygon: Polygon) -> Polygon {
    if !polygon.is_counter_clockwise() {
        polygon.reverse();
    }
    polygon
}

/// `parse_double_token` (`:13892-13902`): the whole trimmed token as a finite number. It reads
/// through `string_to_double_decimal_point`, `fast_float::from_chars` (`LocalesUtils.cpp:86-93`),
/// which takes no leading '+'.
fn parse_double_token(text: &str) -> Option<f64> {
    let token = text.trim();
    if token.starts_with('+') {
        return None;
    }
    token.parse::<f64>().ok().filter(|value| value.is_finite())
}

/// The z range a region specification builds up.
struct ZRange {
    min: f64,
    max: f64,
    has: bool,
}

/// `parse_z_range_token` (`:13904-13938`). A parsed range is stored even when it clamps to a plane,
/// which is then not a range.
fn parse_z_range_token(token: &str, printable_height: f64, z: &mut ZRange) -> bool {
    let token = token.trim();
    let Some(range_pos) = token.find("..") else {
        return false;
    };
    let (min_token, max_token) = (token[..range_pos].trim(), token[range_pos + 2..].trim());
    let parse_or = |text: &str, default: f64| {
        if text.is_empty() {
            Some(default)
        } else {
            parse_double_token(text)
        }
    };
    let (Some(parsed_min), Some(parsed_max)) = (
        parse_or(min_token, 0.0),
        parse_or(max_token, printable_height),
    ) else {
        return false;
    };
    z.min = parsed_min.clamp(0.0, printable_height);
    z.max = parsed_max.clamp(0.0, printable_height);
    z.has = true;
    z.min < z.max
}

/// `parse_exclude_polygon_points` (`:13940-13971`).
fn parse_exclude_polygon_points(points: &str) -> Option<Polygon> {
    let mut parsed = Vec::new();
    for token in points.split(',') {
        let token = token.trim();
        let (x, y) = token.split_once('x').filter(|(_, y)| !y.contains('x'))?;
        parsed.push(scaled_point(parse_double_token(x)?, parse_double_token(y)?));
    }
    if parsed.len() < 3 {
        return None;
    }
    let polygon = Polygon::new(parsed);
    (polygon.area() != 0.0).then(|| counter_clockwise(polygon))
}

/// `append_legacy_bed_exclude_regions` (`:13973-13989`).
fn append_legacy_regions(out: &mut Vec<BedExcludeRegion>, points: &[[f64; 2]], height: f64) {
    if points.len() < 3 {
        return;
    }
    let polygon = Polygon::new(points.iter().map(|&[x, y]| scaled_point(x, y)).collect());
    out.push(BedExcludeRegion {
        polygon: counter_clockwise(polygon),
        z_min: 0.0,
        z_max: height,
        purpose: Purpose::MaterialKeepout,
        has_z_range: false,
    });
}

/// `append_bed_exclusion_volume_regions` (`:13991-14054`).
fn append_volume_regions(out: &mut Vec<BedExcludeRegion>, specs: &str, printable_height: f64) {
    for spec in specs
        .split(['|', '\n'])
        .map(str::trim)
        .filter(|spec| !spec.is_empty())
    {
        if let Some(region) = volume_region(spec, printable_height) {
            out.push(region);
        }
    }
}

/// One region specification of `append_bed_exclusion_volume_regions` (`:14004-14052`).
fn volume_region(spec: &str, printable_height: f64) -> Option<BedExcludeRegion> {
    let mut z = ZRange {
        min: 0.0,
        max: printable_height,
        has: false,
    };
    let mut points_field = "";
    for field in spec
        .split(';')
        .map(str::trim)
        .filter(|field| !field.is_empty())
    {
        if parse_z_range_token(field, printable_height, &mut z) {
            continue;
        }
        if field.contains("..") && !field.contains('x') {
            return None;
        }
        points_field = strip_prefix_ignore_case(field, "points=")
            .or_else(|| strip_prefix_ignore_case(field, "polygon="))
            .unwrap_or(field);
    }
    if !z.has {
        (z.min, z.max) = (0.0, printable_height);
    }
    if z.min >= z.max {
        return None;
    }
    Some(BedExcludeRegion {
        polygon: parse_exclude_polygon_points(points_field)?,
        z_min: z.min,
        z_max: z.max,
        purpose: Purpose::CollisionVolume,
        has_z_range: z.has,
    })
}

/// `boost::algorithm::istarts_with` then the rest of `field`.
fn strip_prefix_ignore_case<'a>(field: &'a str, prefix: &str) -> Option<&'a str> {
    let head = field.get(..prefix.len())?;
    head.eq_ignore_ascii_case(prefix)
        .then(|| &field[prefix.len()..])
}

/// `has_nonempty_string` (`:14056-14059`).
fn has_nonempty_string(value: &str) -> bool {
    value
        .chars()
        .any(|c| !c.is_ascii_whitespace() && c != '\u{0b}')
}

/// `collision_volumes_configured` (`:14061-14069`).
fn collision_volumes_configured(
    shared: &str,
    mode: BedExcludeVolumeMode,
    per_extruder: &[String],
) -> bool {
    if mode == BedExcludeVolumeMode::PerExtruder {
        per_extruder.iter().any(|spec| has_nonempty_string(spec))
    } else {
        has_nonempty_string(shared)
    }
}

/// The keys the resolution reads, as `get_bed_excluded_regions_by_extruder` takes them
/// (`:14213-14228`).
struct Inputs {
    printable_height: f64,
    legacy_points: Vec<[f64; 2]>,
    shared: String,
    mode: BedExcludeVolumeMode,
    per_extruder: Vec<String>,
    offsets: Vec<[f64; 2]>,
    nozzle_count: usize,
    reference: usize,
}

impl Inputs {
    fn read(config: &DynamicPrintConfig) -> Self {
        let mode = match config.option("bed_exclude_volume_mode").key() {
            "toolhead_offset" => BedExcludeVolumeMode::ToolheadOffset,
            "per_extruder" => BedExcludeVolumeMode::PerExtruder,
            _ => BedExcludeVolumeMode::Shared,
        };
        let master = config.option("master_extruder_id").int();
        Self {
            printable_height: config.option("printable_height").float(),
            legacy_points: config.option("bed_exclude_area").points(),
            shared: config.option("bed_exclude_volumes").text().to_owned(),
            mode,
            per_extruder: config
                .option("extruder_bed_exclude_volumes")
                .strings()
                .to_vec(),
            offsets: config.option("extruder_offset").points(),
            nozzle_count: config.option("nozzle_diameter").floats().len(),
            reference: if master > 0 { (master - 1) as usize } else { 0 },
        }
    }

    fn collision_volumes_configured(&self) -> bool {
        collision_volumes_configured(&self.shared, self.mode, &self.per_extruder)
    }
}

/// `resolve_bed_excluded_regions` (`:14111-14148`).
fn resolve(
    master: &[BedExcludeRegion],
    mode: BedExcludeVolumeMode,
    inputs: &Inputs,
    extruder_id: usize,
) -> Vec<BedExcludeRegion> {
    let mut regions = master.to_vec();
    match mode {
        BedExcludeVolumeMode::Shared => {}
        BedExcludeVolumeMode::PerExtruder => {
            if let Some(spec) = inputs.per_extruder.get(extruder_id) {
                append_volume_regions(&mut regions, spec, inputs.printable_height.max(0.0));
            }
        }
        BedExcludeVolumeMode::ToolheadOffset => {
            let offsets = &inputs.offsets;
            if extruder_id >= offsets.len() {
                return regions;
            }
            // A carriage obstacle described through the reference nozzle appears in another
            // nozzle's model space translated by (other_offset - reference_offset).
            let reference = offsets[inputs.reference.min(offsets.len() - 1)];
            let delta = scaled_point(
                offsets[extruder_id][0] - reference[0],
                offsets[extruder_id][1] - reference[1],
            );
            for region in regions
                .iter_mut()
                .filter(|region| region.is_collision_volume())
            {
                region.polygon = Polygon::new(
                    region
                        .polygon
                        .points()
                        .iter()
                        .map(|p| Point::new(p.x() + delta.x(), p.y() + delta.y()))
                        .collect(),
                );
            }
        }
    }
    regions
}

/// `get_bed_excluded_regions_by_extruder(const DynamicPrintConfig&)` (`PrintConfig.cpp:14211-14243`).
pub fn get_bed_excluded_regions_by_extruder(
    config: &DynamicPrintConfig,
) -> Vec<Vec<BedExcludeRegion>> {
    let inputs = Inputs::read(config);
    let active_mode = if inputs.collision_volumes_configured() {
        inputs.mode
    } else {
        BedExcludeVolumeMode::Shared
    };
    // `parse_master_bed_excluded_regions` (`:14071-14087`): legacy areas stay a shared keep-out
    // even with collision volumes configured.
    let height = inputs.printable_height.max(0.0);
    let mut master = Vec::new();
    append_legacy_regions(&mut master, &inputs.legacy_points, height);
    if inputs.collision_volumes_configured() && active_mode != BedExcludeVolumeMode::PerExtruder {
        append_volume_regions(&mut master, &inputs.shared, height);
    }
    // `bed_exclusion_extruder_count` (`:14150-14156`).
    let count = inputs
        .nozzle_count
        .max(inputs.offsets.len())
        .max(inputs.per_extruder.len())
        .max(1);
    (0..count)
        .map(|extruder_id| resolve(&master, active_mode, &inputs, extruder_id))
        .collect()
}

/// `get_bed_excluded_regions(const DynamicPrintConfig&)` (`PrintConfig.cpp:14283-14286`) with
/// `flatten_bed_excluded_regions` (`:14158-14185`).
pub fn get_bed_excluded_regions(config: &DynamicPrintConfig) -> Vec<BedExcludeRegion> {
    let mut groups = get_bed_excluded_regions_by_extruder(config);
    if groups.is_empty() {
        return Vec::new();
    }
    if active_bed_exclude_volume_mode(config) == BedExcludeVolumeMode::Shared {
        return groups.swap_remove(0);
    }
    // A shared legacy area is in every nozzle's group; it is flattened once.
    let mut material_keepout_added = false;
    let mut flattened = Vec::new();
    for region in groups.into_iter().flatten() {
        if !region.is_collision_volume() {
            if material_keepout_added {
                continue;
            }
            material_keepout_added = true;
        }
        flattened.push(region);
    }
    flattened
}

/// `is_valid_bed_exclude_volumes_string` (`PrintConfig.cpp:14298-14317`).
pub fn is_valid_bed_exclude_volumes_string(value: &str, printable_height: f64) -> bool {
    if value.is_empty() {
        return true;
    }
    let mut found_nonempty = false;
    for spec in value
        .split(['|', '\n'])
        .map(str::trim)
        .filter(|spec| !spec.is_empty())
    {
        found_nonempty = true;
        let mut parsed = Vec::new();
        append_volume_regions(&mut parsed, spec, printable_height.max(0.0));
        if parsed.len() != 1 {
            return false;
        }
    }
    found_nonempty
}

/// `has_bed_exclude_volumes(const DynamicPrintConfig&)` (`PrintConfig.cpp:14346-14355`).
pub fn has_bed_exclude_volumes(config: &DynamicPrintConfig) -> bool {
    Inputs::read(config).collision_volumes_configured()
}

/// `active_bed_exclude_volume_mode(const DynamicPrintConfig&)` (`PrintConfig.cpp:14364-14369`).
pub fn active_bed_exclude_volume_mode(config: &DynamicPrintConfig) -> BedExcludeVolumeMode {
    let inputs = Inputs::read(config);
    if inputs.collision_volumes_configured() {
        inputs.mode
    } else {
        BedExcludeVolumeMode::Shared
    }
}
