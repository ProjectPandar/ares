#![cfg(all(feature = "parallel", not(target_arch = "wasm32")))]

#[path = "../../../tests/support/direct_catalog.rs"]
mod direct_catalog;
#[path = "gcode_thumbnail_images/project.rs"]
mod project;

use ares_core::{ConfigOverrides, GenerationMetadata, SlicerContext, ThumbnailData};

fn slice(input: &[u8], workers: usize, render: bool) -> Vec<u8> {
    slice_plate(input, workers, None, || {
        SlicerContext::new(0).with_render_thumbnails(render)
    })
}

fn slice_plate(
    input: &[u8],
    workers: usize,
    plate: Option<u32>,
    context: impl FnOnce() -> SlicerContext<'static> + Send,
) -> Vec<u8> {
    let catalog = direct_catalog::catalog();
    let runtime = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();
    rayon::ThreadPoolBuilder::new()
        .num_threads(workers)
        .build()
        .unwrap()
        .install(|| {
            runtime.block_on(ares_core::slice_project_plate_with_catalog(
                input,
                GenerationMetadata::default(),
                (plate, &ConfigOverrides::default()),
                Some(&catalog),
                context(),
            ))
        })
        .unwrap()
}

#[test]
fn rendered_thumbnail_pixels_and_complete_stream_are_independent_of_worker_count() {
    let input = project::make(project::ENDER3, "31.6x23.6/PNG, 32x24/JPG, 31.6x23.6/PNG");
    let single = slice(&input, 1, true);
    let text = std::str::from_utf8(&single).unwrap();
    let headers: Vec<_> = text
        .lines()
        .filter_map(|line| line.strip_prefix("; thumbnail"))
        .filter_map(|line| line.rsplit_once(' '))
        .filter_map(|(header, length)| (length.parse::<usize>().ok()? > 0).then_some(header))
        .collect();
    assert_eq!(
        headers,
        [" begin 32x24", "_JPG begin 32x24", " begin 32x24"]
    );
    let mut segments = text.split("; THUMBNAIL_BLOCK_START\n");
    let mut without_images = segments.next().unwrap().to_owned();
    for segment in segments {
        without_images.push_str(segment.split_once("; THUMBNAIL_BLOCK_END\n\n").unwrap().1);
    }
    assert!(without_images.into_bytes() == slice(&input, 1, false));
    // Lossless PNG blocks in the complete stream also establish pixel equality.
    for workers in [2, 7, 8, 16] {
        assert!(
            slice(&input, workers, true) == single,
            "output changed with {workers} workers"
        );
    }
}

#[test]
fn rendering_leaves_the_complete_stream_unchanged_without_definitions_or_on_bbl() {
    for (fixture, definitions) in [(project::ENDER3, ""), (project::BBL, "32x24/PNG")] {
        let input = project::make(fixture, definitions);
        let disabled = slice(&input, 1, false);
        assert!(slice(&input, 1, true) == disabled);
        assert!(
            !std::str::from_utf8(&disabled)
                .unwrap()
                .contains("; THUMBNAIL_BLOCK_START")
        );
    }
}

#[test]
fn rendering_fits_the_selected_plates_distinct_geometry() {
    let input = project::make(
        include_bytes!("../../../tests/parity/shared-mesh-centering/plate-local.3mf"),
        "64x48/PNG",
    );
    let image = |plate| {
        let gcode = String::from_utf8(slice_plate(&input, 1, Some(plate), || {
            SlicerContext::new(0).with_render_thumbnails(true)
        }))
        .unwrap();
        let (_, block) = gcode.split_once("; thumbnail begin 64x48 ").unwrap();
        block.split_once("; thumbnail end").unwrap().0.to_owned()
    };
    // The two plates contain solids of different aspect ratios.
    assert_ne!(image(1), image(2));
}

#[test]
fn caller_supplied_thumbnail_takes_precedence_over_rendering() {
    let input = project::make(project::ENDER3, "4x4/PNG");
    let slice = |render| {
        let image = ThumbnailData::new(4, 4, [0, 120, 0, 255].repeat(16)).unwrap();
        slice_plate(&input, 1, None, || {
            SlicerContext::new(0)
                .with_render_thumbnails(render)
                .with_thumbnail_images(vec![image])
        })
    };
    let supplied = slice(false);
    assert!(
        std::str::from_utf8(&supplied)
            .unwrap()
            .contains("; thumbnail begin 4x4 ")
    );
    assert!(supplied == slice(true));
}
