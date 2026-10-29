//! Geometry/Voronoi.cpp default segment constructor; indexed local ownership.
mod cell_range;
mod detect;
mod diagram;
mod planarity;
mod predicates;
mod rotation;

use crate::geometry::Line;
use boostvoronoi::{
    BvError,
    prelude::{Builder, Line as BvLine},
};
pub(crate) use diagram::Diagram;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Issue {
    None,
    NonFiniteVertex,
    MissingVertex,
    NonPlanar,
    IntersectingSegment,
    ParabolaWithoutFocus,
    Unknown,
}
#[derive(Clone, Copy, PartialEq, Eq)]
enum State {
    NotNeeded,
    Successful,
    Unsuccessful,
    Unknown,
}

fn raw(lines: &[Line]) -> Result<boostvoronoi::prelude::Diagram, BvError> {
    Builder::<i64>::default()
        .with_segments(
            lines
                .iter()
                .map(|l| BvLine::from([l.a.x(), l.a.y(), l.b.x(), l.b.y()])),
        )?
        .build()
}

pub(crate) fn build(lines: &[Line]) -> Result<Diagram, BvError> {
    let mut output = Diagram::new(raw(lines)?, false);
    output.issue = detect::issues(&output, lines);
    if output.issue == Issue::None {
        output.state = State::NotNeeded;
        return Ok(output);
    }
    // Voronoi.cpp248-261: ordered attempts, including remapping on failed attempts.
    for divisor in [6.0, 5.0, 7.0, 11.0] {
        let angle = std::f64::consts::PI / divisor;
        let rotated = rotation::sites(lines, angle);
        output = Diagram::new(raw(&rotated)?, true);
        output.issue = detect::issues(&output, &rotated);
        rotation::remap(&mut output, &rotated, lines, angle);
        if output.issue == Issue::None {
            output.state = State::Successful;
            return Ok(output);
        }
    }
    output.state = State::Unsuccessful;
    Ok(output)
}
