pub(super) fn cache_snapshot(
    objects: &[crate::project_slice::prepare_infill::vertical_shells::types::VerticalShellCacheObject],
) -> Vec<usize> {
    let mut snapshot = vec![objects.as_ptr() as usize, objects.len()];
    for object in objects {
        snapshot.extend([object.records.as_ptr() as usize, object.records.len()]);
        for cache in object.records.iter().flatten() {
            snapshot_paths(&mut snapshot, &cache.top_surfaces);
            snapshot_paths(&mut snapshot, &cache.bottom_surfaces);
            snapshot_paths(&mut snapshot, &cache.holes);
        }
    }
    snapshot
}

fn snapshot_paths(snapshot: &mut Vec<usize>, paths: &[crate::geometry::Polygon]) {
    snapshot.extend([paths.as_ptr() as usize, paths.len()]);
    for path in paths {
        snapshot.extend([path.points().as_ptr() as usize, path.points().len()]);
    }
}

pub(super) fn cache_point_buffers(
    objects: &[crate::project_slice::prepare_infill::vertical_shells::types::VerticalShellCacheObject],
) -> Vec<usize> {
    objects
        .iter()
        .flat_map(|object| object.records.iter().flatten())
        .flat_map(|cache| {
            cache
                .top_surfaces
                .iter()
                .chain(&cache.bottom_surfaces)
                .chain(&cache.holes)
        })
        .map(|path| path.points().as_ptr() as usize)
        .collect()
}

pub(in crate::project_slice::tests::prepare_infill) fn predecessor_snapshot(
    predecessor: &crate::project_slice::perimeters::classic::traversal::PreparedPostClassicTraversal,
) -> Vec<usize> {
    let mut snapshot = vec![
        predecessor.objects.as_ptr() as usize,
        predecessor.objects.len(),
    ];
    for traversal in &predecessor.objects {
        let prelude = &traversal.predecessor.predecessor.predecessor.predecessor;
        let (compensated, inputs) = prelude.object.as_parts();
        let (post_regions, lslices) = compensated.as_parts();
        let (plan, _, _) = post_regions.as_parts();
        snapshot.extend([
            inputs.as_ptr() as usize,
            inputs.len(),
            prelude.records.as_ptr() as usize,
            prelude.records.len(),
            plan.layers.as_ptr() as usize,
            plan.layers.len(),
            lslices.as_ptr() as usize,
            lslices.len(),
        ]);
        snapshot_lslices(&mut snapshot, lslices);
    }
    snapshot.extend(tree_allocation_snapshot(predecessor));
    snapshot
}

fn tree_allocation_snapshot(
    predecessor: &crate::project_slice::perimeters::classic::traversal::PreparedPostClassicTraversal,
) -> Vec<usize> {
    let mut snapshot = Vec::new();
    for object in &predecessor.objects {
        snapshot.extend([object.records.as_ptr() as usize, object.records.len()]);
        snapshot.extend([
            object.predecessor.records.as_ptr() as usize,
            object.predecessor.records.len(),
        ]);
    }
    for record in predecessor
        .objects
        .iter()
        .flat_map(|object| object.records.iter().flatten())
    {
        snapshot.extend([record.surfaces.as_ptr() as usize, record.surfaces.len()]);
    }
    for surface in predecessor
        .objects
        .iter()
        .flat_map(|object| object.records.iter().flatten())
        .flat_map(|record| &record.surfaces)
    {
        snapshot.extend([surface.roots.as_ptr() as usize, surface.roots.len()]);
        snapshot_traversal_seeds(&mut snapshot, &surface.roots);
    }
    for record in predecessor
        .objects
        .iter()
        .flat_map(|object| object.predecessor.records.iter().flatten())
    {
        snapshot.extend([record.surfaces.as_ptr() as usize, record.surfaces.len()]);
    }
    for surface in predecessor
        .objects
        .iter()
        .flat_map(|object| object.predecessor.records.iter().flatten())
        .flat_map(|record| &record.surfaces)
    {
        snapshot.extend([surface.roots.as_ptr() as usize, surface.roots.len()]);
        snapshot_hierarchy_loops(&mut snapshot, &surface.roots);
        snapshot_loop_buckets(&mut snapshot, &surface.remaining_contours);
        snapshot_loop_buckets(&mut snapshot, &surface.remaining_holes);
    }
    snapshot
}

fn snapshot_traversal_seeds(
    snapshot: &mut Vec<usize>,
    roots: &[crate::project_slice::perimeters::classic::traversal::TraversalSeed],
) {
    let mut pending = roots.iter().collect::<Vec<_>>();
    while let Some(seed) = pending.pop() {
        snapshot.extend([
            seed.polygon.points().as_ptr() as usize,
            seed.polygon.points().len(),
            seed.children.as_ptr() as usize,
            seed.children.len(),
        ]);
        pending.extend(seed.children.iter());
    }
}

fn snapshot_hierarchy_loops(
    snapshot: &mut Vec<usize>,
    roots: &[crate::project_slice::perimeters::classic::hierarchy::PerimeterGeneratorLoop],
) {
    let mut pending = roots.iter().collect::<Vec<_>>();
    while let Some(node) = pending.pop() {
        snapshot.extend([
            node.polygon.points().as_ptr() as usize,
            node.polygon.points().len(),
            node.children.as_ptr() as usize,
            node.children.len(),
        ]);
        pending.extend(node.children.iter());
    }
}

fn snapshot_loop_buckets(
    snapshot: &mut Vec<usize>,
    buckets: &[Vec<crate::project_slice::perimeters::classic::hierarchy::PerimeterGeneratorLoop>],
) {
    snapshot.extend([buckets.as_ptr() as usize, buckets.len()]);
    for roots in buckets {
        snapshot.extend([roots.as_ptr() as usize, roots.len()]);
        snapshot_hierarchy_loops(snapshot, roots);
    }
}

fn snapshot_lslices(snapshot: &mut Vec<usize>, lslices: &[Vec<crate::geometry::ExPolygon>]) {
    for layer in lslices {
        snapshot.extend([layer.as_ptr() as usize, layer.len()]);
        for expolygon in layer {
            snapshot_expolygon(snapshot, expolygon);
        }
    }
}

fn snapshot_expolygon(snapshot: &mut Vec<usize>, expolygon: &crate::geometry::ExPolygon) {
    snapshot.extend([
        expolygon.contour().points().as_ptr() as usize,
        expolygon.contour().points().len(),
        expolygon.holes().as_ptr() as usize,
        expolygon.holes().len(),
    ]);
    for hole in expolygon.holes() {
        snapshot.extend([hole.points().as_ptr() as usize, hole.points().len()]);
    }
}

pub(in crate::project_slice::tests::prepare_infill) fn lslice_point_buffers(
    predecessor: &crate::project_slice::perimeters::classic::traversal::PreparedPostClassicTraversal,
) -> Vec<usize> {
    predecessor
        .objects
        .iter()
        .flat_map(|traversal| {
            traversal
                .predecessor
                .predecessor
                .predecessor
                .predecessor
                .object
                .as_parts()
                .0
                .as_parts()
                .1
                .iter()
                .flatten()
        })
        .flat_map(|expolygon| std::iter::once(expolygon.contour()).chain(expolygon.holes().iter()))
        .map(|path| path.points().as_ptr() as usize)
        .collect()
}

pub(in crate::project_slice::tests::prepare_infill) fn predecessor_geometry_point_buffers(
    predecessor: &crate::project_slice::perimeters::classic::traversal::PreparedPostClassicTraversal,
) -> Vec<usize> {
    let mut points = lslice_point_buffers(predecessor);
    for seed in predecessor
        .objects
        .iter()
        .flat_map(|object| object.records.iter().flatten())
        .flat_map(|record| &record.surfaces)
        .flat_map(|surface| &surface.roots)
    {
        traversal_seed_point_buffers(&mut points, seed);
    }
    for surface in predecessor
        .objects
        .iter()
        .flat_map(|object| object.predecessor.records.iter().flatten())
        .flat_map(|record| &record.surfaces)
    {
        for root in &surface.roots {
            hierarchy_loop_point_buffers(&mut points, root);
        }
        for root in surface
            .remaining_contours
            .iter()
            .chain(&surface.remaining_holes)
            .flatten()
        {
            hierarchy_loop_point_buffers(&mut points, root);
        }
    }
    points
}

fn traversal_seed_point_buffers(
    points: &mut Vec<usize>,
    root: &crate::project_slice::perimeters::classic::traversal::TraversalSeed,
) {
    let mut pending = vec![root];
    while let Some(seed) = pending.pop() {
        points.push(seed.polygon.points().as_ptr() as usize);
        pending.extend(&seed.children);
    }
}

fn hierarchy_loop_point_buffers(
    points: &mut Vec<usize>,
    root: &crate::project_slice::perimeters::classic::hierarchy::PerimeterGeneratorLoop,
) {
    let mut pending = vec![root];
    while let Some(node) = pending.pop() {
        points.push(node.polygon.points().as_ptr() as usize);
        pending.extend(&node.children);
    }
}
