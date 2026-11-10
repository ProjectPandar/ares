//! MultiMaterialSegmentation.cpp:1247-1582: painted top/bottom segmentation.
mod projection;
use super::super::PostRegion;
use super::{clip_error, math::paths, preprocess::Layer, projection::Input, slabs};
use crate::geometry::difference_polygons_paths;
use crate::{
    FloatOrPercent, SliceError,
    geometry::{
        ClipperError, CoordinateScale, ExPolygon, FillRule, JoinType, difference_ex, opening_ex,
        union_ex, union_expolygons,
    },
};

fn color_penetration_layers(penetration: i32, shell: i32) -> usize {
    usize::try_from(if penetration > 0 { penetration } else { shell }).unwrap()
}

#[derive(Default)]
struct Stat {
    width: f32,
    spacing: f32,
    small: f32,
    top: usize,
    bottom: usize,
}
fn stat(
    regions: &[PostRegion],
    color: usize,
    (height, nozzle): (f32, f64),
    scale: CoordinateScale,
    (settings, fallback_width): (&crate::ProjectSettings, FloatOrPercent),
) -> Result<Stat, SliceError> {
    let mut out = Stat::default();
    for region in regions
        .iter()
        .filter(|r| color == 0 || r.options.outer_wall_filament_id.0 == color as i32)
    {
        let cfg = &region.options;
        let nozzles = &settings.project.print.nozzle_diameter.0;
        let nozzle = nozzles
            .get((cfg.outer_wall_filament_id.0 - 1) as usize)
            .map_or(nozzle, |value| value.0);
        let raw = match cfg.outer_wall_line_width {
            FloatOrPercent::Float(value) => value,
            FloatOrPercent::Percent(value) => value.0,
        };
        let width = match if raw == 0.0 {
            fallback_width
        } else {
            cfg.outer_wall_line_width
        } {
            FloatOrPercent::Float(value) if value <= 0.0 => f64::from(1.125_f32 * nozzle as f32),
            FloatOrPercent::Float(value) => value,
            FloatOrPercent::Percent(value) => value.0 * nozzle / 100.0,
        };
        let spacing = width as f32 - height * (1.0 - 0.25 * std::f64::consts::PI) as f32;
        if spacing <= 0.0 {
            return Err(SliceError::InvalidInput(
                "negative painted extrusion spacing".into(),
            ));
        }
        out.width = out.width.max(width as f32);
        out.top = out.top.max(color_penetration_layers(
            cfg.top_color_penetration_layers.0,
            cfg.top_shell_layers.0,
        ));
        out.bottom = out.bottom.max(color_penetration_layers(
            cfg.bottom_color_penetration_layers.0,
            cfg.bottom_shell_layers.0,
        ));
        // MultiMaterialSegmentation.cpp:1430 selects the outer-wall extruder.
        let slot = crate::options::mapped_extruder_id(
            settings,
            (cfg.outer_wall_filament_id.0 - 1) as usize,
        );
        let small = if cfg.gap_infill_speed.get_at(slot) > 0.0 {
            (0.5 * width) as f32
        } else {
            (width + f64::from(0.7_f32 * spacing)) as f32
        };
        out.small = (small * 0.5) / scale.factor() as f32;
        out.spacing = spacing;
    }
    out.width /= scale.factor() as f32;
    out.spacing /= scale.factor() as f32;
    Ok(out)
}

type Patches = Vec<Vec<Vec<ExPolygon>>>;
struct Embedded {
    base: Patches,
    shell: Patches,
}
impl Embedded {
    fn new(states: usize, count: usize) -> Self {
        Self {
            base: vec![vec![Vec::new(); count]; states],
            shell: vec![vec![Vec::new(); count]; states],
        }
    }
}
struct EmbedInput<'a> {
    raw: &'a [slabs::Surfaces],
    layers: &'a [Layer],
    stats: &'a [Vec<Stat>],
    scale: CoordinateScale,
}
fn embed_patch<const TOP: bool>(
    patch: &[ExPolygon],
    input: &EmbedInput<'_>,
    (index, color): (usize, usize),
    output: &mut Embedded,
) -> Result<(), ClipperError> {
    let stat = &input.stats[index][color];
    let count = input.layers.len();
    output.base[color][index].extend(patch.iter().cloned());
    let target_count = if TOP {
        index.saturating_sub(index.saturating_sub(stat.top) + 1)
    } else {
        (index + stat.bottom).min(count).saturating_sub(index + 1)
    };
    let targets = (0..target_count)
        .map(|step| {
            if TOP {
                index - 1 - step
            } else {
                index + 1 + step
            }
        })
        .collect::<Vec<_>>();
    let projections =
        projection::project_to_shells(patch, (input.layers, input.scale), index, &targets, stat)?;
    for (target, shell) in targets.into_iter().zip(projections) {
        output.shell[color][target].extend(shell);
    }
    Ok(())
}
fn embed<const TOP: bool>(
    input: EmbedInput<'_>,
    output: &mut Embedded,
) -> Result<(), ClipperError> {
    let count = input.layers.len();
    for index in 0..count {
        for (color, raw) in input
            .raw
            .iter()
            .enumerate()
            .filter(|(_, raw)| !raw.is_empty())
        {
            if raw[index].is_empty() {
                continue;
            }
            let stat = &input.stats[index][color];
            let patch = opening_ex(
                &union_ex(&raw[index], FillRule::NonZero)?,
                stat.small,
                JoinType::Miter,
                3.0,
            )?;
            if patch.is_empty() {
                continue;
            }
            embed_patch::<TOP>(&patch, &input, (index, color), output)?;
        }
    }
    Ok(())
}
fn merge(top: Embedded, bottom: Embedded) -> Result<Patches, ClipperError> {
    let states = top.base.len();
    let count = top.base[0].len();
    let mut out = vec![vec![Vec::new(); count]; states];
    for layer in 0..count {
        let mut painted = Vec::new();
        for (color, surfaces) in out.iter_mut().enumerate() {
            surfaces[layer] =
                union_expolygons(&bottom.base[color][layer], &top.base[color][layer])?;
            painted.extend(surfaces[layer].iter().cloned());
        }
        painted = union_expolygons(&painted, &[])?;
        for (color, surfaces) in out.iter_mut().enumerate() {
            surfaces[layer].extend(difference_ex(
                &union_expolygons(&top.shell[color][layer], &[])?,
                &painted,
            )?);
            surfaces[layer].extend(difference_ex(
                &union_expolygons(&bottom.shell[color][layer], &[])?,
                &painted,
            )?);
            surfaces[layer] = union_expolygons(&surfaces[layer], &[])?;
        }
        let mut owned = Vec::new();
        for surfaces in out.iter_mut().skip(1) {
            surfaces[layer] = difference_ex(&surfaces[layer], &owned)?;
            owned.extend(surfaces[layer].iter().cloned());
        }
        out[0][layer] = difference_ex(&out[0][layer], &owned)?;
    }
    Ok(out)
}
pub(super) fn segment(
    input: &Input<'_>,
    layers: &[Layer],
    (regions, settings): (&[PostRegion], &crate::ProjectSettings),
    heights: &[f32],
    (nozzle, fallback_width): (f64, FloatOrPercent),
) -> Result<Patches, SliceError> {
    let max_top = regions
        .iter()
        .map(|r| {
            color_penetration_layers(
                r.options.top_color_penetration_layers.0,
                r.options.top_shell_layers.0,
            )
        })
        .max()
        .unwrap();
    let max_bottom = regions
        .iter()
        .map(|r| {
            color_penetration_layers(
                r.options.bottom_color_penetration_layers.0,
                r.options.bottom_shell_layers.0,
            )
        })
        .max()
        .unwrap();
    let mut raw = slabs::collect(input, (max_top > 0, max_bottom > 0))?;
    let area = input.scale.scaled_delta(f64::from(0.1_f32)).powi(2);
    for surfaces in [&mut raw.top, &mut raw.bottom] {
        for color in surfaces.iter_mut().filter(|v| !v.is_empty()) {
            for layer in color {
                layer.retain(|p| p.area().abs() >= area);
            }
        }
    }
    for color in 0..input.states {
        for index in 0..layers.len() {
            if !raw.top[color].is_empty()
                && !raw.top[color][index].is_empty()
                && index + 1 < layers.len()
            {
                raw.top[color][index] = difference_polygons_paths(
                    &raw.top[color][index],
                    &paths(layers[index + 1].input.iter().cloned()),
                )
                .map_err(clip_error)?;
            }
            if !raw.bottom[color].is_empty() && !raw.bottom[color][index].is_empty() && index > 0 {
                raw.bottom[color][index] = difference_polygons_paths(
                    &raw.bottom[color][index],
                    &paths(layers[index - 1].input.iter().cloned()),
                )
                .map_err(clip_error)?;
            }
        }
    }
    let stats = heights
        .iter()
        .map(|&h| {
            (0..input.states)
                .map(|color| {
                    stat(
                        regions,
                        color,
                        (h, nozzle),
                        input.scale,
                        (settings, fallback_width),
                    )
                })
                .collect::<Result<Vec<_>, _>>()
        })
        .collect::<Result<Vec<_>, _>>()?;
    let (mut top, mut bottom) = (
        Embedded::new(input.states, layers.len()),
        Embedded::new(input.states, layers.len()),
    );
    embed::<true>(
        EmbedInput {
            raw: &raw.top,
            layers,
            stats: &stats,
            scale: input.scale,
        },
        &mut top,
    )
    .map_err(clip_error)?;
    embed::<false>(
        EmbedInput {
            raw: &raw.bottom,
            layers,
            stats: &stats,
            scale: input.scale,
        },
        &mut bottom,
    )
    .map_err(clip_error)?;
    merge(top, bottom).map_err(clip_error)
}
