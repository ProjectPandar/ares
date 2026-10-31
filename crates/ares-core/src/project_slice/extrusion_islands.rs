// Source boundary: OrcaSlicer v2.4.2 `GCode.cpp:4982-5000,5010-5068,8220-8249`
// island membership and sortable collection flattening.

mod tool_regions;
mod volume;
pub(in crate::project_slice) use tool_regions::{RegionToolUsage, fill_role, gap_role, wall_role};

use crate::{
    geometry::{Coord, CoordinateScale, ExPolygon, Point, fixed_gcc_sort_by},
    project_slice::{
        fill_entities::{
            FillExtrusionCollection, FillExtrusionEntity, LayerFillEntities,
            PreparedPostFillEntities,
        },
        perimeters::classic::{
            entity_collections::ExtrusionEntityCollection, gap_extrusion::GapFillEntity,
        },
    },
};

#[derive(Debug, PartialEq)]
pub(in crate::project_slice) enum IslandInfillEntity {
    Fill(FillExtrusionEntity),
    FillCollection(FillExtrusionCollection),
    Thin(GapFillEntity),
}

#[derive(Debug, Default, PartialEq)]
pub(in crate::project_slice) struct ExtrusionIsland {
    // Canonical region and original collection identity, before path flattening.
    pub(in crate::project_slice) infills: Vec<(usize, usize, IslandInfillEntity)>,
    pub(in crate::project_slice) perimeters: Vec<(usize, ExtrusionEntityCollection)>,
}

#[derive(Debug, PartialEq)]
pub(in crate::project_slice) struct LayerExtrusionIslands {
    pub(in crate::project_slice) islands: Vec<ExtrusionIsland>,
    pub(in crate::project_slice) tool_regions: Vec<RegionToolUsage>,
    pub(in crate::project_slice) wiping_volumes: volume::LayerVolumes,
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
                let source_index = prelude.identity().0;
                let object_options = &traversal
                    .resolved
                    .objects
                    .iter()
                    .find(|object| object.source_object_index == source_index)
                    .expect("prepared object retains resolved owner")
                    .object;
                let capture_wiping =
                    object_options.flush_into_infill.0 || object_options.flush_into_objects.0;
                let (compensated, _) = prelude.as_parts();
                let (_, layer_slices) = compensated.as_parts();
                layers
                    .iter_mut()
                    .zip(layer_slices)
                    .map(|(layer, slices)| {
                        assign_layer(layer, slices, traversal.scale, capture_wiping)
                    })
                    .collect()
            })
            .collect()
    };
    PreparedPostExtrusionIslands {
        predecessor,
        objects,
    }
}

fn assign_layer(
    layer: &mut LayerFillEntities,
    slices: &[ExPolygon],
    scale: CoordinateScale,
    capture_wiping: bool,
) -> LayerExtrusionIslands {
    // Print.cpp plans wiping before collection flattening and path simplification.
    let wiping_volumes = if capture_wiping {
        volume::collect(layer, scale)
    } else {
        volume::LayerVolumes::default()
    };
    let tool_regions = tool_regions::collect(layer);
    let bounds = slices.iter().map(contour_bounds).collect::<Vec<_>>();
    let mut order = (0..slices.len()).collect::<Vec<_>>();
    fixed_gcc_sort_by(&mut order, |left, right| {
        bounds[*left].area() < bounds[*right].area()
    });
    let mut islands = (0..=slices.len())
        .map(|_| ExtrusionIsland::default())
        .collect::<Vec<_>>();

    let collection_regions = layer
        .collection_ranges
        .iter()
        .flat_map(|(region, range)| std::iter::repeat_n(*region, range.len()));
    let collection_count = layer.collections.len();
    for (source_order, (region, collection)) in collection_regions
        .zip(std::mem::take(&mut layer.collections))
        .enumerate()
    {
        let island = island_index(collection.first_point(), slices, &bounds, &order);
        if collection.no_sort {
            islands[island].infills.push((
                region,
                source_order,
                IslandInfillEntity::FillCollection(collection),
            ));
        } else {
            islands[island].infills.extend(
                collection
                    .entities
                    .into_iter()
                    .map(|entity| (region, source_order, IslandInfillEntity::Fill(entity))),
            );
        }
    }
    let perimeters = std::mem::take(&mut layer.perimeters);
    let perimeter_regions = layer
        .perimeter_ranges
        .iter()
        .enumerate()
        .flat_map(|(region, range)| std::iter::repeat_n(region, range.len()));
    for (source_order, (region, mut perimeter)) in perimeter_regions.zip(perimeters).enumerate() {
        perimeter.source_order = source_order;
        let first = *perimeter.entities[0].first_point3();
        let island = island_index(Point::new(first.x, first.y), slices, &bounds, &order);
        islands[island].perimeters.push((region, perimeter));
    }
    let thin_regions = layer
        .thin_fill_ranges
        .iter()
        .enumerate()
        .flat_map(|(region, range)| std::iter::repeat_n(region, range.len()));
    for (source_order, (region, thin)) in thin_regions
        .zip(std::mem::take(&mut layer.thin_fills))
        .enumerate()
    {
        let island = island_index(thin_first_point(&thin), slices, &bounds, &order);
        islands[island].infills.push((
            region,
            collection_count + source_order,
            IslandInfillEntity::Thin(thin),
        ));
    }
    LayerExtrusionIslands {
        islands,
        tool_regions,
        wiping_volumes,
    }
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
            && point.x() < self.max_x
            && point.y() >= self.min_y
            && point.y() < self.max_y
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
