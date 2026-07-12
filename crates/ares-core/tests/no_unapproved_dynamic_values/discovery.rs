use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};

use syn::{
    Item, ItemMacro, ItemMod, LitStr,
    parse::{Parse, ParseStream},
};
use walkdir::WalkDir;

use crate::{classification::skip, finding::normalize_path};

pub(super) const PRODUCTION_ROOTS: [&str; 3] = [
    "crates/ares-core/src/lib.rs",
    "crates/ares-cli/src/main.rs",
    "crates/ares-wasm/src/lib.rs",
];

pub(super) fn production_files(repo: &Path) -> Result<Vec<PathBuf>, String> {
    let mut sources = BTreeMap::new();
    for root in PRODUCTION_ROOTS {
        let source_dir = repo.join(root).parent().unwrap().to_path_buf();
        if !source_dir.exists() {
            return Err(format!(
                "missing production source directory: {}",
                source_dir.display()
            ));
        }
        for entry in WalkDir::new(source_dir) {
            let entry = entry.map_err(|error| format!("could not index Rust sources: {error}"))?;
            let path = entry.path();
            if entry.file_type().is_file() && path.extension().is_some_and(|ext| ext == "rs") {
                let relative = relative(repo, path)?;
                let source = fs::read_to_string(path)
                    .map_err(|error| format!("could not read {}: {error}", path.display()))?;
                sources.insert(relative, source);
            }
        }
    }
    production_sources(&sources)
        .map(|paths| paths.into_iter().map(|path| repo.join(path)).collect())
}

pub(super) fn production_sources(
    sources: &BTreeMap<String, String>,
) -> Result<Vec<String>, String> {
    let candidates = sources.keys().cloned().collect::<BTreeSet<_>>();
    for root in PRODUCTION_ROOTS {
        if !candidates.contains(root) {
            return Err(format!("missing production root: {root}"));
        }
    }
    let mut graph = Graph {
        sources,
        candidates,
        reachable: BTreeSet::new(),
        visited: BTreeSet::new(),
    };
    for root in PRODUCTION_ROOTS {
        graph.walk_file(root)?;
    }
    Ok(graph.reachable.into_iter().collect())
}

struct Graph<'a> {
    sources: &'a BTreeMap<String, String>,
    candidates: BTreeSet<String>,
    reachable: BTreeSet<String>,
    visited: BTreeSet<(String, String)>,
}

impl Graph<'_> {
    fn walk_file(&mut self, path: &str) -> Result<(), String> {
        self.walk_file_in(path, &module_directory(path))
    }

    fn walk_file_in(&mut self, path: &str, module_dir: &str) -> Result<(), String> {
        self.reachable.insert(path.to_owned());
        if !self
            .visited
            .insert((path.to_owned(), module_dir.to_owned()))
        {
            return Ok(());
        }
        let source = self
            .sources
            .get(path)
            .ok_or_else(|| format!("reachable Rust file is missing: {path}"))?;
        let file =
            syn::parse_file(source).map_err(|error| format!("could not parse {path}: {error}"))?;
        self.walk_items(&file.items, path, module_dir)
    }

    fn walk_items(
        &mut self,
        items: &[Item],
        current: &str,
        module_dir: &str,
    ) -> Result<(), String> {
        for item in items {
            if skip(item_attrs(item)) {
                continue;
            }
            match item {
                Item::Mod(module) => self.walk_module(module, current, module_dir)?,
                Item::Macro(item) if item.mac.path.is_ident("option_modules") => {
                    self.walk_option_modules(item, current, module_dir)?;
                }
                Item::Macro(item) if item.mac.path.is_ident("include") => {
                    self.walk_include(item, current, module_dir)?;
                }
                _ => {}
            }
        }
        Ok(())
    }

    fn walk_option_modules(
        &mut self,
        item: &ItemMacro,
        current: &str,
        module_dir: &str,
    ) -> Result<(), String> {
        let modules = syn::parse2::<ModuleList>(item.mac.tokens.clone())
            .map_err(|error| format!("invalid option_modules! in {current}: {error}"))?;
        for module in modules.0 {
            let child = resolve_module(module_dir, &module, &self.candidates)?;
            self.walk_file(&child)?;
        }
        Ok(())
    }

    fn walk_module(
        &mut self,
        module: &ItemMod,
        current: &str,
        module_dir: &str,
    ) -> Result<(), String> {
        if let Some((_, items)) = &module.content {
            return self.walk_items(items, current, &format!("{module_dir}/{}", module.ident));
        }
        if let Some(path) = path_attribute(module)? {
            let parent = Path::new(current).parent().unwrap();
            let child = clean_path(&parent.join(path));
            if !self.candidates.contains(&child) {
                return Err(format!("reachable #[path] module is missing: {child}"));
            }
            return self.walk_file(&child);
        }
        let child = resolve_module(module_dir, &module.ident.to_string(), &self.candidates)?;
        self.walk_file(&child)
    }

    fn walk_include(
        &mut self,
        item: &ItemMacro,
        current: &str,
        module_dir: &str,
    ) -> Result<(), String> {
        let literal = syn::parse2::<LitStr>(item.mac.tokens.clone())
            .map_err(|_| format!("nonliteral include! in reachable file: {current}"))?;
        if !literal.value().ends_with(".rs") {
            return Ok(());
        }
        let parent = Path::new(current).parent().unwrap();
        let child = clean_path(&parent.join(literal.value()));
        if !self.candidates.contains(&child) {
            return Err(format!("reachable include! is missing: {child}"));
        }
        self.walk_file_in(&child, module_dir)
    }
}

fn resolve_module(
    module_dir: &str,
    name: &str,
    candidates: &BTreeSet<String>,
) -> Result<String, String> {
    let flat = format!("{module_dir}/{name}.rs");
    let nested = format!("{module_dir}/{name}/mod.rs");
    match (candidates.contains(&flat), candidates.contains(&nested)) {
        (true, false) => Ok(flat),
        (false, true) => Ok(nested),
        (false, false) => Err(format!("reachable module is missing: {module_dir}/{name}")),
        (true, true) => Err(format!(
            "reachable module is ambiguous: {flat} and {nested}"
        )),
    }
}

fn module_directory(path: &str) -> String {
    let path = Path::new(path);
    let parent = normalize_path(&path.parent().unwrap().to_string_lossy());
    if path
        .file_name()
        .is_some_and(|name| name == "lib.rs" || name == "main.rs" || name == "mod.rs")
    {
        parent
    } else {
        format!("{parent}/{}", path.file_stem().unwrap().to_string_lossy())
    }
}

fn path_attribute(module: &ItemMod) -> Result<Option<String>, String> {
    module
        .attrs
        .iter()
        .find(|attr| attr.path().is_ident("path"))
        .map(|attr| {
            if let syn::Meta::NameValue(value) = &attr.meta
                && let syn::Expr::Lit(literal) = &value.value
                && let syn::Lit::Str(path) = &literal.lit
            {
                Ok(path.value())
            } else {
                Err(format!("invalid #[path] on module {}", module.ident))
            }
        })
        .transpose()
}

fn relative(repo: &Path, path: &Path) -> Result<String, String> {
    path.strip_prefix(repo)
        .map(clean_path)
        .map_err(|_| format!("{} is outside {}", path.display(), repo.display()))
}

fn clean_path(path: &Path) -> String {
    let mut parts = Vec::new();
    for component in path.components() {
        match component {
            std::path::Component::Normal(part) => parts.push(part.to_os_string()),
            std::path::Component::ParentDir => {
                parts.pop();
            }
            std::path::Component::CurDir => {}
            other => parts.push(other.as_os_str().to_os_string()),
        }
    }
    normalize_path(&parts.into_iter().collect::<PathBuf>().to_string_lossy())
}

struct ModuleList(Vec<String>);
impl Parse for ModuleList {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let mut modules = Vec::new();
        while !input.is_empty() {
            let _: syn::Visibility = input.parse()?;
            modules.push(input.parse::<syn::Ident>()?.to_string());
            if input.peek(syn::Token![,]) {
                let _: syn::Token![,] = input.parse()?;
            }
        }
        Ok(Self(modules))
    }
}

fn item_attrs(item: &Item) -> &[syn::Attribute] {
    match item {
        Item::Const(item) => &item.attrs,
        Item::Enum(item) => &item.attrs,
        Item::Fn(item) => &item.attrs,
        Item::Impl(item) => &item.attrs,
        Item::Macro(item) => &item.attrs,
        Item::Mod(item) => &item.attrs,
        Item::Static(item) => &item.attrs,
        Item::Struct(item) => &item.attrs,
        Item::Trait(item) => &item.attrs,
        Item::Type(item) => &item.attrs,
        Item::Use(item) => &item.attrs,
        _ => &[],
    }
}
