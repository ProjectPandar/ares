//! `WipeTower.cpp:3080–3160`: finish-layer solid/sparse grid.
use super::super::geometry::BoxCoordinates;
use super::{generator::Generator, writer::Writer};
use std::fmt::Write;

pub(super) fn fill(
    generator: &Generator<'_>,
    writer: &mut Writer,
    bounds: BoxCoordinates,
    (fill, wall): (bool, bool),
    speed: f32,
) -> Option<[[f32; 2]; 2]> {
    let mut wipe = if wall && bounds.size()[1] > 1e-3 {
        generator.rectangle_fill(writer, bounds, speed)
    } else {
        None
    };
    let perimeter = generator.plan.perimeter;
    let dy = bounds.size()[1] - perimeter;
    let mut left = bounds.lower[0] + 2.0 * perimeter;
    let mut right = bounds.upper[0] - 2.0 * perimeter;
    if !fill || dy <= perimeter {
        return wipe;
    }
    writer.travel([left, bounds.lower[1]], 0.0);
    writeln!(
        writer.gcode,
        ";--------------------\n; CP EMPTY GRID START\n; layer #{}",
        generator.index + 2
    )
    .unwrap();
    let solid = generator.first_layer()
        || generator
            .plan
            .layers
            .get(generator.index + 1)
            .is_some_and(|layer| {
                layer.changes.iter().any(|c| {
                    generator.filaments[c.new].soluble || generator.filaments[c.old].soluble
                })
            });
    if solid {
        let factor = if generator.first_layer() {
            left -= perimeter;
            right += perimeter;
            1.0
        } else {
            1.5
        };
        let mut y = bounds.lower[1] + perimeter;
        let count = (dy / (perimeter * factor)) as i32;
        let spacing = (dy - perimeter) / (count - 1) as f32;
        for i in 0..count {
            writer.extrude([writer.position[0], y], speed);
            writer.extrude([if i % 2 == 1 { left } else { right }, y], 0.0);
            y += spacing;
        }
        writer.extrude([writer.position[0], bounds.upper[1]], 0.0);
    } else {
        writer.extrude([left, bounds.upper[1]], speed);
        let count = 1 + ((right - left) / 10.0) as i32;
        let step = (right - left) / count as f32;
        for i in 1..=count {
            let x = left + step * i as f32;
            writer.travel([x, writer.position[1]], 0.0);
            writer.extrude(
                [
                    x,
                    if i % 2 == 1 {
                        bounds.lower[1]
                    } else {
                        bounds.upper[1]
                    },
                ],
                0.0,
            );
        }
        wipe = Some([
            writer.position,
            [
                left + step * count as f32,
                if count % 2 == 1 {
                    bounds.upper[1]
                } else {
                    bounds.lower[1]
                },
            ],
        ]);
    }
    writer
        .gcode
        .push_str("; CP EMPTY GRID END\n;------------------\n\n\n\n\n\n\n");
    wipe
}
