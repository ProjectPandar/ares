//! The IMEX zone layout (`libslic3r/IMEXZones.cpp:17-377`, `IMEXZones.hpp:25-71` at v2.5.0-beta
//! `345efec64`): where an IDEX/IQEX plate's primary zone, Copy/Mirror zones, carriage collision
//! strips and advisory margin bands sit, in plate-local mm. See
//! `docs/architecture/web-plater-arrange-imex.md`, "A2c1".

use std::collections::{BTreeMap, BTreeSet};

use super::imex::BoundingBoxf;
use crate::DynamicPrintConfig;
use crate::geometry::{Point, Polygon};
use grid::{Cell, Grid, Zones, secondary_cells, zone_roles};
use helpers::{ImexRole, find_imex_mode_active_tools};

mod grid;
mod helpers;
pub(crate) use helpers::imex_hull_violates_zones;

/// `kImexPrimaryMode` (`IMEXHelpers.hpp:27`): the mode with no parallel tools.
pub const IMEX_PRIMARY_MODE: &str = "primary";

/// `BoundingBoxf3`: a box in mm.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BoundingBoxf3 {
    pub min: [f64; 3],
    pub max: [f64; 3],
}

impl BoundingBoxf3 {
    /// `BoundingBox3Base::polygon(true)` (`BoundingBox.cpp:119-134`): the XY rectangle in scaled
    /// coordinates, truncated.
    pub(crate) fn scaled_polygon(&self) -> Polygon {
        let point = |x: f64, y: f64| Point::new((x / 1e-6) as i64, (y / 1e-6) as i64);
        let (min, max) = (self.min, self.max);
        Polygon::new(vec![
            point(min[0], min[1]),
            point(max[0], min[1]),
            point(max[0], max[1]),
            point(min[0], max[1]),
        ])
    }
}

/// `ImexZoneLayout` (`IMEXZones.hpp:25-71`). Every list is empty and `primary_zone_box` unset
/// when no parallel mode is in effect.
#[derive(Clone, Debug, PartialEq)]
pub struct ImexZoneLayout {
    /// The active mode's Primary tool; -1 when the layout is empty.
    pub primary_head: i32,
    /// Physical head -> the XY centre of its zone, Primary included; a Span tool has none.
    pub head_zone_centers: BTreeMap<i32, [f64; 2]>,
    /// The primary tool's zone, where objects go.
    pub primary_zone_box: Option<BoundingBoxf>,
    pub copy_zones: Vec<BoundingBoxf>,
    pub mirror_zones: Vec<BoundingBoxf>,
    /// `copy_zones` then `mirror_zones` as full-height boxes, for the placement check.
    pub secondary_zone_boxes: Vec<BoundingBoxf3>,
    /// Carriage danger strips inside the primary zone, right / left / top / bottom.
    pub collision_zones: Vec<BoundingBoxf3>,
    /// Advisory bands inside each strip, when `imex_carriage_margin` > 0.
    pub margin_bands: Vec<BoundingBoxf>,
}

impl Default for ImexZoneLayout {
    fn default() -> Self {
        Self {
            primary_head: -1,
            head_zone_centers: BTreeMap::new(),
            primary_zone_box: None,
            copy_zones: Vec::new(),
            mirror_zones: Vec::new(),
            secondary_zone_boxes: Vec::new(),
            collision_zones: Vec::new(),
            margin_bands: Vec::new(),
        }
    }
}

/// A full-height box (`z` -1 to 1e4) over `b`.
fn full_height(b: &BoundingBoxf) -> BoundingBoxf3 {
    BoundingBoxf3 {
        min: [b.min[0], b.min[1], -1.0],
        max: [b.max[0], b.max[1], 1e4],
    }
}

/// `compute_imex_zone_layout` (`IMEXZones.cpp:17-377`): the layout of one plate from the printer
/// config, the plate's own mode (which wins unless it is `IMEX_PRIMARY_MODE`), the process
/// preset's `imex_parallel_mode` and the plate's XY extents.
pub fn compute_imex_zone_layout(
    printer_cfg: &DynamicPrintConfig,
    plate_mode: &str,
    process_mode: &str,
    bed_extents: BoundingBoxf,
) -> ImexZoneLayout {
    let mut out = ImexZoneLayout::default();
    if !printer_cfg.option("is_imex").bool() {
        return out;
    }
    let active_mode = if plate_mode == IMEX_PRIMARY_MODE && !process_mode.is_empty() {
        process_mode
    } else {
        plate_mode
    };
    if active_mode == IMEX_PRIMARY_MODE || active_mode.is_empty() {
        return out;
    }
    let grid = Grid::read(printer_cfg);
    if grid.n_cols == 1 && grid.n_rows == 1 {
        return out;
    }
    let active_tools = find_imex_mode_active_tools(printer_cfg, active_mode);
    let roles = zone_roles(&active_tools, grid.n_rows * grid.n_cols);
    // No Primary on the grid: nothing to measure the zones from (`:101-122`).
    let Some((&primary, _)) = roles.iter().find(|(_, role)| **role == ImexRole::Primary) else {
        return out;
    };
    out.primary_head = primary;
    let (pri_col, pri_row) = grid.tool_to_phys(primary);
    let (copy_cells, mirror_cells) = secondary_cells(&roles, (&grid, &active_tools), pri_col);
    let zones = Zones::new(&roles, &grid, bed_extents);
    let pri = (zones.col(pri_col), zones.row(pri_row));
    // Every zone-owning head's centre (`:196-205`).
    for &idx in roles.keys() {
        let (col, row) = grid.tool_to_phys(idx);
        let (ck, rk) = (f64::from(zones.col(col)), f64::from(zones.row(row)));
        out.head_zone_centers.insert(
            idx,
            [
                zones.min[0] + (ck + 0.5) * zones.size[0],
                zones.min[1] + (rk + 0.5) * zones.size[1],
            ],
        );
    }
    // The separation axes (`:207-212`).
    let separation = copy_cells
        .iter()
        .chain(&mirror_cells)
        .fold((false, false), |(row, col), &(sc, sr)| {
            (row || sr != pri_row, col || sc != pri_col)
        });
    out.copy_zones = zones.make_boxes(&copy_cells, separation);
    out.mirror_zones = zones.make_boxes(&mirror_cells, separation);
    out.secondary_zone_boxes = out
        .copy_zones
        .iter()
        .chain(&out.mirror_zones)
        .map(full_height)
        .collect();
    add_strips(
        &mut out,
        (printer_cfg, &zones),
        (&mirror_cells, separation),
        (pri, pri_col),
    );
    out
}

/// The primary zone, the carriage collision strips and the margin bands (`:276-374`). Only a
/// Mirror zone adjacent to the primary's in zone indices, on the primary's row (X strips) or
/// physical column (Y strips), raises a strip; each strip is the axis's nozzle clearance wide.
fn add_strips(
    out: &mut ImexZoneLayout,
    (cfg, zones): (&DynamicPrintConfig, &Zones),
    (mirror_cells, (row_sep, col_sep)): (&BTreeSet<Cell>, (bool, bool)),
    ((pri_col_k, pri_row_k), pri_col): ((i32, i32), i32),
) {
    let carriage_w = cfg.option("imex_nozzle_clearance_x").float();
    let carriage_h = cfg.option("imex_nozzle_clearance_y").float();
    let margin = cfg.option("imex_carriage_margin").float();
    let pz = [
        if col_sep {
            zones.edge(0, pri_col_k)
        } else {
            zones.min[0]
        },
        if row_sep {
            zones.edge(1, pri_row_k)
        } else {
            zones.min[1]
        },
        if col_sep {
            zones.edge(0, pri_col_k + 1)
        } else {
            zones.max[0]
        },
        if row_sep {
            zones.edge(1, pri_row_k + 1)
        } else {
            zones.max[1]
        },
    ];
    out.primary_zone_box = Some(BoundingBoxf {
        min: [pz[0], pz[1]],
        max: [pz[2], pz[3]],
    });
    let (mut right, mut left, mut top, mut bottom) = (false, false, false, false);
    for &(sc, sr) in mirror_cells {
        let (Some(&zc), Some(&zr)) = (zones.col_to_zone.get(&sc), zones.row_to_zone.get(&sr))
        else {
            continue;
        };
        right |= zr == pri_row_k && zc == pri_col_k + 1;
        left |= zr == pri_row_k && zc == pri_col_k - 1;
        top |= zr == pri_row_k + 1 && sc == pri_col;
        bottom |= zr == pri_row_k - 1 && sc == pri_col;
    }
    let mut strip = |x: [f64; 2], y: [f64; 2], band: Option<([f64; 2], [f64; 2])>| {
        out.collision_zones.push(BoundingBoxf3 {
            min: [x[0], y[0], -1.0],
            max: [x[1], y[1], 1e4],
        });
        if let Some((bx, by)) = band.filter(|_| margin > 0.0) {
            out.margin_bands.push(BoundingBoxf {
                min: [bx[0], by[0]],
                max: [bx[1], by[1]],
            });
        }
    };
    let y = [pz[1], pz[3]];
    let x = [pz[0], pz[2]];
    if carriage_w > 0.0 {
        if right {
            let bnd = zones.edge(0, pri_col_k + 1);
            let inner = bnd - carriage_w;
            strip([inner, bnd], y, Some(([inner - margin, inner], y)));
        }
        if left {
            let bnd = zones.edge(0, pri_col_k);
            let inner = bnd + carriage_w;
            strip([bnd, inner], y, Some(([inner, inner + margin], y)));
        }
    }
    if carriage_h > 0.0 {
        if top {
            let bnd = zones.edge(1, pri_row_k + 1);
            let inner = bnd - carriage_h;
            strip(x, [inner, bnd], Some((x, [inner - margin, inner])));
        }
        if bottom {
            let bnd = zones.edge(1, pri_row_k);
            let inner = bnd + carriage_h;
            strip(x, [bnd, inner], Some((x, [inner, inner + margin])));
        }
    }
}

/// `compute_imex_slice_offset` (`IMEXHelpers.cpp:634-643`): the primary zone's centre when the
/// firmware places the secondaries and a parallel mode is active, else zero.
pub fn compute_imex_slice_offset(
    firmware_managed: bool,
    parallel_mode: &str,
    primary_zone_box: Option<BoundingBoxf>,
) -> [f64; 2] {
    match primary_zone_box {
        Some(zone)
            if firmware_managed
                && !parallel_mode.is_empty()
                && parallel_mode != IMEX_PRIMARY_MODE =>
        {
            [
                0.5 * (zone.min[0] + zone.max[0]),
                0.5 * (zone.min[1] + zone.max[1]),
            ]
        }
        _ => [0.0, 0.0],
    }
}
