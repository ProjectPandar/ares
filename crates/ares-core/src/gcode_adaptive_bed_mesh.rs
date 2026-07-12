use crate::gcode_format::format_decimal;
use crate::gcode_placeholders::MachineStartPlaceholderContext;
use crate::options::GCodeFlavor;
use crate::{LayerPrintPaths, Point2, SliceError, SliceOptions};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct AdaptiveBedMeshPlaceholders {
    min: [String; 2],
    max: [String; 2],
    probe_count: [u32; 2],
    algorithm: &'static str,
}

impl AdaptiveBedMeshPlaceholders {
    pub(crate) fn min_list(&self) -> String {
        self.min.join(",")
    }

    pub(crate) fn max_list(&self) -> String {
        self.max.join(",")
    }

    pub(crate) fn min_x(&self) -> &str {
        &self.min[0]
    }

    pub(crate) fn min_y(&self) -> &str {
        &self.min[1]
    }

    pub(crate) fn max_x(&self) -> &str {
        &self.max[0]
    }

    pub(crate) fn max_y(&self) -> &str {
        &self.max[1]
    }

    pub(crate) fn probe_count_list(&self) -> String {
        format!("{},{}", self.probe_count[0], self.probe_count[1])
    }

    pub(crate) const fn probe_count_x(&self) -> u32 {
        self.probe_count[0]
    }

    pub(crate) const fn probe_count_y(&self) -> u32 {
        self.probe_count[1]
    }

    pub(crate) const fn algorithm(&self) -> &'static str {
        self.algorithm
    }
}

pub(crate) fn placeholders(
    options: &SliceOptions,
    gcode_flavor: GCodeFlavor,
    layer_print_paths: &[LayerPrintPaths],
) -> Result<AdaptiveBedMeshPlaceholders, SliceError> {
    let config = options.adaptive_bed_mesh_options()?;
    let bounds = first_layer_bounds(layer_print_paths)
        .unwrap_or_else(|| Bounds::new(config.bed_mesh_min(), config.bed_mesh_max()));
    let margin = config.adaptive_bed_mesh_margin();
    let min = Point2::new(
        config.bed_mesh_min().x().max(bounds.min.x() - margin),
        config.bed_mesh_min().y().max(bounds.min.y() - margin),
    );
    let max = Point2::new(
        config.bed_mesh_max().x().min(bounds.max.x() + margin),
        config.bed_mesh_max().y().min(bounds.max.y() + margin),
    );
    let probe_distance = config.bed_mesh_probe_distance();
    let mut probe_count = [
        probe_count(max.x() - min.x(), probe_distance.x()),
        probe_count(max.y() - min.y(), probe_distance.y()),
    ];
    let algorithm = if probe_count[0] * probe_count[1] <= 6 {
        "lagrange"
    } else {
        if gcode_flavor == GCodeFlavor::Klipper {
            probe_count[0] = probe_count[0].max(4);
            probe_count[1] = probe_count[1].max(4);
        }
        "bicubic"
    };
    Ok(AdaptiveBedMeshPlaceholders {
        min: [format_decimal(min.x()), format_decimal(min.y())],
        max: [format_decimal(max.x()), format_decimal(max.y())],
        probe_count,
        algorithm,
    })
}

pub(crate) fn machine_start_gcode(
    options: &SliceOptions,
    gcode_flavor: GCodeFlavor,
    layer_print_paths: &[LayerPrintPaths],
    context: MachineStartPlaceholderContext,
) -> Result<String, SliceError> {
    let adaptive_bed_mesh = placeholders(options, gcode_flavor, layer_print_paths)?;
    let first_layer_print =
        crate::gcode_first_layer_print_placeholders::placeholders(layer_print_paths);
    crate::gcode_placeholders::machine_start_gcode(
        options,
        Some(&adaptive_bed_mesh),
        Some(&first_layer_print),
        context,
    )
}

fn first_layer_bounds(layer_print_paths: &[LayerPrintPaths]) -> Option<Bounds> {
    layer_print_paths
        .first()
        .and_then(|layer| points_bounds(layer.paths().iter().flat_map(|path| path.points())))
}

fn points_bounds<'a>(points: impl IntoIterator<Item = &'a Point2>) -> Option<Bounds> {
    let mut points = points.into_iter();
    let first = *points.next()?;
    let mut bounds = Bounds::new(first, first);
    for point in points {
        bounds.include(*point);
    }
    Some(bounds)
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Bounds {
    min: Point2,
    max: Point2,
}

impl Bounds {
    const fn new(min: Point2, max: Point2) -> Self {
        Self { min, max }
    }

    fn include(&mut self, point: Point2) {
        self.min = Point2::new(self.min.x().min(point.x()), self.min.y().min(point.y()));
        self.max = Point2::new(self.max.x().max(point.x()), self.max.y().max(point.y()));
    }
}

fn probe_count(size: f64, distance: f64) -> u32 {
    let distance = distance.max(1.0);
    ((size / distance).ceil() as u32 + 1).max(3)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SliceOptions;

    #[test]
    fn empty_first_layer_paths_fall_back_to_configured_mesh_bounds() {
        let placeholders =
            placeholders(&SliceOptions::default(), GCodeFlavor::MarlinLegacy, &[]).unwrap();

        assert_eq!(placeholders.min_list(), "-99999,-99999");
        assert_eq!(placeholders.max_list(), "99999,99999");
        assert_eq!(placeholders.probe_count_list(), "4001,4001");
        assert_eq!(placeholders.algorithm(), "bicubic");
    }
}
