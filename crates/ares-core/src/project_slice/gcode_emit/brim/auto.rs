use super::geometry::{even_width_mm, outer_width};
use crate::geometry::{BoundingBox, CoordinateScale, ExPolygon, Line};

pub(crate) struct WidthContext<'a> {
    pub(crate) max_speed: f64,
    pub(crate) adhesion: f64,
    pub(crate) mesh_bbox: [f64; 3],
    pub(crate) volume_materials: &'a [(usize, String)],
    pub(crate) volume_ids: &'a [usize],
}

pub(crate) fn width_mm(
    source: &[ExPolygon],
    scale: CoordinateScale,
    context: &WidthContext<'_>,
) -> f64 {
    // Brim.cpp:260-305: group width uses second-moment/thermal/max-speed terms.
    let (mut min_x, mut max_x, mut min_y, mut max_y) = (
        f64::INFINITY,
        f64::NEG_INFINITY,
        f64::INFINITY,
        f64::NEG_INFINITY,
    );
    for expolygon in source {
        for point in expolygon.contour().points() {
            let x = scale.unscale(point.x());
            let y = scale.unscale(point.y());
            min_x = min_x.min(x);
            max_x = max_x.max(x);
            min_y = min_y.min(y);
            max_y = max_y.max(y);
        }
    }
    let width = (max_x - min_x).max(0.0);
    let depth = (max_y - min_y).max(0.0);
    let diagonal = width.hypot(depth); // Brim.cpp:289-294 bbox2 diagonal.
    let height = context.mesh_bbox[2]; // Brim.cpp:262-276 merged mesh bbox height.
    let (ixx, iyy) = second_moment(source, scale);
    // Brim.cpp:267-276: the max of the two bbox-weighted moments.
    let height_to_area = (height / ixx * depth).max(height / iyy * width) * height / 1920.0;
    // Model.cpp:3258-3280 takes only the thermal reference per group.
    let names = context
        .volume_materials
        .iter()
        .filter(|(index, _)| context.volume_ids.contains(index))
        .map(|(_, name)| name.as_str());
    let (_, thermal_length_ref) = material_terms(names);
    // Model.cpp:3258-3280 getThermalLength; Brim.cpp:298-303 compares the
    // geometry thermal term against this material reference.
    let thermal = diagonal * 8.0 / thermal_length_ref * (height.min(30.0) / 30.0);
    let brim = context.adhesion
        * (height_to_area * context.max_speed)
            .max(thermal)
            .min(18.0)
            .min(1.5 * diagonal);
    if brim < 5.0 && brim < 1.5 * diagonal {
        0.0
    } else {
        brim.clamp(0.0, 18.0)
    }
}

// MaterialType.cpp:134-168. Unknown names use the upstream defaults rather
// than a hand-written material fallback.
pub(super) fn material_terms<'a>(names: impl IntoIterator<Item = &'a str>) -> (f64, f64) {
    let mut adhesion = 1.0;
    let mut thermal_length: f64 = 1250.0;
    for name in names {
        let (current_adhesion, current_thermal) = match name {
            "FLEX" | "TPU" => (0.5, 1000.0),
            "PET" | "PET-CF" | "PET-GF" | "PETG" => (2.0, 100.0),
            "PETG-CF" | "PETG-GF" => (1.0, 100.0),
            "PA" | "PA-CF" | "PA-GF" | "PA6" | "PA6-CF" | "PA6-GF" | "PA11" | "PA11-CF"
            | "PA11-GF" | "PA12" | "PA12-CF" | "PA12-GF" => (1.0, 100.0),
            "ABS" | "ABS-CF" | "ABS-GF" | "ASA" | "ASA-CF" | "ASA-GF" | "ASA-AERO" | "BVOH"
            | "CoPE" | "EVA" | "HIPS" | "PC" | "PC-ABS" | "PC-CF" | "PC-PBT" | "PCL" | "PE"
            | "PE-CF" | "PE-GF" | "PEI-1010" | "PEI-1010-CF" | "PEI-1010-GF" | "PEI-9085"
            | "PEI-9085-CF" | "PEI-9085-GF" | "PEEK" | "PEEK-CF" | "PEEK-GF" | "PEKK"
            | "PEKK-CF" | "PEKK-GF" | "PES" | "PHA" | "PI" | "PLA" | "PLA-AERO" | "PLA-CF"
            | "POM" | "PP" | "PP-CF" | "PP-GF" | "PPA-CF" | "PPA-GF" | "PPS" | "PPS-CF"
            | "PPSU" | "PSU" | "PVA" | "PVB" | "PVDF" | "SBS" | "TPI" => (1.0, 200.0),
            _ => (1.0, 200.0),
        };
        adhesion = current_adhesion;
        thermal_length = thermal_length.min(current_thermal);
    }
    (adhesion, thermal_length)
}

fn second_moment(source: &[ExPolygon], scale: CoordinateScale) -> (f64, f64) {
    // Brim.cpp:174-258: Polygon/ExPolygon moments and parallel-axis merge.
    let scale4 = scale.factor().powi(4);
    if source.is_empty() {
        // Brim.cpp:277-283 leaves the failed-moment sentinel unchanged.
        return (-1.0e30 * scale4, -1.0e30 * scale4);
    }
    let mut props = Vec::new();
    for expolygon in source {
        let mut area = 0.0;
        let mut cx = 0.0;
        let mut cy = 0.0;
        let mut smx = 0.0;
        let mut smy = 0.0;
        for polygon in std::iter::once(expolygon.contour()).chain(expolygon.holes()) {
            let points = polygon.points();
            for (p1, p2) in points
                .iter()
                .copied()
                .cycle()
                .zip(points.iter().copied().cycle().skip(1))
                .take(points.len())
            {
                let x1 = p1.x() as f64;
                let y1 = p1.y() as f64;
                let x2 = p2.x() as f64;
                let y2 = p2.y() as f64;
                let cross = x1 * y2 - y1 * x2;
                area += cross / 2.0;
                cx += (x1 + x2) * cross / 6.0;
                cy += (y1 + y2) * cross / 6.0;
                smx += (y1 * y1 + y1 * y2 + y2 * y2) * cross / 12.0;
                smy += (x1 * x1 + x1 * x2 + x2 * x2) * cross / 12.0;
            }
        }
        props.push((area, cx / area, cy / area, smx, smy));
    }
    let total_area = props.iter().map(|p| p.0).sum::<f64>();
    let total_cx = props.iter().map(|p| p.1 * p.0).sum::<f64>() / total_area;
    let total_cy = props.iter().map(|p| p.2 * p.0).sum::<f64>() / total_area;
    let mut ixx = 0.0;
    let mut iyy = 0.0;
    for (area, cx, cy, smx, smy) in props {
        ixx += smx - cy * cy * area + area * (cy - total_cy).powi(2);
        iyy += smy - cx * cx * area + area * (cx - total_cx).powi(2);
    }
    (ixx * scale4, iyy * scale4)
}

fn polygon_width(
    polygon: &ExPolygon,
    (group_width, additional_width): (f32, f32),
    group_height: f64,
    (spacing, scale): (f32, CoordinateScale),
) -> f32 {
    let mut width = group_width;
    let contour = polygon.contour();
    // Brim.cpp:569–574, Polygon.cpp:24–32: keep the adhesion ratio in
    // scaled units and sum the closing edge before the remaining edges.
    if f64::from(width) < scale.scaled_delta(5.0) && group_height > 10.0 && width < spacing {
        let points = contour.points();
        let closing = Line::new(points[points.len() - 1], points[0]).length();
        let length = points.windows(2).fold(closing, |length, pair| {
            length + Line::new(pair[0], pair[1]).length()
        });
        if polygon.area() / length < f64::from(scale.scaled_delta(1.1) as f32) {
            width += additional_width;
        }
    }
    let bbox = BoundingBox::from_polygon(contour).expect("nonempty brim contour");
    let max_side = (bbox.max().x() - bbox.min().x()).max(bbox.max().y() - bbox.min().y());
    outer_width(width.min(max_side as f32), spacing)
}

pub(super) struct AutoGroupContext<'a> {
    pub(super) traversal:
        &'a crate::project_slice::perimeters::classic::traversal::PreparedPostClassicTraversal,
    pub(super) object_index: usize,
    pub(super) max_speed: f64,
    pub(super) adhesion: f64,
    pub(super) spacing_mm: f64,
    pub(super) scale: CoordinateScale,
    pub(super) gap: f32,
    pub(super) spacing: f32,
    pub(super) resolution: f64,
    pub(super) volume_materials: &'a [(usize, String)],
    pub(super) efc_bottom: Option<&'a [ExPolygon]>,
}

type AutoGroupAreas = (
    Vec<crate::geometry::ExPolygon>,
    Vec<crate::geometry::ExPolygon>,
    Vec<crate::geometry::ExPolygon>,
);

pub(super) fn auto_group_area(
    context: &AutoGroupContext<'_>,
    group: &super::groups::BrimVolumeGroup,
) -> Result<AutoGroupAreas, crate::SliceError> {
    let group_bbox = super::groups::model_mesh_bbox(
        context.traversal,
        context.object_index,
        Some(&group.volume_ids),
    );
    let width_context = WidthContext {
        max_speed: context.max_speed,
        adhesion: context.adhesion,
        mesh_bbox: group_bbox,
        volume_materials: context.volume_materials,
        volume_ids: &group.volume_ids,
    };
    let scaled_width = |mm| {
        context
            .scale
            .scaled_delta(even_width_mm(mm, context.spacing_mm)) as f32
    };
    let group_width = scaled_width(width_mm(&group.slices, context.scale, &width_context));
    let additional_width = scaled_width(5.0);
    // Brim.cpp:552–562 clips only the per-polygon consumer; group width
    // above retains the original slices for its moment/height calculation.
    let clipped;
    let slices = if let Some(bottom) = context.efc_bottom {
        clipped = crate::geometry::intersection_ex(bottom, &group.slices)
            .map_err(super::brim_geometry_error)?;
        clipped.as_slice()
    } else {
        &group.slices
    };
    let mut area = Vec::new();
    let mut forbidden = Vec::new();
    let mut islands = Vec::new();
    for polygon in slices {
        let outer_width = polygon_width(
            polygon,
            (group_width, additional_width),
            group_bbox[2],
            (context.spacing, context.scale),
        );
        if outer_width <= 0.0 {
            continue;
        }
        let one = crate::brims::outer::area(
            std::slice::from_ref(polygon),
            context.gap,
            outer_width,
            context.spacing,
            context.resolution,
        )?;
        area.extend(one.area);
        forbidden.extend(one.forbidden);
        islands.extend(one.object_islands);
    }
    Ok((area, forbidden, islands))
}
