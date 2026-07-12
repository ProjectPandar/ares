use quick_xml::name::ResolveResult;

use crate::SliceError;

use super::XmlRole;

pub(super) fn validate_namespace(
    role: XmlRole,
    qualified_name: &[u8],
    local_name: &[u8],
    namespace: &ResolveResult<'_>,
) -> Result<(), SliceError> {
    if qualified_name == b"xmlns" || qualified_name.starts_with(b"xmlns:") {
        return Ok(());
    }
    if role == XmlRole::Model {
        return super::model::validate_attribute_namespace(local_name, namespace);
    }

    let typed_attribute = match role {
        XmlRole::ContentTypes => {
            matches!(local_name, b"Extension" | b"ContentType" | b"PartName")
        }
        XmlRole::Relationships => matches!(local_name, b"Target" | b"Id" | b"Type"),
        XmlRole::ModelSettings => matches!(
            local_name,
            b"id"
                | b"key"
                | b"value"
                | b"subtype"
                | b"edges_fixed"
                | b"degenerate_facets"
                | b"facets_removed"
                | b"facets_reversed"
                | b"backwards_edges"
                | b"object_id"
                | b"instance_id"
                | b"transform"
                | b"offset"
        ),
        XmlRole::SliceInfo => matches!(local_name, b"key" | b"value"),
        XmlRole::Model => unreachable!(),
    };
    if typed_attribute && !matches!(namespace, ResolveResult::Unbound) {
        return Err(invalid(role));
    }
    Ok(())
}

fn invalid(role: XmlRole) -> SliceError {
    SliceError::InvalidInput(format!(
        "invalid project {} XML: typed attribute must be unprefixed",
        role.name()
    ))
}
