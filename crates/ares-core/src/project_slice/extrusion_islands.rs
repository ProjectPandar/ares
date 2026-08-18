use std::collections::HashMap;

use crate::{
    geometry::{Coord, ExPolygon, Point},
    project_slice::{
        fill_entities::{FillExtrusionCollection, LayerFillEntities, PreparedPostFillEntities},
        perimeters::classic::{
            entity_collections::ExtrusionEntityCollection, gap_extrusion::GapFillEntity,
        },
    },
};

#[derive(Debug, PartialEq)]
pub(in crate::project_slice) enum IslandInfillEntity {
    Fill(FillExtrusionCollection),
    Thin(GapFillEntity),
}

#[derive(Debug, Default, PartialEq)]
pub(in crate::project_slice) struct ExtrusionIsland {
    pub(in crate::project_slice) infills: Vec<IslandInfillEntity>,
    pub(in crate::project_slice) perimeters: Vec<ExtrusionEntityCollection>,
}

#[derive(Debug, PartialEq)]
pub(in crate::project_slice) struct LayerExtrusionIslands {
    pub(in crate::project_slice) islands: Vec<ExtrusionIsland>,
}

pub(in crate::project_slice) struct PreparedPostExtrusionIslands {
    pub(in crate::project_slice) predecessor: PreparedPostFillEntities,
    pub(in crate::project_slice) objects: Vec<Vec<LayerExtrusionIslands>>,
}

pub(in crate::project_slice) fn prepare(
    mut predecessor: PreparedPostFillEntities,
) -> PreparedPostExtrusionIslands {
    let objects = {
        let traversal = &predecessor
            .predecessor
            .predecessor
            .predecessor
            .predecessor
            .predecessor;
        traversal
            .objects
            .iter()
            .zip(&mut predecessor.objects)
            .map(|(source, layers)| {
                let prelude = &source
                    .predecessor
                    .predecessor
                    .predecessor
                    .predecessor
                    .object;
                let (compensated, _) = prelude.as_parts();
                let (_, layer_slices) = compensated.as_parts();
                layers
                    .iter_mut()
                    .zip(layer_slices)
                    .map(|(layer, slices)| assign_layer(layer, slices))
                    .collect()
            })
            .collect()
    };
    PreparedPostExtrusionIslands {
        predecessor,
        objects,
    }
}

fn assign_layer(layer: &mut LayerFillEntities, slices: &[ExPolygon]) -> LayerExtrusionIslands {
    let bounds = slices.iter().map(contour_bounds).collect::<Vec<_>>();
    let mut order = (0..slices.len()).collect::<Vec<_>>();
    order.sort_unstable_by_key(|&index| bounds[index].area());
    let mut islands = (0..=slices.len())
        .map(|_| ExtrusionIsland::default())
        .collect::<Vec<_>>();

    for collection in std::mem::take(&mut layer.collections) {
        let island = island_index(
            collection.paths[0].polyline.points()[0],
            slices,
            &bounds,
            &order,
        );
        islands[island]
            .infills
            .push(IslandInfillEntity::Fill(collection));
    }
    let perimeter_sources = std::mem::take(&mut layer.perimeter_source_indices);
    let perimeters = std::mem::take(&mut layer.perimeters);
    assert_eq!(perimeters.len(), perimeter_sources.len());
    let mut source_islands = HashMap::new();
    let mut perimeter_islands = Vec::with_capacity(perimeters.len());
    for (perimeter, source_index) in perimeters.into_iter().zip(perimeter_sources) {
        let path = &perimeter.entities[0].extrusion_loop.paths[0];
        let first = path.polyline.points[0];
        let island = island_index(Point::new(first.x, first.y), slices, &bounds, &order);
        source_islands
            .entry(source_index)
            .or_insert_with(Vec::new)
            .push((island, Point::new(first.x, first.y)));
        perimeter_islands.push((island, Point::new(first.x, first.y)));
        islands[island].perimeters.push(perimeter);
    }
    let thin_sources = std::mem::take(&mut layer.thin_fill_source_indices);
    let thin_fills = std::mem::take(&mut layer.thin_fills);
    assert_eq!(thin_fills.len(), thin_sources.len());
    for (thin, source_index) in thin_fills.into_iter().zip(thin_sources) {
        let point = thin_first_point(&thin);
        let source_candidates = source_index
            .and_then(|source_index| source_islands.get(&source_index))
            .filter(|candidates| candidates.len() > 1);
        let candidates = source_candidates.unwrap_or(&perimeter_islands);
        let island = candidates
            .iter()
            .min_by_key(|(_, candidate)| squared_distance(point, *candidate))
            .map(|(island, _)| *island)
            .unwrap_or_else(|| island_index(point, slices, &bounds, &order));
        islands[island].infills.push(IslandInfillEntity::Thin(thin));
    }
    LayerExtrusionIslands { islands }
}

fn thin_first_point(entity: &GapFillEntity) -> Point {
    let point = match entity {
        GapFillEntity::Path(path) => path.polyline.points[0],
        GapFillEntity::Loop(paths) => paths[0].polyline.points[0],
    };
    Point::new(point.x, point.y)
}

fn island_index(point: Point, slices: &[ExPolygon], bounds: &[Bounds], order: &[usize]) -> usize {
    order
        .iter()
        .copied()
        .find(|&index| bounds[index].contains(point) && slices[index].contour().contains(&point))
        .unwrap_or(slices.len())
}

fn squared_distance(left: Point, right: Point) -> i128 {
    let dx = i128::from(left.x() - right.x());
    let dy = i128::from(left.y() - right.y());
    dx * dx + dy * dy
}

#[derive(Clone, Copy)]
struct Bounds {
    min_x: Coord,
    min_y: Coord,
    max_x: Coord,
    max_y: Coord,
}

impl Bounds {
    fn contains(self, point: Point) -> bool {
        point.x() >= self.min_x
            && point.x() <= self.max_x
            && point.y() >= self.min_y
            && point.y() <= self.max_y
    }

    fn area(self) -> i128 {
        i128::from(self.max_x - self.min_x) * i128::from(self.max_y - self.min_y)
    }
}

fn contour_bounds(expolygon: &ExPolygon) -> Bounds {
    let first = expolygon.contour().points()[0];
    expolygon.contour().points().iter().copied().fold(
        Bounds {
            min_x: first.x(),
            min_y: first.y(),
            max_x: first.x(),
            max_y: first.y(),
        },
        |bounds, point| Bounds {
            min_x: bounds.min_x.min(point.x()),
            min_y: bounds.min_y.min(point.y()),
            max_x: bounds.max_x.max(point.x()),
            max_y: bounds.max_y.max(point.y()),
        },
    )
}

pub(in crate::project_slice) fn dispose(prepared: PreparedPostExtrusionIslands) {
    super::fill_entities::dispose(prepared.predecessor);
}
#[cfg(test)]
mod tests {
    use crate::geometry::{ExPolygon, Point, Polygon};

    use super::{contour_bounds, island_index};

    #[test]
    fn task22o210_maximum_boundary_selects_smallest_containing_island() {
        let rectangle = |minimum, maximum| {
            ExPolygon::new(
                Polygon::new(vec![
                    Point::new(minimum, minimum),
                    Point::new(maximum, minimum),
                    Point::new(maximum, maximum),
                    Point::new(minimum, maximum),
                ]),
                Vec::new(),
            )
        };
        let slices = [rectangle(0, 10), rectangle(-10, 20)];
        let bounds = slices.iter().map(contour_bounds).collect::<Vec<_>>();

        assert_eq!(
            island_index(Point::new(10, 5), &slices, &bounds, &[0, 1]),
            0
        );
    }
}
