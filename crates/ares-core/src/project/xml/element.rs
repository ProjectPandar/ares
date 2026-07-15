use quick_xml::name::ResolveResult;

use crate::SliceError;

use super::{
    super::model_xml::{MATERIAL_NAMESPACE, PRODUCTION_NAMESPACE},
    XmlRole,
};

pub(super) fn validate(
    role: XmlRole,
    local_name: &[u8],
    namespace: &ResolveResult<'_>,
) -> Result<(), SliceError> {
    if role == XmlRole::Model {
        return validate_model(local_name, namespace);
    }

    let matches = match (role.namespace(), namespace) {
        (Some(expected), ResolveResult::Bound(actual)) => actual.as_ref() == expected,
        (None, ResolveResult::Unbound) => true,
        _ => false,
    };
    if matches {
        Ok(())
    } else {
        Err(invalid(role, "unexpected or unresolved XML namespace"))
    }
}

fn validate_model(local_name: &[u8], namespace: &ResolveResult<'_>) -> Result<(), SliceError> {
    let valid = match namespace {
        ResolveResult::Bound(namespace)
            if namespace.as_ref() == XmlRole::Model.namespace().unwrap() =>
        {
            matches!(
                local_name,
                b"model"
                    | b"metadata"
                    | b"resources"
                    | b"object"
                    | b"mesh"
                    | b"vertices"
                    | b"vertex"
                    | b"triangles"
                    | b"triangle"
                    | b"components"
                    | b"component"
                    | b"build"
                    | b"item"
            )
        }
        ResolveResult::Bound(namespace) if namespace.as_ref() == MATERIAL_NAMESPACE.as_bytes() => {
            matches!(local_name, b"colorgroup" | b"color")
        }
        ResolveResult::Bound(namespace)
            if namespace.as_ref() == PRODUCTION_NAMESPACE.as_bytes() =>
        {
            false
        }
        _ => false,
    };
    if valid {
        Ok(())
    } else {
        Err(invalid(
            XmlRole::Model,
            "element namespace does not match the fixed 3MF vocabulary",
        ))
    }
}

fn invalid(role: XmlRole, reason: &str) -> SliceError {
    SliceError::InvalidInput(format!("invalid project {} XML: {reason}", role.name()))
}
