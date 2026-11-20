use crate::geometry::{
    BoundingBox, ClipperError, CoordinateScale, ExPolygon, Point, intersection_ex,
};

pub(super) struct BodyLayout {
    ids: Vec<Vec<usize>>,
    boxes: Vec<Vec<BoundingBox>>,
    pub(super) count: usize,
}

impl BodyLayout {
    // ConnectedBodies.cpp:27-89: only positive-area intersection between
    // adjacent layers joins islands. Number bodies by first island encounter.
    pub(super) fn connected(layers: &[Vec<ExPolygon>]) -> Result<Self, ClipperError> {
        let mut first = Vec::with_capacity(layers.len() + 1);
        first.push(0);
        for islands in layers {
            first.push(first.last().unwrap() + islands.len());
        }
        let boxes: Vec<Vec<_>> = layers
            .iter()
            .map(|islands| {
                islands
                    .iter()
                    .map(|island| BoundingBox::from_expolygon(island).unwrap())
                    .collect()
            })
            .collect();
        let mut parent: Vec<_> = (0..*first.last().unwrap()).collect();
        for (layer, adjacent) in layers.windows(2).enumerate() {
            connect_adjacent(layer, adjacent, &boxes, &first, &mut parent)?;
        }
        let (ids, count) = assign_body_ids(&first, &mut parent);
        Ok(Self { ids, boxes, count })
    }

    // PrintObject.cpp:1115-1131: prefer containment, otherwise the nearest
    // outline strictly within one millimeter. Iteration breaks ties by island order.
    pub(super) fn body_at(
        &self,
        layer: usize,
        islands: &[ExPolygon],
        point: Point,
        scale: CoordinateScale,
    ) -> Option<usize> {
        let mut best = scale.scaled_delta(1.0);
        let mut body = None;
        for (i, island) in islands.iter().enumerate() {
            let Some(distance) = candidate_distance(&self.boxes[layer][i], island, point, best)
            else {
                continue;
            };
            if distance < best && update_nearest(&mut best, &mut body, distance, self.ids[layer][i])
            {
                break;
            }
        }
        body
    }

    // Fill.cpp:1308-1330: fill and anchoring select the greatest intersection
    // area, not the lookup of the contour point used during tree preparation.
    pub(super) fn fill_body(
        &self,
        layer: usize,
        islands: &[ExPolygon],
        surface: &ExPolygon,
    ) -> Result<Option<usize>, ClipperError> {
        let bounds = BoundingBox::from_expolygon(surface).unwrap();
        let mut best = 0.0;
        let mut body = None;
        for (i, island) in islands.iter().enumerate() {
            let Some(area) = overlap_area(&self.boxes[layer][i], bounds, island, surface)? else {
                continue;
            };
            if area > best {
                best = area;
                body = Some(self.ids[layer][i]);
            }
        }
        Ok(body)
    }
}

fn connect_adjacent(
    layer: usize,
    adjacent: &[Vec<ExPolygon>],
    boxes: &[Vec<BoundingBox>],
    first: &[usize],
    parent: &mut [usize],
) -> Result<(), ClipperError> {
    let [lower, upper] = adjacent else {
        unreachable!()
    };
    let mut upper_by_x: Vec<_> = (0..upper.len()).collect();
    upper_by_x.sort_unstable_by_key(|&i| boxes[layer + 1][i].min().x());
    for (i, island) in lower.iter().enumerate() {
        let bounds = boxes[layer][i];
        let end =
            upper_by_x.partition_point(|&j| boxes[layer + 1][j].min().x() <= bounds.max().x());
        for &j in upper_by_x[..end]
            .iter()
            .filter(|&&j| bounds.overlap(boxes[layer + 1][j]))
        {
            join_islands((layer, i, j), (island, &upper[j]), (first, parent))?;
        }
    }
    Ok(())
}

fn join_islands(
    indices: (usize, usize, usize),
    islands: (&ExPolygon, &ExPolygon),
    context: (&[usize], &mut [usize]),
) -> Result<(), ClipperError> {
    let (layer, lower, upper) = indices;
    let (lower_island, upper_island) = islands;
    let (first, parent) = context;
    let a = root(parent, first[layer] + lower);
    let b = root(parent, first[layer + 1] + upper);
    if a != b && !intersect(lower_island, upper_island)?.is_empty() {
        parent[a] = b;
    }
    Ok(())
}

fn assign_body_ids(first: &[usize], parent: &mut [usize]) -> (Vec<Vec<usize>>, usize) {
    let mut body_ids = vec![None; parent.len()];
    let mut count = 0;
    let ids = first
        .windows(2)
        .map(|range| {
            (range[0]..range[1])
                .map(|i| {
                    *body_ids[root(parent, i)].get_or_insert_with(|| {
                        let id = count;
                        count += 1;
                        id
                    })
                })
                .collect()
        })
        .collect();
    (ids, count)
}

fn candidate_distance(
    bounds: &BoundingBox,
    island: &ExPolygon,
    point: Point,
    best: f64,
) -> Option<f64> {
    let mut bounds = *bounds;
    bounds.offset(best as i64);
    if !bounds.contains(point) {
        return None;
    }
    if island.contour().contains(&point) && !island.holes().iter().any(|hole| hole.contains(&point))
    {
        return Some(0.0);
    }
    Some(
        std::iter::once(island.contour())
            .chain(island.holes())
            .map(|polygon| polygon.point_projection(point))
            .map(|p| ((p.x() - point.x()) as f64).hypot((p.y() - point.y()) as f64))
            .fold(f64::INFINITY, f64::min),
    )
}

fn update_nearest(best: &mut f64, body: &mut Option<usize>, distance: f64, id: usize) -> bool {
    *best = distance;
    *body = Some(id);
    distance == 0.0
}

fn overlap_area(
    bounds: &BoundingBox,
    surface_bounds: BoundingBox,
    island: &ExPolygon,
    surface: &ExPolygon,
) -> Result<Option<f64>, ClipperError> {
    if !bounds.overlap(surface_bounds) {
        return Ok(None);
    }
    Ok(Some(
        intersect(island, surface)?
            .iter()
            .map(ExPolygon::area)
            .sum(),
    ))
}

fn root(parent: &mut [usize], mut i: usize) -> usize {
    while parent[i] != i {
        parent[i] = parent[parent[i]];
        i = parent[i];
    }
    i
}

fn intersect(a: &ExPolygon, b: &ExPolygon) -> Result<Vec<ExPolygon>, ClipperError> {
    intersection_ex(std::slice::from_ref(a), std::slice::from_ref(b))
}
