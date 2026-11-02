//! The tool grid, the zone-owning heads and the zones sized by the active tools, for
//! `compute_imex_zone_layout` (`libslic3r/IMEXZones.cpp:37-240` at v2.5.0-beta `345efec64`).

use std::collections::{BTreeMap, BTreeSet};

use super::helpers::{
    ImexRole, group_imex_active_tools_by_gantry, imex_primary_tool_for_mode,
    parse_imex_active_tools,
};
use crate::DynamicPrintConfig;
use crate::arrangement::imex::BoundingBoxf;

pub(super) type Cell = (i32, i32);

/// The tool grid (`:37-49`): `imex_tools_per_gantry` columns by `imex_gantry_count` rows, flipped
/// so column and row 0 are min-X and min-Y whatever corner `imex_tool_layout` puts T0 in.
pub(super) struct Grid {
    pub(super) n_cols: i32,
    pub(super) n_rows: i32,
    flip_x: bool,
    flip_y: bool,
}

impl Grid {
    pub(super) fn read(cfg: &DynamicPrintConfig) -> Self {
        // `imex_cfg_enum<ImexToolLayout>` (`IMEXHelpers.hpp:386-421`), keys `PrintConfig.cpp:751-756`.
        let layout = cfg.option("imex_tool_layout");
        let layout = layout.key();
        Self {
            n_cols: (cfg.option("imex_tools_per_gantry").int() as i32).max(1),
            n_rows: (cfg.option("imex_gantry_count").int() as i32).max(1),
            flip_x: matches!(layout, "front-right" | "rear-right"),
            flip_y: matches!(layout, "rear-left" | "rear-right"),
        }
    }

    /// `tool_to_phys` (`:46-50`): a tool index to its (column, row).
    pub(super) fn tool_to_phys(&self, idx: i32) -> Cell {
        let (raw_col, raw_row) = (idx % self.n_cols, idx / self.n_cols);
        (
            if self.flip_x {
                self.n_cols - 1 - raw_col
            } else {
                raw_col
            },
            if self.flip_y {
                self.n_rows - 1 - raw_row
            } else {
                raw_row
            },
        )
    }
}

/// The heads owning a zone cell (`:76-97`): the Primary on the grid, then each Copy and Mirror;
/// a Span head shares the primary's zone and an extra Primary is ignored.
pub(super) fn zone_roles(active_tools: &str, cells: i32) -> BTreeMap<i32, ImexRole> {
    let mut roles = BTreeMap::new();
    let primary = imex_primary_tool_for_mode(active_tools);
    if (0..cells).contains(&primary) {
        roles.insert(primary, ImexRole::Primary);
    }
    for (phys, role) in parse_imex_active_tools(active_tools) {
        if !(0..cells).contains(&phys) || phys == primary {
            continue;
        }
        match role {
            ImexRole::Copy | ImexRole::Mirror => {
                roles.insert(phys, role);
            }
            ImexRole::Primary | ImexRole::Span => {}
        }
    }
    roles
}

/// The Copy and Mirror cells (`:128-160`): an aggregated gantry contributes its representative
/// alone, pinned to the primary's column.
pub(super) fn secondary_cells(
    roles: &BTreeMap<i32, ImexRole>,
    (grid, active_tools): (&Grid, &str),
    pri_col: i32,
) -> (BTreeSet<Cell>, BTreeSet<Cell>) {
    let groups = group_imex_active_tools_by_gantry(active_tools, grid.n_cols);
    let (mut copy_cells, mut mirror_cells) = (BTreeSet::new(), BTreeSet::new());
    for (&idx, &role) in roles {
        let group = groups
            .iter()
            .find(|group| group.gantry_index == idx / grid.n_cols);
        let cell = match group {
            Some(group) if group.aggregate => {
                if idx != group.representative_phys {
                    continue;
                }
                (pri_col, grid.tool_to_phys(idx).1)
            }
            _ => grid.tool_to_phys(idx),
        };
        match role {
            ImexRole::Copy => {
                copy_cells.insert(cell);
            }
            ImexRole::Mirror => {
                mirror_cells.insert(cell);
            }
            ImexRole::Primary | ImexRole::Span => {}
        }
    }
    (copy_cells, mirror_cells)
}

/// The zones sized by the active tools per axis (`:166-190`): sorted active columns and rows map
/// onto zone indices, so an inactive tool donates its share.
pub(super) struct Zones {
    pub(super) min: [f64; 2],
    pub(super) max: [f64; 2],
    pub(super) size: [f64; 2],
    pub(super) col_to_zone: BTreeMap<i32, i32>,
    pub(super) row_to_zone: BTreeMap<i32, i32>,
}

impl Zones {
    pub(super) fn new(roles: &BTreeMap<i32, ImexRole>, grid: &Grid, bed: BoundingBoxf) -> Self {
        let cells: Vec<Cell> = roles.keys().map(|&idx| grid.tool_to_phys(idx)).collect();
        let index =
            |values: BTreeSet<i32>| -> BTreeMap<i32, i32> { values.into_iter().zip(0..).collect() };
        let col_to_zone = index(cells.iter().map(|cell| cell.0).collect());
        let row_to_zone = index(cells.iter().map(|cell| cell.1).collect());
        let active = [col_to_zone.len().max(1), row_to_zone.len().max(1)];
        Self {
            min: bed.min,
            max: bed.max,
            size: std::array::from_fn(|axis| (bed.max[axis] - bed.min[axis]) / active[axis] as f64),
            col_to_zone,
            row_to_zone,
        }
    }

    pub(super) fn col(&self, col: i32) -> i32 {
        self.col_to_zone.get(&col).copied().unwrap_or(0)
    }

    pub(super) fn row(&self, row: i32) -> i32 {
        self.row_to_zone.get(&row).copied().unwrap_or(0)
    }

    /// The edge `k` zones from the minimum along `axis`.
    pub(super) fn edge(&self, axis: usize, k: i32) -> f64 {
        self.min[axis] + f64::from(k) * self.size[axis]
    }

    /// `make_boxes` (`:212-240`): full-width row strips when the secondaries differ from the
    /// primary by row only, full-height column strips by column only, else one box per cell.
    pub(super) fn make_boxes(
        &self,
        cells: &BTreeSet<Cell>,
        (row_sep, col_sep): (bool, bool),
    ) -> Vec<BoundingBoxf> {
        let rect = |x: [f64; 2], y: [f64; 2]| BoundingBoxf {
            min: [x[0], y[0]],
            max: [x[1], y[1]],
        };
        if row_sep && !col_sep {
            let rows: BTreeSet<i32> = cells.iter().map(|cell| cell.1).collect();
            rows.into_iter()
                .map(|row| {
                    let k = self.row(row);
                    rect(
                        [self.min[0], self.max[0]],
                        [self.edge(1, k), self.edge(1, k + 1)],
                    )
                })
                .collect()
        } else if col_sep && !row_sep {
            let cols: BTreeSet<i32> = cells.iter().map(|cell| cell.0).collect();
            cols.into_iter()
                .map(|col| {
                    let k = self.col(col);
                    rect(
                        [self.edge(0, k), self.edge(0, k + 1)],
                        [self.min[1], self.max[1]],
                    )
                })
                .collect()
        } else {
            cells
                .iter()
                .map(|&(col, row)| {
                    let (ck, rk) = (self.col(col), self.row(row));
                    rect(
                        [self.edge(0, ck), self.edge(0, ck + 1)],
                        [self.edge(1, rk), self.edge(1, rk + 1)],
                    )
                })
                .collect()
        }
    }
}
