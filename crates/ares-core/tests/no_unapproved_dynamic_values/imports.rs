use std::collections::{BTreeMap, BTreeSet};

use syn::{Item, ItemMod, ItemType, ItemUse, Path, Type, UseTree, Visibility};

use crate::classification::skip;

#[derive(Clone, Debug)]
pub(super) struct UseBinding {
    pub scope: Vec<String>,
    pub target: String,
    pub local: Option<String>,
    pub glob: bool,
    pub public: bool,
}

#[derive(Clone, Debug)]
struct Binding {
    origin: Vec<String>,
    target: String,
}

#[derive(Default)]
struct Scope {
    bindings: BTreeMap<String, Binding>,
    globs: Vec<Binding>,
}

#[derive(Default)]
pub(super) struct ImportResolver {
    scopes: BTreeMap<String, Scope>,
    uses: Vec<UseBinding>,
    resolved: BTreeMap<String, String>,
}

impl ImportResolver {
    pub fn new(file: &syn::File) -> Self {
        let mut resolver = Self::default();
        resolver.collect_items(&file.items, &mut Vec::new());
        resolver.resolve_project_exports();
        resolver
    }

    pub fn project<'a>(files: impl IntoIterator<Item = (&'a str, &'a syn::File)>) -> Self {
        let mut resolver = Self::default();
        for (path, file) in files {
            resolver.collect_items(&file.items, &mut source_scope(path));
        }
        resolver.resolve_project_exports();
        resolver
    }

    pub fn uses_in<'a>(&'a self, scope: &'a [String]) -> impl Iterator<Item = &'a UseBinding> {
        self.uses
            .iter()
            .filter(move |binding| binding.scope == scope)
    }

    pub fn resolve_path(&self, scope: &[String], path: &Path) -> String {
        let raw = path
            .segments
            .iter()
            .map(|segment| segment.ident.to_string())
            .collect::<Vec<_>>()
            .join("::");
        self.resolve_known(scope, &raw, 0).unwrap_or(raw)
    }

    pub fn resolve_use(&self, binding: &UseBinding) -> Option<String> {
        if binding.glob && external_root(&binding.target) {
            return Some(binding.target.clone());
        }
        self.resolve_known(&binding.scope, &binding.target, 0)
    }

    fn collect_items(&mut self, items: &[Item], scope: &mut Vec<String>) {
        self.scopes.entry(key(scope)).or_default();
        for item in items.iter().filter(|item| !skip(item_attrs(item))) {
            match item {
                Item::Use(item_use) => self.collect_use(item_use, scope),
                Item::Type(item_type) => self.collect_alias(item_type, scope),
                Item::Mod(item_mod) => self.collect_module(item_mod, scope),
                _ => {}
            }
        }
    }

    fn collect_use(&mut self, item: &ItemUse, scope: &[String]) {
        let public = !matches!(item.vis, Visibility::Inherited);
        let mut flattened = Vec::new();
        flatten_use(&item.tree, Vec::new(), &mut flattened);
        for (target, local, glob) in flattened {
            let binding = UseBinding {
                scope: scope.to_vec(),
                target: target.clone(),
                local: local.clone(),
                glob,
                public,
            };
            let internal = Binding {
                origin: scope.to_vec(),
                target,
            };
            let current = self.scopes.entry(key(scope)).or_default();
            if glob {
                current.globs.push(internal);
            } else if let Some(local) = local {
                current.bindings.insert(local, internal);
            }
            self.uses.push(binding);
        }
    }

    fn collect_alias(&mut self, item: &ItemType, scope: &[String]) {
        if let Type::Path(target) = item.ty.as_ref() {
            self.scopes.entry(key(scope)).or_default().bindings.insert(
                item.ident.to_string(),
                Binding {
                    origin: scope.to_vec(),
                    target: path_text(&target.path),
                },
            );
        }
    }

    fn collect_module(&mut self, item: &ItemMod, scope: &mut Vec<String>) {
        let Some((_, items)) = &item.content else {
            return;
        };
        scope.push(item.ident.to_string());
        self.collect_items(items, scope);
        scope.pop();
    }

    fn resolve_project_exports(&mut self) {
        loop {
            let unresolved = self
                .scopes
                .values()
                .flat_map(|scope| &scope.bindings)
                .map(|(local, binding)| (symbol(&binding.origin, local), binding))
                .filter(|(symbol, _)| !self.resolved.contains_key(symbol));
            let additions = unresolved
                .filter_map(|(symbol, binding)| {
                    self.resolve_known(&binding.origin, &binding.target, 0)
                        .map(|target| (symbol, target))
                })
                .collect::<Vec<_>>();
            if additions.is_empty() {
                break;
            }
            self.resolved.extend(additions);
        }
    }

    fn resolve_known(&self, scope: &[String], raw: &str, depth: usize) -> Option<String> {
        if depth > 32 {
            return None;
        }
        let raw = raw.trim_start_matches("::");
        if external_root(raw) {
            return Some(canonical_external(raw));
        }
        if let Some((target_scope, rest)) = self.internal_target(scope, raw) {
            return self.resolve_known(&target_scope, &rest, depth + 1);
        }
        let (first, suffix) = raw.split_once("::").unwrap_or((raw, ""));
        let current = self.scopes.get(&key(scope));
        if current
            .and_then(|scope| scope.bindings.get(first))
            .is_some()
        {
            let target = self.resolved.get(&symbol(scope, first))?;
            return Some(canonical_external(&append(target, suffix)));
        }
        for module in module_candidates(scope, first) {
            if self.scopes.contains_key(&key(&module)) {
                return self.resolve_known(&module, suffix, depth + 1);
            }
        }
        let mut candidates = BTreeSet::new();
        for glob in current.into_iter().flat_map(|scope| &scope.globs) {
            let candidate = append(&glob.target, raw);
            if let Some(resolved) = self.resolve_known(&glob.origin, &candidate, depth + 1)
                && allowed_glob_export(&resolved)
            {
                candidates.insert(resolved);
            }
        }
        (candidates.len() == 1).then(|| candidates.pop_first().unwrap())
    }

    fn internal_target(&self, scope: &[String], raw: &str) -> Option<(Vec<String>, String)> {
        let parts = raw.split("::").collect::<Vec<_>>();
        let mut target = match parts.first().copied()? {
            "crate" => project_root(scope),
            "self" => scope.to_vec(),
            "super" => {
                let mut parent = scope.to_vec();
                if parent.len() > project_root(scope).len() {
                    parent.pop();
                }
                parent
            }
            _ => return None,
        };
        let mut index = 1;
        while index < parts.len() {
            let mut candidate = target.clone();
            candidate.push(parts[index].to_owned());
            if !self.scopes.contains_key(&key(&candidate)) {
                break;
            }
            target = candidate;
            index += 1;
        }
        (index < parts.len()).then(|| (target, parts[index..].join("::")))
    }
}

fn flatten_use(
    tree: &UseTree,
    mut prefix: Vec<String>,
    output: &mut Vec<(String, Option<String>, bool)>,
) {
    match tree {
        UseTree::Path(path) => {
            prefix.push(path.ident.to_string());
            flatten_use(&path.tree, prefix, output);
        }
        UseTree::Name(name) if name.ident == "self" => {
            output.push((prefix.join("::"), prefix.last().cloned(), false));
        }
        UseTree::Name(name) => {
            prefix.push(name.ident.to_string());
            output.push((prefix.join("::"), Some(name.ident.to_string()), false));
        }
        UseTree::Rename(rename) => {
            if rename.ident != "self" {
                prefix.push(rename.ident.to_string());
            }
            output.push((prefix.join("::"), Some(rename.rename.to_string()), false));
        }
        UseTree::Glob(_) => output.push((prefix.join("::"), None, true)),
        UseTree::Group(group) => {
            for item in &group.items {
                flatten_use(item, prefix.clone(), output);
            }
        }
    }
}

fn module_candidates(scope: &[String], first: &str) -> [Vec<String>; 2] {
    let mut local = scope.to_vec();
    local.push(first.to_owned());
    let mut root = project_root(scope);
    root.push(first.to_owned());
    [local, root]
}

fn external_root(path: &str) -> bool {
    [
        "serde_json",
        "roxmltree",
        "xmltree",
        "simd_json",
        "json",
        "sonic_rs",
        "std",
        "core",
        "erased_serde",
    ]
    .iter()
    .any(|root| path == *root || path.starts_with(&format!("{root}::")))
}

fn canonical_external(path: &str) -> String {
    match path {
        "serde_json::value::Value" => "serde_json::Value".to_owned(),
        "serde_json::map::Map" => "serde_json::Map".to_owned(),
        "serde_json::RawValue" => "serde_json::value::RawValue".to_owned(),
        other => other.to_owned(),
    }
}

fn allowed_glob_export(path: &str) -> bool {
    matches!(
        path,
        "serde_json::Value"
            | "serde_json::Map"
            | "serde_json::value::RawValue"
            | "serde_json::from_value"
            | "serde_json::json"
    ) || path.split("::").any(|part| {
        matches!(
            part,
            "ConfigValue" | "DynamicValue" | "ErasedValue" | "ErasedPayload"
        )
    })
}

fn append(left: &str, right: &str) -> String {
    if right.is_empty() {
        left.to_owned()
    } else {
        format!("{left}::{right}")
    }
}

fn path_text(path: &Path) -> String {
    path.segments
        .iter()
        .map(|segment| segment.ident.to_string())
        .collect::<Vec<_>>()
        .join("::")
}

fn key(scope: &[String]) -> String {
    scope.join("::")
}

fn symbol(scope: &[String], local: &str) -> String {
    append(&key(scope), local)
}

fn project_root(scope: &[String]) -> Vec<String> {
    scope
        .first()
        .filter(|part| part.starts_with('@'))
        .cloned()
        .into_iter()
        .collect()
}

pub(super) fn source_scope(path: &str) -> Vec<String> {
    let path = path.replace('\\', "/");
    let Some((crate_path, relative)) = path.split_once("/src/") else {
        return Vec::new();
    };
    let mut parts = relative.split('/').collect::<Vec<_>>();
    let file = parts.pop().unwrap();
    let mut scope = vec![format!("@{crate_path}")];
    scope.extend(parts.into_iter().map(str::to_owned));
    let stem = file.strip_suffix(".rs").unwrap_or(file);
    if !matches!(stem, "lib" | "main" | "mod") {
        scope.push(stem.to_owned());
    }
    scope
}

fn item_attrs(item: &Item) -> &[syn::Attribute] {
    match item {
        Item::Const(item) => &item.attrs,
        Item::Enum(item) => &item.attrs,
        Item::ExternCrate(item) => &item.attrs,
        Item::Fn(item) => &item.attrs,
        Item::ForeignMod(item) => &item.attrs,
        Item::Impl(item) => &item.attrs,
        Item::Macro(item) => &item.attrs,
        Item::Mod(item) => &item.attrs,
        Item::Static(item) => &item.attrs,
        Item::Struct(item) => &item.attrs,
        Item::Trait(item) => &item.attrs,
        Item::TraitAlias(item) => &item.attrs,
        Item::Type(item) => &item.attrs,
        Item::Union(item) => &item.attrs,
        Item::Use(item) => &item.attrs,
        _ => &[],
    }
}
