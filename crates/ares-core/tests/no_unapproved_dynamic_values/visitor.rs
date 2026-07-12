use std::collections::BTreeSet;

use syn::{
    ExprCall, ExprMethodCall, ExprPath, Field, Fields, ImplItemConst, ImplItemFn, ImplItemType,
    ItemConst, ItemEnum, ItemFn, ItemImpl, ItemMod, ItemStatic, ItemStruct, ItemTrait, ItemType,
    Macro, TraitItemConst, TraitItemFn, TraitItemType, Type,
    visit::{self, Visit},
};

use crate::{
    classification::{
        canonical_call, canonical_dynamic_path, canonical_macro, dynamic_type, render_type,
        runtime_method, skip,
    },
    finding::{Finding, associated_owner, module_owner, qualified, render_findings},
    imports::{ImportResolver, UseBinding, source_scope},
};

pub(super) fn scan_source(path: &str, source: &str) -> Result<Vec<Finding>, String> {
    let file =
        syn::parse_file(source).map_err(|error| format!("could not parse {path}: {error}"))?;
    let resolver = ImportResolver::new(&file);
    scan_file(path, &file, &resolver, Vec::new())
}
fn scan_file<'a>(
    path: &'a str,
    file: &syn::File,
    resolver: &'a ImportResolver,
    scope: Vec<String>,
) -> Result<Vec<Finding>, String> {
    let base_len = scope.len();
    let mut scanner = Scanner {
        path,
        resolver,
        scope,
        base_len,
        owner: "crate".to_owned(),
        impl_name: None,
        trait_name: None,
        emitted_uses: BTreeSet::new(),
        findings: Vec::new(),
        error: None,
    };
    scanner.emit_uses();
    scanner.visit_file(file);
    match scanner.error {
        Some(error) => Err(format!("could not canonicalize {path}: {error}")),
        None => Ok(scanner.findings),
    }
}
pub(super) fn scan_sources<'a>(
    sources: impl IntoIterator<Item = (&'a str, &'a str)>,
) -> Result<Vec<Finding>, String> {
    let files = sources
        .into_iter()
        .map(|(path, source)| {
            syn::parse_file(source)
                .map(|file| (path, file))
                .map_err(|error| format!("could not parse {path}: {error}"))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let resolver = ImportResolver::project(files.iter().map(|(path, file)| (*path, file)));
    let mut findings = Vec::new();
    for (path, file) in &files {
        findings.extend(scan_file(path, file, &resolver, source_scope(path))?);
    }
    Ok(findings)
}
pub(super) fn fingerprints(path: &str, source: &str) -> Result<BTreeSet<String>, String> {
    scan_source(path, source).map(|findings| render_findings(&findings))
}

struct Scanner<'a> {
    path: &'a str,
    resolver: &'a ImportResolver,
    scope: Vec<String>,
    base_len: usize,
    owner: String,
    impl_name: Option<String>,
    trait_name: Option<String>,
    emitted_uses: BTreeSet<String>,
    findings: Vec<Finding>,
    error: Option<String>,
}

impl Scanner<'_> {
    fn local_scope(&self) -> &[String] {
        &self.scope[self.base_len..]
    }
    fn push(&mut self, kind: &'static str, detail: impl Into<String>) {
        self.findings
            .push(Finding::new(self.path, self.owner.clone(), kind, detail));
    }
    fn set_owner(&mut self, owner: String) -> String {
        std::mem::replace(&mut self.owner, owner)
    }
    fn canonical_type(&mut self, ty: &Type) -> Option<String> {
        match render_type(ty, self.resolver, &self.scope) {
            Ok(rendered) => Some(rendered),
            Err(error) => {
                self.error.get_or_insert(error);
                None
            }
        }
    }
    fn resolved_use(&self, binding: &UseBinding) -> Option<(&'static str, String)> {
        let resolved = self.resolver.resolve_use(binding)?;
        let detail = if binding.glob {
            match resolved.as_str() {
                "serde_json" | "serde_json::value" | "serde_json::map" => {
                    format!("{resolved}::*")
                }
                _ => return None,
            }
        } else {
            let canonical = canonical_dynamic_path(&resolved)?;
            let original_name = resolved.rsplit("::").next().unwrap();
            match binding.local.as_deref() {
                Some(local) if local != original_name => format!("{canonical} as {local}"),
                _ => canonical,
            }
        };
        let kind = if binding.public { "reexport" } else { "use" };
        Some((kind, detail))
    }
    fn emit_uses(&mut self) {
        let scope_key = self.scope.join("::");
        if !self.emitted_uses.insert(scope_key) {
            return;
        }
        let old = self.set_owner(module_owner(self.local_scope()));
        let uses = self
            .resolver
            .uses_in(&self.scope)
            .cloned()
            .collect::<Vec<_>>();
        for binding in &uses {
            let Some((kind, detail)) = self.resolved_use(binding) else {
                continue;
            };
            self.push(kind, detail);
        }
        self.owner = old;
    }
    fn visit_field(&mut self, field: &Field, owner: String) {
        if skip(&field.attrs) {
            return;
        }
        let old = self.set_owner(owner);
        self.visit_type(&field.ty);
        self.owner = old;
    }
    fn visit_variant_fields(&mut self, fields: &Fields, owner: &str) {
        match fields {
            Fields::Named(fields) => {
                for field in &fields.named {
                    self.visit_field(field, owner.to_owned());
                }
            }
            Fields::Unnamed(fields) => {
                for field in &fields.unnamed {
                    self.visit_field(field, owner.to_owned());
                }
            }
            Fields::Unit => {}
        }
    }
    fn emit_call_path(&mut self, function: &ExprPath) {
        let path = self.resolver.resolve_path(&self.scope, &function.path);
        if let Some(call) = canonical_call(&path) {
            self.push("call", call);
        }
        let Some(qself) = &function.qself else {
            return;
        };
        let method = function.path.segments.last().unwrap();
        let Some(receiver) = self.canonical_type(&qself.ty) else {
            return;
        };
        if dynamic_type(&receiver)
            && runtime_method(
                &method.ident.to_string(),
                !matches!(method.arguments, syn::PathArguments::None),
            )
        {
            self.push("runtime_type", method.ident.to_string());
        }
    }
}
impl<'ast> Visit<'ast> for Scanner<'_> {
    fn visit_item_mod(&mut self, node: &'ast ItemMod) {
        if skip(&node.attrs) {
            return;
        }
        let Some((_, items)) = &node.content else {
            return;
        };
        self.scope.push(node.ident.to_string());
        let old = self.set_owner(module_owner(self.local_scope()));
        self.emit_uses();
        for item in items {
            self.visit_item(item);
        }
        self.owner = old;
        self.scope.pop();
    }
    fn visit_item_use(&mut self, _: &'ast syn::ItemUse) {}
    fn visit_item_fn(&mut self, node: &'ast ItemFn) {
        if skip(&node.attrs) {
            return;
        }
        let old = self.set_owner(qualified(self.local_scope(), &node.sig.ident.to_string()));
        visit::visit_signature(self, &node.sig);
        self.visit_block(&node.block);
        self.owner = old;
    }
    fn visit_item_type(&mut self, node: &'ast ItemType) {
        if skip(&node.attrs) {
            return;
        }
        let old = self.set_owner(qualified(self.local_scope(), &node.ident.to_string()));
        if let Some(detail) = self.canonical_type(&node.ty)
            && dynamic_type(&detail)
        {
            self.push("alias", detail);
        }
        visit::visit_generics(self, &node.generics);
        self.owner = old;
    }
    fn visit_item_struct(&mut self, node: &'ast ItemStruct) {
        if skip(&node.attrs) {
            return;
        }
        let name = qualified(self.local_scope(), &node.ident.to_string());
        let old = self.set_owner(name.clone());
        if canonical_dynamic_path(&name).is_some() {
            self.push("definition", name.clone());
        }
        visit::visit_generics(self, &node.generics);
        for (index, field_node) in node.fields.iter().enumerate() {
            let field = field_node
                .ident
                .as_ref()
                .map(ToString::to_string)
                .unwrap_or_else(|| index.to_string());
            self.visit_field(field_node, format!("{name}.{field}"));
        }
        self.owner = old;
    }
    fn visit_item_enum(&mut self, node: &'ast ItemEnum) {
        if skip(&node.attrs) {
            return;
        }
        let name = qualified(self.local_scope(), &node.ident.to_string());
        let old = self.set_owner(name.clone());
        if canonical_dynamic_path(&name).is_some() {
            self.push("definition", name.clone());
        }
        visit::visit_generics(self, &node.generics);
        for variant in &node.variants {
            if skip(&variant.attrs) {
                continue;
            }
            let owner = format!("{name}.{}", variant.ident);
            self.visit_variant_fields(&variant.fields, &owner);
            if let Some((_, discriminant)) = &variant.discriminant {
                let prior = self.set_owner(owner);
                self.visit_expr(discriminant);
                self.owner = prior;
            }
        }
        self.owner = old;
    }
    fn visit_item_const(&mut self, node: &'ast ItemConst) {
        if skip(&node.attrs) {
            return;
        }
        let old = self.set_owner(qualified(self.local_scope(), &node.ident.to_string()));
        visit::visit_item_const(self, node);
        self.owner = old;
    }
    fn visit_item_static(&mut self, node: &'ast ItemStatic) {
        if skip(&node.attrs) {
            return;
        }
        let old = self.set_owner(qualified(self.local_scope(), &node.ident.to_string()));
        visit::visit_item_static(self, node);
        self.owner = old;
    }
    fn visit_item_impl(&mut self, node: &'ast ItemImpl) {
        if skip(&node.attrs) {
            return;
        }
        let previous = std::mem::take(&mut self.impl_name);
        self.impl_name = self.canonical_type(&node.self_ty);
        visit::visit_item_impl(self, node);
        self.impl_name = previous;
    }
    fn visit_impl_item_fn(&mut self, node: &'ast ImplItemFn) {
        if skip(&node.attrs) {
            return;
        }
        let owner = associated_owner(self.impl_name.as_deref(), "impl", &node.sig.ident);
        let old = self.set_owner(owner);
        visit::visit_impl_item_fn(self, node);
        self.owner = old;
    }
    fn visit_impl_item_type(&mut self, node: &'ast ImplItemType) {
        if skip(&node.attrs) {
            return;
        }
        let owner = associated_owner(self.impl_name.as_deref(), "impl", &node.ident);
        let old = self.set_owner(owner);
        visit::visit_impl_item_type(self, node);
        self.owner = old;
    }
    fn visit_impl_item_const(&mut self, node: &'ast ImplItemConst) {
        if skip(&node.attrs) {
            return;
        }
        let owner = associated_owner(self.impl_name.as_deref(), "impl", &node.ident);
        let old = self.set_owner(owner);
        visit::visit_impl_item_const(self, node);
        self.owner = old;
    }
    fn visit_item_trait(&mut self, node: &'ast ItemTrait) {
        if skip(&node.attrs) {
            return;
        }
        let previous = self.trait_name.replace(node.ident.to_string());
        visit::visit_item_trait(self, node);
        self.trait_name = previous;
    }
    fn visit_trait_item_fn(&mut self, node: &'ast TraitItemFn) {
        if skip(&node.attrs) {
            return;
        }
        let owner = associated_owner(self.trait_name.as_deref(), "trait", &node.sig.ident);
        let old = self.set_owner(owner);
        visit::visit_trait_item_fn(self, node);
        self.owner = old;
    }
    fn visit_trait_item_type(&mut self, node: &'ast TraitItemType) {
        if skip(&node.attrs) {
            return;
        }
        let owner = associated_owner(self.trait_name.as_deref(), "trait", &node.ident);
        let old = self.set_owner(owner);
        visit::visit_trait_item_type(self, node);
        self.owner = old;
    }
    fn visit_trait_item_const(&mut self, node: &'ast TraitItemConst) {
        if skip(&node.attrs) {
            return;
        }
        let owner = associated_owner(self.trait_name.as_deref(), "trait", &node.ident);
        let old = self.set_owner(owner);
        visit::visit_trait_item_const(self, node);
        self.owner = old;
    }
    fn visit_type(&mut self, node: &'ast Type) {
        if let Some(rendered) = self.canonical_type(node)
            && dynamic_type(&rendered)
        {
            self.push("type", rendered);
        }
    }
    fn visit_expr_path(&mut self, node: &'ast ExprPath) {
        let resolved = self.resolver.resolve_path(&self.scope, &node.path);
        if let Some(path) = canonical_dynamic_path(&resolved) {
            self.push("path", path);
        }
        visit::visit_expr_path(self, node);
    }
    fn visit_expr_call(&mut self, node: &'ast ExprCall) {
        if let syn::Expr::Path(function) = node.func.as_ref() {
            self.emit_call_path(function);
        }
        visit::visit_expr_call(self, node);
    }
    fn visit_expr_method_call(&mut self, node: &'ast ExprMethodCall) {
        if runtime_method(&node.method.to_string(), node.turbofish.is_some()) {
            self.push("runtime_type", node.method.to_string());
        }
        visit::visit_expr_method_call(self, node);
    }
    fn visit_macro(&mut self, node: &'ast Macro) {
        let path = self.resolver.resolve_path(&self.scope, &node.path);
        if let Some(mac) = canonical_macro(&path) {
            self.push("macro", mac);
        }
    }
}
