use std::{
    collections::{BTreeMap, BTreeSet, VecDeque},
    io::{Cursor, Write},
};

use zip::{CompressionMethod, ZipWriter, write::SimpleFileOptions};

use crate::{
    SliceError,
    project::{
        ArchiveLimits, PackagePath, ProjectArchive, ProjectModel, ProjectObject,
        content_types::ContentTypes,
        model_settings::ModelSettings,
        transform::Transform3d,
        xml::{XmlRole, deserialize_xml},
    },
};

use super::super::{graph, metadata};
use super::{
    EXPANDED_MODEL_LIMIT, ExpandedModelBudget, Pending, enqueue_occurrence,
    project_domain_with_budget,
};

const CONTENT_TYPES: &str = r#"<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/><Default Extension="model" ContentType="application/vnd.ms-package.3dmanufacturing-3dmodel+xml"/><Default Extension="png" ContentType="image/png"/></Types>"#;
const ROOT_MODEL_PATH: &str = "3D/root.model";
const KSR_MODEL_PATH: &str = "3D/3dmodel.model";
const KSR_FIXTURE: &[u8] =
    include_bytes!("../../../../../../tests/ksr_fdmtest_v4/ksr_fdmtest_v4.project.3mf");
type ExactPending = (PackagePath, u32, Transform3d);
type ExactQueueSeam = fn(
    &mut VecDeque<ExactPending>,
    &mut ExpandedModelBudget,
    ExactPending,
) -> Result<(), SliceError>;

fn mesh(id: u32) -> String {
    format!(
        r#"<object id="{id}" type="model"><mesh><vertices><vertex x="0" y="0" z="0"/><vertex x="1" y="0" z="0"/><vertex x="0" y="1" z="0"/></vertices><triangles><triangle v1="0" v2="1" v3="2"/></triangles></mesh></object>"#
    )
}

fn group(id: u32, children: impl IntoIterator<Item = u32>) -> String {
    let components = children
        .into_iter()
        .map(|child| format!(r#"<component objectid="{child}"/>"#))
        .collect::<String>();
    format!(r#"<object id="{id}" type="model"><components>{components}</components></object>"#)
}

fn model(resources: &str, build_ids: &[u32]) -> String {
    let build = build_ids
        .iter()
        .map(|id| format!(r#"<item objectid="{id}"/>"#))
        .collect::<String>();
    format!(
        r#"<model xmlns="http://schemas.microsoft.com/3dmanufacturing/core/2015/02"><resources>{resources}</resources><build>{build}</build></model>"#
    )
}

fn model_settings(build_ids: &[u32]) -> String {
    let mut instance_counts = BTreeMap::<u32, u32>::new();
    let instances = build_ids
        .iter()
        .enumerate()
        .map(|(index, object_id)| {
            let instance_id = instance_counts.entry(*object_id).or_default();
            let instance = format!(
                r#"<model_instance><metadata key="object_id" value="{object_id}"/><metadata key="instance_id" value="{}"/><metadata key="identify_id" value="{}"/></model_instance>"#,
                *instance_id,
                index + 1
            );
            *instance_id += 1;
            instance
        })
        .collect::<String>();
    format!(r#"<config><plate><metadata key="plater_id" value="1"/>{instances}</plate></config>"#)
}

fn synthetic_archive(resources: &str, build_ids: &[u32]) -> Vec<u8> {
    let entries = [
        ("[Content_Types].xml", CONTENT_TYPES.to_owned()),
        (ROOT_MODEL_PATH, model(resources, build_ids)),
        ("Metadata/model_settings.config", model_settings(build_ids)),
    ];
    let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
    let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
    for (path, text) in entries {
        writer.start_file(path, options).unwrap();
        writer.write_all(text.as_bytes()).unwrap();
    }
    writer.finish().unwrap().into_inner()
}

fn assemble_with_budget(
    input: &[u8],
    root_model_path: &str,
    budget: &mut ExpandedModelBudget,
) -> Result<(Vec<ProjectModel>, Vec<ProjectObject>), SliceError> {
    let mut archive = ProjectArchive::open(input, ArchiveLimits::PROJECT)?;
    let archive_paths = archive.paths().cloned().collect::<BTreeSet<_>>();
    let content_types_path = PackagePath::entry(b"[Content_Types].xml")?;
    let content_types: ContentTypes =
        deserialize_xml(&archive.read(&content_types_path)?, XmlRole::ContentTypes)?;
    content_types.validate_required()?;
    let graph = graph::load(
        &mut archive,
        &content_types,
        &archive_paths,
        PackagePath::entry(root_model_path.as_bytes())?,
    )?;
    let settings_path = PackagePath::entry(b"Metadata/model_settings.config")?;
    let settings: ModelSettings =
        deserialize_xml(&archive.read(&settings_path)?, XmlRole::ModelSettings)?;
    let metadata = metadata::index(&settings)?;
    project_domain_with_budget(&graph, &metadata, &settings, budget)
}

#[test]
fn task22b_expanded_model_budget_accepts_limit_rejects_next_and_overflow() {
    let expected = "project expanded model item count exceeds supported limit of 1000000";
    let _exact_queue_seam: ExactQueueSeam = enqueue_occurrence;
    let exact_pending = VecDeque::<Pending>::new();
    let _: VecDeque<ExactPending> = exact_pending;

    let empty = synthetic_archive(
        r#"<object id="1" type="model"><mesh><vertices/><triangles/></mesh></object>"#,
        &[1],
    );
    let mut at_limit = ExpandedModelBudget {
        used: EXPANDED_MODEL_LIMIT - 1,
    };
    assemble_with_budget(&empty, ROOT_MODEL_PATH, &mut at_limit).unwrap();
    assert_eq!(at_limit.used, EXPANDED_MODEL_LIMIT);

    let mut reject_next = ExpandedModelBudget {
        used: EXPANDED_MODEL_LIMIT,
    };
    assert_eq!(
        assemble_with_budget(&empty, ROOT_MODEL_PATH, &mut reject_next)
            .unwrap_err()
            .to_string(),
        expected
    );

    let mut pending = VecDeque::<Pending>::new();
    let mut reject_enqueue = ExpandedModelBudget {
        used: EXPANDED_MODEL_LIMIT,
    };
    assert_eq!(
        enqueue_occurrence(
            &mut pending,
            &mut reject_enqueue,
            (
                PackagePath::entry(ROOT_MODEL_PATH.as_bytes()).unwrap(),
                1,
                Transform3d::IDENTITY,
            ),
        )
        .unwrap_err()
        .to_string(),
        expected
    );
    assert_eq!(reject_enqueue.used, EXPANDED_MODEL_LIMIT);
    assert!(pending.is_empty());

    let mut overflow = ExpandedModelBudget { used: usize::MAX };
    assert_eq!(overflow.claim(1).unwrap_err().to_string(), expected);
    assert_eq!(
        ExpandedModelBudget::default()
            .claim_mesh(usize::MAX, 1)
            .unwrap_err()
            .to_string(),
        expected
    );
}

#[test]
fn task22b_component_expansion_is_ancestry_free_and_claims_before_queue_growth() {
    let mut resources = (1..=32).map(mesh).collect::<String>();
    for depth in 0..32_u32 {
        let id = 100 + depth;
        let children = if depth == 31 {
            (1..=32).collect::<Vec<_>>()
        } else {
            vec![id + 1]
        };
        resources.push_str(&group(id, children));
    }

    let input = synthetic_archive(&resources, &[100]);
    let mut budget = ExpandedModelBudget::default();
    let (_, objects) = assemble_with_budget(&input, ROOT_MODEL_PATH, &mut budget).unwrap();
    assert_eq!(
        objects[0]
            .volumes()
            .iter()
            .map(|volume| volume.id())
            .collect::<Vec<_>>(),
        (1..=32).collect::<Vec<_>>()
    );
    assert_eq!(budget.used, 32 + 32 + 32 * 3 + 32);
}

#[test]
fn task22b_expanded_model_budget_is_request_wide_instances_reuse_dag_and_ksr_claims_18345() {
    let two_roots = synthetic_archive(&format!("{}{}", mesh(1), mesh(2)), &[1, 2]);
    let mut shared = ExpandedModelBudget::default();
    let (_, objects) = assemble_with_budget(&two_roots, ROOT_MODEL_PATH, &mut shared).unwrap();
    assert_eq!(objects.len(), 2);
    assert_eq!(shared.used, 2 + 6 + 2);

    let repeated = synthetic_archive(&mesh(1), &[1, 1]);
    let mut reused = ExpandedModelBudget::default();
    let (_, objects) = assemble_with_budget(&repeated, ROOT_MODEL_PATH, &mut reused).unwrap();
    assert_eq!(objects[0].instances().len(), 2);
    assert_eq!(reused.used, 1 + 3 + 1);

    let mut ksr = ExpandedModelBudget::default();
    assemble_with_budget(KSR_FIXTURE, KSR_MODEL_PATH, &mut ksr).unwrap();
    assert_eq!(ksr.used, 2 + 6_109 + 12_234);
}
