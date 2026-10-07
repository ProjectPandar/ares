use ares_vgcode::{Color, ColorPrintLegend, PartialTimeKind, TimeMode};
use serde::Serialize;

// Transport only: record selection and missing-volume policy belong to vgcode.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ColorPrintView {
    visible: bool,
    extruders: Vec<ExtruderView>,
    partial_usage: PartialUsageView,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ExtruderView {
    extruder_id: u8,
    segments: Vec<SegmentView>,
    ranges: Vec<RangeView>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SegmentView {
    color_id: u8,
    layer_id: u32,
    times: [f32; TimeMode::COUNT],
    color: Color,
}

#[derive(Serialize)]
struct RangeView {
    color: Color,
    z: [f64; 2],
}

#[derive(Serialize)]
#[serde(
    tag = "status",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
enum PartialUsageView {
    Available { rows: Vec<PartialTimeView> },
    Unavailable { missing_index: usize },
}

#[derive(Serialize)]
#[serde(remote = "PartialTimeKind")]
enum PartialTimeKindView {
    Print,
    ColorChange,
    Pause,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct PartialTimeView {
    #[serde(with = "PartialTimeKindView")]
    kind: PartialTimeKind,
    extruder_id: u32,
    color1: Color,
    color2: Color,
    duration: f32,
    remaining: f32,
    volume: Option<f64>,
}

impl From<&ColorPrintLegend> for ColorPrintView {
    fn from(legend: &ColorPrintLegend) -> Self {
        Self {
            visible: legend.visible,
            extruders: legend
                .extruders
                .iter()
                .map(|extruder| ExtruderView {
                    extruder_id: extruder.extruder_id,
                    segments: extruder
                        .segments
                        .iter()
                        .map(|item| SegmentView {
                            color_id: item.segment.color_id,
                            layer_id: item.segment.layer_id,
                            times: item.segment.times,
                            color: item.color,
                        })
                        .collect(),
                    ranges: extruder
                        .ranges
                        .iter()
                        .map(|range| RangeView {
                            color: range.color,
                            z: range.z,
                        })
                        .collect(),
                })
                .collect(),
            partial_usage: match &legend.partial_usage {
                Ok(rows) => PartialUsageView::Available {
                    rows: rows
                        .iter()
                        .map(|row| PartialTimeView {
                            kind: row.kind,
                            extruder_id: row.extruder_id,
                            color1: row.color1,
                            color2: row.color2,
                            duration: row.duration,
                            remaining: row.remaining,
                            volume: row.volume,
                        })
                        .collect(),
                },
                Err(error) => PartialUsageView::Unavailable {
                    missing_index: error.missing_index,
                },
            },
        }
    }
}
