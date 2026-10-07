use ares_vgcode::ColorPrintLegend;
use serde_json::{Value, json};

// Transport only: record selection and missing-volume policy belong to vgcode.
pub(crate) fn legend_json(legend: &ColorPrintLegend) -> Value {
    let partial_usage = match &legend.partial_usage {
        Ok(rows) => json!({"status": "available", "rows": rows.iter().map(|row| json!({
            "kind": format!("{:?}", row.kind), "extruderId": row.extruder_id,
            "color1": row.color1, "color2": row.color2,
            "duration": row.duration, "remaining": row.remaining, "volume": row.volume,
        })).collect::<Vec<_>>() }),
        Err(error) => json!({"status": "unavailable", "missingIndex": error.missing_index}),
    };
    json!({
        "visible": legend.visible,
        "extruders": legend.extruders.iter().map(|extruder| json!({
            "extruderId": extruder.extruder_id,
            "segments": extruder.segments.iter().map(|item| json!({
                "colorId": item.segment.color_id, "layerId": item.segment.layer_id,
                "times": item.segment.times, "color": item.color,
            })).collect::<Vec<_>>(),
            "ranges": extruder.ranges.iter().map(|range| json!({
                "color": range.color, "z": range.z,
            })).collect::<Vec<_>>(),
        })).collect::<Vec<_>>(),
        "partialUsage": partial_usage,
    })
}
