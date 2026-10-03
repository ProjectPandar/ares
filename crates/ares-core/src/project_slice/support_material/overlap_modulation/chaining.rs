//! Fragment consumption and connected multipaths from `SupportCommon.cpp:1231-1328`.

use super::{PathFragment, lookup::FragmentLookup};
use crate::{
    ExtrusionRole,
    geometry::{Point, Polyline},
    project_slice::perimeters::classic::materialize::Point3,
    project_slice::support_material::extrusion_path::{
        SupportExtrusionEntity, SupportExtrusionPath,
    },
};

pub(super) fn reconnect(
    output: &mut Vec<SupportExtrusionEntity>,
    fragments: &mut [PathFragment],
    ends: &[(Point, Point)],
    role: ExtrusionRole,
) {
    let lookup = FragmentLookup::new(fragments);
    for &(start, end) in ends {
        let mut current = start;
        let mut paths: Vec<SupportExtrusionPath> = Vec::new();
        while let Some(endpoint) = lookup.find(current, fragments) {
            let fragment = &mut fragments[endpoint.layer];
            let mut line = std::mem::replace(
                &mut fragment.polylines[endpoint.polyline],
                Polyline::new(Vec::new()),
            );
            if !endpoint.start {
                line.reverse();
            }
            line.points_mut()[0] = current;
            let compatible = paths.last().is_some_and(|path| {
                path.height == fragment.height && path.mm3_per_mm == fragment.mm3_per_mm
            });
            if compatible {
                let path = paths.last_mut().unwrap();
                path.polyline.points.pop();
                let first = Point3 {
                    x: current.x(),
                    y: current.y(),
                    z: 0,
                };
                let skip_first = usize::from(path.polyline.points.last() == Some(&first));
                path.polyline
                    .points
                    .extend(
                        line.into_points()
                            .into_iter()
                            .skip(skip_first)
                            .map(|point| Point3 {
                                x: point.x(),
                                y: point.y(),
                                z: 0,
                            }),
                    );
            } else {
                paths.push(SupportExtrusionPath::from_parameters(
                    line,
                    role,
                    (fragment.mm3_per_mm, fragment.width, fragment.height),
                    true,
                ));
            }
            let back = paths.last().unwrap().polyline.points.last().unwrap();
            current = Point::new(back.x, back.y);
            if current == end {
                break;
            }
        }
        if paths.len() == 1 {
            output.push(SupportExtrusionEntity::Path(paths.pop().unwrap()));
        } else if !paths.is_empty() {
            output.push(SupportExtrusionEntity::MultiPath(paths));
        }
    }
    for fragment in fragments {
        output.extend(
            std::mem::take(&mut fragment.polylines)
                .into_iter()
                .filter(|line| line.is_valid())
                .map(|line| {
                    SupportExtrusionEntity::Path(SupportExtrusionPath::from_parameters(
                        line,
                        role,
                        (fragment.mm3_per_mm, fragment.width, fragment.height),
                        true,
                    ))
                }),
        );
    }
}
