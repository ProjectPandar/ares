use quick_xml::name::ResolveResult;

use crate::SliceError;

use super::super::model_xml::{MATERIAL_NAMESPACE, PRODUCTION_NAMESPACE};

const XML_NAMESPACE: &[u8] = b"http://www.w3.org/XML/1998/namespace";

pub(super) fn validate_attribute_namespace(
    local_name: &[u8],
    namespace: &ResolveResult<'_>,
) -> Result<(), SliceError> {
    let production_attribute = matches!(local_name, b"path" | b"UUID" | b"uuid");
    let language_attribute = local_name == b"lang";
    let core_attribute = matches!(
        local_name,
        b"unit"
            | b"requiredextensions"
            | b"name"
            | b"id"
            | b"type"
            | b"objectid"
            | b"transform"
            | b"x"
            | b"y"
            | b"z"
            | b"v1"
            | b"v2"
            | b"v3"
            | b"printable"
            | b"auto_drop"
            | b"pid"
            | b"pindex"
            | b"color"
    );
    let valid = if production_attribute {
        matches!(namespace, ResolveResult::Bound(value) if value.as_ref() == PRODUCTION_NAMESPACE.as_bytes())
    } else if language_attribute {
        matches!(namespace, ResolveResult::Bound(value) if value.as_ref() == XML_NAMESPACE)
    } else if core_attribute {
        matches!(namespace, ResolveResult::Unbound)
    } else {
        false
    };
    if valid {
        Ok(())
    } else {
        Err(invalid(
            "attribute namespace does not match its 3MF meaning",
        ))
    }
}

pub(super) fn validate_required_extensions(
    required_extensions: Option<&str>,
    namespace_bindings: &[(Vec<u8>, String)],
) -> Result<(), SliceError> {
    let mut resolved_namespaces = Vec::new();
    for extension in required_extensions
        .unwrap_or_default()
        .split_ascii_whitespace()
    {
        let binding = namespace_bindings
            .iter()
            .find(|(prefix, _)| prefix.as_slice() == extension.as_bytes())
            .map(|(_, namespace)| namespace.as_str());
        let Some(namespace) = binding else {
            return Err(invalid("unsupported required extension"));
        };
        if !matches!(namespace, PRODUCTION_NAMESPACE | MATERIAL_NAMESPACE)
            || resolved_namespaces.contains(&namespace)
        {
            return Err(invalid("unsupported required extension"));
        }
        resolved_namespaces.push(namespace);
    }
    Ok(())
}

fn invalid(reason: impl std::fmt::Display) -> SliceError {
    SliceError::InvalidInput(format!("invalid project model XML: {reason}"))
}
