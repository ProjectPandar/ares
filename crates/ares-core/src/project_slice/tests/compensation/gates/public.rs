use crate::project_slice::tests::support::{KsrArchive, metadata};
use crate::slice_project;

#[tokio::test]
async fn task22m_flow_public_raft_slices_instead_of_gating() {
    for raft_layers in ["1", "2"] {
        let mut archive = KsrArchive::new();
        archive.replace(
            "Metadata/project_settings.config",
            "\t\"raft_layers\": \"0\",",
            &format!("\t\"raft_layers\": \"{raft_layers}\","),
        );
        let result = slice_project(archive.bytes(), metadata(), crate::SlicerContext::new(0)).await;
        assert!(result.is_ok());
    }
}
