use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct Finding {
    pub path: String,
    pub owner: String,
    pub kind: &'static str,
    pub detail: String,
}

impl Finding {
    pub fn new(
        path: impl Into<String>,
        owner: impl Into<String>,
        kind: &'static str,
        detail: impl Into<String>,
    ) -> Self {
        Self {
            path: normalize_path(&path.into()),
            owner: owner.into(),
            kind,
            detail: detail.into(),
        }
    }

    pub fn field_owner(&self, containing_struct: &str, field: &str) -> bool {
        self.owner == format!("{containing_struct}.{field}") && self.kind == "type"
    }
}

pub(super) fn render_findings(findings: &[Finding]) -> BTreeSet<String> {
    let mut ordinals = BTreeMap::<(&str, &str, &str, &str), usize>::new();
    findings
        .iter()
        .map(|finding| {
            let key = (
                finding.path.as_str(),
                finding.owner.as_str(),
                finding.kind,
                finding.detail.as_str(),
            );
            let ordinal = ordinals
                .entry(key)
                .and_modify(|value| *value += 1)
                .or_insert(1);
            format!(
                "{}#{}@{}|{}|{}",
                finding.path, finding.owner, ordinal, finding.kind, finding.detail
            )
        })
        .collect()
}

pub(super) fn normalize_path(path: &str) -> String {
    path.replace('\\', "/")
}

pub(super) fn associated_owner(
    container: Option<&str>,
    fallback: &str,
    item: &syn::Ident,
) -> String {
    format!("{}::{item}", container.unwrap_or(fallback))
}

pub(super) fn module_owner(scope: &[String]) -> String {
    if scope.is_empty() {
        "crate".to_owned()
    } else {
        scope.join("::")
    }
}

pub(super) fn qualified(scope: &[String], name: &str) -> String {
    if scope.is_empty() {
        name.to_owned()
    } else {
        format!("{}::{name}", scope.join("::"))
    }
}
