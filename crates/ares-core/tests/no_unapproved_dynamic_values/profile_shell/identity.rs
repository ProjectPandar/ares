use std::collections::{BTreeMap, BTreeSet};

use syn::{Item, Path, Type, Visibility};

use crate::{
    classification::skip,
    imports::{ImportResolver, UseBinding, source_scope},
};

#[derive(Clone)]
struct Alias {
    target: String,
    public: bool,
}

#[derive(Default)]
pub(super) struct ProfileIdentity {
    declarations: BTreeMap<String, bool>,
    aliases: BTreeMap<String, Alias>,
    scopes: BTreeSet<String>,
    uses: BTreeMap<String, Vec<UseBinding>>,
    owners: BTreeSet<String>,
    slice_options: Option<String>,
}

impl ProfileIdentity {
    pub(super) fn new(
        files: &[(&str, syn::File)],
        resolver: &ImportResolver,
        root: &[String],
    ) -> Self {
        let mut identity = Self::default();
        for (path, file) in files {
            collect(&file.items, &mut source_scope(path), &mut identity);
        }
        for scope in identity.scopes.clone() {
            identity.uses.insert(
                scope.clone(),
                resolver
                    .uses_in(&scope.split("::").map(str::to_owned).collect::<Vec<_>>())
                    .cloned()
                    .collect(),
            );
        }
        let owners = ["ProfileFragment", "MergedProfile", "ComposedProfile"]
            .into_iter()
            .filter_map(|name| identity.resolve(root, name, true, 0))
            .collect::<Vec<_>>();
        identity.owners.extend(owners);
        identity.slice_options = identity.resolve(root, "SliceOptions", true, 0);
        identity
    }

    pub(super) fn is_owner(&self, scope: &[String], ty: &Type) -> bool {
        self.resolve_type(scope, ty)
            .is_some_and(|owner| self.owners.contains(&owner))
    }

    pub(super) fn is_slice_type(&self, scope: &[String], ty: &Type) -> bool {
        self.resolve_type(scope, ty)
            .is_some_and(|owner| self.slice_options.as_deref() == Some(owner.as_str()))
    }

    pub(super) fn is_slice_path(&self, scope: &[String], path: &Path) -> bool {
        let parts = path
            .segments
            .iter()
            .map(|part| part.ident.to_string())
            .collect::<Vec<_>>();
        (1..=parts.len()).rev().any(|end| {
            self.resolve(scope, &parts[..end].join("::"), false, 0)
                .is_some_and(|owner| self.slice_options.as_deref() == Some(owner.as_str()))
        })
    }

    pub(super) fn slice_imports(&self, scope: &[String]) -> BTreeSet<String> {
        self.uses
            .get(&scope.join("::"))
            .into_iter()
            .flatten()
            .filter_map(|binding| binding.local.as_deref())
            .filter(|name| {
                self.resolve(scope, name, false, 0)
                    .is_some_and(|owner| self.slice_options.as_deref() == Some(owner.as_str()))
            })
            .map(str::to_owned)
            .collect()
    }

    fn resolve_type(&self, scope: &[String], ty: &Type) -> Option<String> {
        match ty {
            Type::Path(ty) if ty.qself.is_none() => {
                self.resolve(scope, &path_text(&ty.path), false, 0)
            }
            Type::Reference(ty) => self.resolve_type(scope, &ty.elem),
            Type::Slice(ty) => self.resolve_type(scope, &ty.elem),
            Type::Array(ty) => self.resolve_type(scope, &ty.elem),
            Type::Paren(ty) => self.resolve_type(scope, &ty.elem),
            Type::Group(ty) => self.resolve_type(scope, &ty.elem),
            _ => None,
        }
    }

    fn resolve(&self, scope: &[String], raw: &str, public_only: bool, depth: u8) -> Option<String> {
        if depth > 32 {
            return None;
        }
        let absolute = absolute_path(scope, raw)?;
        let candidate = absolute.join("::");
        if let Some(public) = self.declarations.get(&candidate) {
            return (!public_only || *public).then_some(candidate);
        }
        if let Some(alias) = self.aliases.get(&candidate) {
            if public_only && !alias.public {
                return None;
            }
            return self.resolve(
                &absolute[..absolute.len() - 1],
                &alias.target,
                public_only,
                depth + 1,
            );
        }
        let split = (1..absolute.len())
            .rev()
            .find(|split| self.scopes.contains(&absolute[..*split].join("::")))?;
        let (base, parts) = absolute.split_at(split);
        for binding in self
            .uses
            .get(&base.join("::"))
            .into_iter()
            .flatten()
            .filter(|binding| {
                (!public_only || binding.public)
                    && (binding.glob
                        || binding.local.as_deref() == parts.first().map(String::as_str))
            })
        {
            let suffix = if binding.glob { parts } else { &parts[1..] };
            let target = [binding.target.as_str(), &suffix.join("::")]
                .into_iter()
                .filter(|part| !part.is_empty())
                .collect::<Vec<_>>()
                .join("::");
            if let Some(owner) = self.resolve(&binding.scope, &target, public_only, depth + 1) {
                return Some(owner);
            }
        }
        None
    }
}

fn collect(items: &[Item], scope: &mut Vec<String>, identity: &mut ProfileIdentity) {
    identity.scopes.insert(scope.join("::"));
    for item in items {
        match item {
            Item::Struct(item) if !skip(&item.attrs) => {
                identity.declarations.insert(
                    symbol(scope, &item.ident.to_string()),
                    matches!(item.vis, Visibility::Public(_)),
                );
            }
            Item::Enum(item) if !skip(&item.attrs) => {
                identity.declarations.insert(
                    symbol(scope, &item.ident.to_string()),
                    matches!(item.vis, Visibility::Public(_)),
                );
            }
            Item::Type(item) if !skip(&item.attrs) => {
                if let Type::Path(target) = item.ty.as_ref() {
                    identity.aliases.insert(
                        symbol(scope, &item.ident.to_string()),
                        Alias {
                            target: path_text(&target.path),
                            public: matches!(item.vis, Visibility::Public(_)),
                        },
                    );
                }
            }
            Item::Mod(item) if item.content.is_some() && !skip(&item.attrs) => {
                scope.push(item.ident.to_string());
                collect(&item.content.as_ref().unwrap().1, scope, identity);
                scope.pop();
            }
            _ => {}
        }
    }
}

fn absolute_path(scope: &[String], raw: &str) -> Option<Vec<String>> {
    let mut path = scope.to_vec();
    for part in raw.split("::") {
        match part {
            "crate" => path.truncate(1),
            "self" => {}
            "super" => {
                path.pop()?;
            }
            part => path.push(part.to_owned()),
        }
    }
    Some(path)
}

fn path_text(path: &Path) -> String {
    path.segments
        .iter()
        .map(|part| part.ident.to_string())
        .collect::<Vec<_>>()
        .join("::")
}

fn symbol(scope: &[String], name: &str) -> String {
    format!("{}::{name}", scope.join("::"))
}

#[cfg(test)]
mod tests {
    use super::*;

    const LIB: &str = "pub struct ProfileFragment; pub struct SliceOptions; mod profiles; pub use profiles::ComposedProfile;";
    const PROFILES: &str = "pub mod composition; pub use composition::ComposedProfile;";
    const COMPOSITION: &str = r#"
        pub struct ComposedProfile;
        pub mod child {}
        pub mod deep { pub mod nested {} }
        pub mod named { use super::ComposedProfile; }
        pub mod renamed { use super::ComposedProfile as Profile; }
        pub mod globbed { use super::*; }
        pub mod exported { pub use super::ComposedProfile as Profile; }
        mod private_nested { use super::*; struct ComposedProfile; }
        mod private_public { use super::*; pub struct ComposedProfile; }
    "#;

    fn project(extra: &[(&str, &str)]) -> ProfileIdentity {
        let parsed = [
            ("crates/aaa/src/lib.rs", "pub struct SliceOptions;"),
            ("crates/ares-core/src/lib.rs", LIB),
            ("crates/ares-core/src/profiles/mod.rs", PROFILES),
            ("crates/ares-core/src/profiles/composition.rs", COMPOSITION),
        ]
        .into_iter()
        .chain(extra.iter().copied())
        .map(|(path, source)| (path, syn::parse_file(source).unwrap()))
        .collect::<Vec<_>>();
        let resolver = ImportResolver::project(parsed.iter().map(|(path, file)| (*path, file)));
        ProfileIdentity::new(
            &parsed,
            &resolver,
            &source_scope("crates/ares-core/src/lib.rs"),
        )
    }

    fn scope(path: &str, inline: &[&str]) -> Vec<String> {
        let mut scope = source_scope(path);
        scope.extend(inline.iter().map(|part| (*part).to_owned()));
        scope
    }

    fn ty(source: &str) -> Type {
        syn::parse_str(source).unwrap()
    }

    #[test]
    fn fixed_owner_paths_resolve_by_symbol_identity() {
        let identity = project(&[]);
        let composition = scope("crates/ares-core/src/profiles/composition.rs", &[]);
        let cases = [
            (
                scope("crates/ares-core/src/profiles/child.rs", &[]),
                "crate::ProfileFragment",
            ),
            (composition.clone(), "self::ComposedProfile"),
            (
                scope(
                    "crates/ares-core/src/profiles/composition.rs",
                    &["deep", "nested"],
                ),
                "super::super::ComposedProfile",
            ),
            (
                scope("crates/ares-core/src/profiles/composition.rs", &["named"]),
                "ComposedProfile",
            ),
            (
                scope("crates/ares-core/src/profiles/composition.rs", &["renamed"]),
                "Profile",
            ),
            (
                scope("crates/ares-core/src/profiles/composition.rs", &["globbed"]),
                "ComposedProfile",
            ),
            (composition, "crate::ComposedProfile"),
            (
                scope("crates/ares-core/src/profiles/composition.rs", &["child"]),
                "super::exported::Profile",
            ),
        ];
        for (scope, owner) in cases {
            assert!(identity.is_owner(&scope, &ty(owner)), "{scope:?}: {owner}");
        }
    }

    #[test]
    fn local_and_unexported_same_name_declarations_are_not_fixed_owners() {
        let identity = project(&[]);
        for module in ["private_nested", "private_public"] {
            let scope = scope("crates/ares-core/src/profiles/composition.rs", &[module]);
            assert!(
                !identity.is_owner(&scope, &ty("ComposedProfile")),
                "{module}"
            );
        }
    }

    #[test]
    fn slice_options_aliases_ignore_unrelated_workspace_root() {
        let path = "crates/ares-core/src/profiles/aliases.rs";
        let identity = project(&[(
            path,
            "mod aliases { pub type GlobConfig = crate::SliceOptions; } use crate::SliceOptions as NamedConfig; use aliases::*;",
        )]);
        let scope = scope(path, &[]);
        assert!(identity.is_slice_type(&scope, &ty("NamedConfig")));
        assert!(identity.is_slice_type(&scope, &ty("GlobConfig")));
    }
}
