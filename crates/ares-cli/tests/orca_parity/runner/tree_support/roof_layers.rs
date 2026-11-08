//! Public 3MF replay of the `TreeSupport.cpp` roof counter (`:3122,:3207,:3368,:3770`)
//! and the first-roof fill angle (`:1728`).
//!
//! Every value derives from the stream at runtime: a layer belongs to the roof when it
//! extrudes under `;TYPE:Support interface`, a first roof carries a closed interface
//! wall (`:1736`), and the fill axis is the axis with the longest straight extrusion.

const INTERFACE: &str = "Support interface";
/// Longest gap, in mm, between the ends of a wall clipped by the default `seam_gap`;
/// fill paths that merely end near their start stay well above it.
const CLOSED_WALL_GAP: f64 = 0.1;
const CLOSED_WALL_SEGMENTS: usize = 3;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FillAxis {
    Horizontal,
    Vertical,
}

#[derive(Debug, PartialEq, Eq)]
struct RoofLayer {
    z: String,
    closed_walls: usize,
    axis: Option<FillAxis>,
}

#[derive(Default)]
struct Walk {
    at: (f64, f64),
    path_start: (f64, f64),
    path_segments: usize,
    closed_walls: usize,
    horizontal: f64,
    vertical: f64,
    extruded: bool,
}

impl Walk {
    fn end_path(&mut self) {
        let gap = (self.at.0 - self.path_start.0).hypot(self.at.1 - self.path_start.1);
        if self.path_segments >= CLOSED_WALL_SEGMENTS && gap <= CLOSED_WALL_GAP {
            self.closed_walls += 1;
        }
        self.path_start = self.at;
        self.path_segments = 0;
    }

    fn line(&mut self, line: &str, interface: bool) {
        if !line.starts_with("G1 ") {
            return;
        }
        let word = |axis: char| {
            line.split_whitespace()
                .find_map(|word| word.strip_prefix(axis)?.parse::<f64>().ok())
        };
        let extrude = word('E');
        if extrude.is_some_and(|e| e < 0.0) {
            self.end_path();
            return;
        }
        let (Some(x), Some(y)) = (word('X'), word('Y')) else {
            return;
        };
        let from = std::mem::replace(&mut self.at, (x, y));
        if extrude.is_none() {
            let to = self.at;
            self.at = from;
            self.end_path();
            self.at = to;
            self.path_start = to;
            return;
        }
        if !interface {
            return;
        }
        self.extruded = true;
        self.path_segments += 1;
        if y == from.1 {
            self.horizontal += (x - from.0).abs();
        } else if x == from.0 {
            self.vertical += (y - from.1).abs();
        }
    }
}

/// Per-layer roof signature of `stream`, for the layers that extrude the interface role.
fn roof_layers(stream: &str) -> Vec<RoofLayer> {
    let mut role = "";
    let mut walk = Walk::default();
    let mut layers = Vec::new();
    for block in stream.split("\n;Z:").skip(1) {
        let (z, body) = block.split_once('\n').unwrap();
        for line in body.lines() {
            if let Some(value) = line.strip_prefix(";TYPE:") {
                walk.end_path();
                role = value;
            }
            walk.line(line, role == INTERFACE);
        }
        walk.end_path();
        if walk.extruded {
            layers.push(RoofLayer {
                z: z.to_owned(),
                closed_walls: walk.closed_walls,
                axis: (walk.horizontal != walk.vertical).then_some(
                    if walk.horizontal > walk.vertical {
                        FillAxis::Horizontal
                    } else {
                        FillAxis::Vertical
                    },
                ),
            });
        }
        walk = Walk {
            at: walk.at,
            path_start: walk.at,
            ..Walk::default()
        };
    }
    layers
}

/// Rewrites `from` markers to `to`, only in the first layer whose text contains `from`.
fn relabel_first_layer(stream: &str, from: &str, to: &str) -> String {
    let at = stream.find(from).unwrap();
    let end = stream[at..]
        .find("\n;Z:")
        .map_or(stream.len(), |end| at + end);
    format!(
        "{}{}{}",
        &stream[..at],
        stream[at..end].replace(from, to),
        &stream[end..]
    )
}

/// Swaps X and Y in the extrusions of the roles selected by `selected`.
fn swap_axes(stream: &str, selected: impl Fn(&str) -> bool) -> String {
    let mut role = "";
    let mut out = Vec::new();
    for line in stream.lines() {
        if let Some(value) = line.strip_prefix(";TYPE:") {
            role = value;
        }
        out.push(if line.starts_with("G1 X") && selected(role) {
            line.replace('X', "#").replace('Y', "X").replace('#', "Y")
        } else {
            line.to_owned()
        });
    }
    out.join("\n")
}

fn slice(case: &crate::runner::ParityCase) -> String {
    let output = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap()
        .block_on(crate::runner::application::slice_project(
            &case.project,
            ares_core::GenerationMetadata::deterministic(2026, 8, 27, 0, 0, 0),
            ares_core::SlicerContext::new(0),
        ))
        .unwrap();
    String::from_utf8(output).unwrap()
}

fn walls(layers: &[RoofLayer]) -> Vec<(&str, usize)> {
    layers
        .iter()
        .map(|layer| (layer.z.as_str(), layer.closed_walls))
        .collect()
}

/// The Native stream carries one roof layer per requested interface layer with a closed
/// wall only on the lowest; Rust must reproduce that per-layer signature.
fn assert_roof_layers(label: &str, top_layers: usize, overrides: &[(&str, &str)]) {
    let Some(case) = super::overhang_case(label, "tree_slim", overrides) else {
        return;
    };
    let native = roof_layers(std::str::from_utf8(&case.reference).unwrap());
    assert_eq!(native.len(), top_layers, "Native roof layer count");
    assert!(native[0].closed_walls > 0, "Native first roof has a wall");
    assert!(
        native[1..].iter().all(|layer| layer.closed_walls == 0),
        "only the Native first roof has a wall"
    );
    let output = slice(&case);
    let ares = roof_layers(&output);
    assert_eq!(walls(&ares), walls(&native), "roof layers");
    assert_ne!(
        walls(&roof_layers(&relabel_first_layer(
            &output,
            ";TYPE:Support interface",
            ";TYPE:Support"
        ))),
        walls(&ares),
        "relabelling a roof layer must move the signature"
    );
    assert_eq!(
        walls(&roof_layers(&relabel_first_layer(
            &output,
            ";TYPE:Outer wall",
            ";TYPE:Inner wall"
        ))),
        walls(&ares),
        "relabelling outside the roof must not move the signature"
    );
}

#[test]
fn tree_roof_layer_count_matches_native_with_independent_heights() {
    assert_roof_layers("tree-support/slim/contained-travel", 3, &[]);
}

#[test]
fn tree_roof_layer_count_matches_native_with_object_heights() {
    assert_roof_layers(
        "tree-support/slim/roof-layers-object-heights",
        2,
        &[
            ("independent_support_layer_height", "0"),
            ("support_interface_top_layers", "2"),
        ],
    );
}

#[test]
fn tree_single_roof_layer_matches_native_with_object_heights() {
    assert_roof_layers(
        "tree-support/slim/single-roof-layer-object-heights",
        1,
        &[
            ("independent_support_layer_height", "0"),
            ("support_interface_top_layers", "1"),
        ],
    );
}

fn assert_roof_fill_axes(label: &str, overrides: &[(&str, &str)]) {
    let Some(case) = super::overhang_case(label, "tree_slim", overrides) else {
        return;
    };
    let axes = |stream: &str| -> Vec<(String, Option<FillAxis>)> {
        roof_layers(stream)
            .into_iter()
            .map(|layer| (layer.z, layer.axis))
            .collect()
    };
    let native = axes(std::str::from_utf8(&case.reference).unwrap());
    assert!(
        native.iter().all(|(_, axis)| axis.is_some()),
        "Native roof layers have a dominant fill axis"
    );
    let output = slice(&case);
    assert_eq!(axes(&output), native, "first-roof fill axis");
    assert_ne!(
        axes(&swap_axes(&output, |role| role == INTERFACE)),
        axes(&output),
        "swapping the roof axes must move the signature"
    );
    assert_eq!(
        axes(&swap_axes(&output, |role| role != INTERFACE)),
        axes(&output),
        "swapping axes outside the roof must not move the signature"
    );
}

#[test]
fn tree_first_roof_fill_axis_matches_native() {
    assert_roof_fill_axes("tree-support/slim/contained-travel", &[]);
}

#[test]
fn tree_first_roof_fill_axis_matches_native_with_grid_interface() {
    assert_roof_fill_axes(
        "tree-support/slim/grid-interface-fill-axis",
        &[("support_interface_pattern", "grid")],
    );
}
