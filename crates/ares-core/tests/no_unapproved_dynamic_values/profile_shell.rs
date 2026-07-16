use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};

use syn::{
    Expr, ExprCall, ExprClosure, ExprMethodCall, FnArg, ImplItem, ImplItemFn, Item, ItemFn,
    ItemImpl, ItemMod, Local, Pat, PatIdent, TraitItem, TraitItemFn, Type,
    visit::{self, Visit},
};

use crate::{
    classification::{render_type, skip},
    finding::{normalize_path, render_findings},
    imports::{ImportResolver, source_scope},
};

#[path = "profile_shell/identity.rs"]
mod identity;
use identity::ProfileIdentity;

const PROFILE_ROOT: &str = "crates/ares-core/src/profiles/";

pub(super) fn assert_profile_modules_use_only_typed_shells(repo: &Path) {
    let (findings, sources) =
        super::scan_repository(repo).unwrap_or_else(|error| panic!("{error}"));
    let mut violations = render_findings(
        &findings
            .into_iter()
            .filter(|finding| finding.path.starts_with(PROFILE_ROOT))
            .collect::<Vec<_>>(),
    );
    violations.extend(scan_profile_shells(&sources).unwrap_or_else(|error| panic!("{error}")));
    assert!(
        violations.is_empty(),
        "profile typed-shell audit failed:\n{}",
        violations.into_iter().collect::<Vec<_>>().join("\n")
    );
}

fn scan_profile_shells(sources: &BTreeMap<String, String>) -> Result<BTreeSet<String>, String> {
    let files = sources
        .iter()
        .map(|(path, source)| {
            syn::parse_file(source)
                .map(|file| (path.as_str(), file))
                .map_err(|error| format!("could not parse {path}: {error}"))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let resolver = ImportResolver::project(files.iter().map(|(path, file)| (*path, file)));
    let identity = ProfileIdentity::new(
        &files,
        &resolver,
        &source_scope("crates/ares-core/src/lib.rs"),
    );
    let mut violations = BTreeSet::new();
    for (path, file) in files
        .iter()
        .filter(|(path, _)| normalize_path(path).starts_with(PROFILE_ROOT))
    {
        let mut scanner = ShellScanner {
            path,
            resolver: &resolver,
            identity: &identity,
            scope: source_scope(path),
            option_variables: BTreeSet::new(),
            violations: BTreeSet::new(),
        };
        scanner.emit_shell_uses();
        scanner.visit_file(file);
        violations.extend(scanner.violations);
    }
    Ok(violations)
}

struct ShellScanner<'a> {
    path: &'a str,
    resolver: &'a ImportResolver,
    identity: &'a ProfileIdentity,
    scope: Vec<String>,
    option_variables: BTreeSet<String>,
    violations: BTreeSet<String>,
}

impl<'a> ShellScanner<'a> {
    fn reject(&mut self, category: &str, detail: impl AsRef<str>) {
        self.violations
            .insert(format!("{}|{category}|{}", self.path, detail.as_ref()));
    }

    fn emit_shell_uses(&mut self) {
        for name in self.identity.slice_imports(&self.scope) {
            self.reject("slice-options-use", name);
        }
    }

    fn rendered_type(&mut self, ty: &Type) -> Option<String> {
        render_type(ty, self.resolver, &self.scope).ok()
    }

    fn forbidden_type(&self, ty: &Type) -> bool {
        self.identity.is_slice_type(&self.scope, ty) || self.identity.is_owner(&self.scope, ty)
    }

    fn bind(&mut self, pat: &Pat, forbidden: bool) {
        let mut bindings = Bindings::default();
        bindings.visit_pat(pat);
        self.option_variables
            .retain(|name| !bindings.names.contains(name));
        if forbidden {
            self.option_variables.extend(bindings.names);
        }
    }

    fn enter_function(&mut self, inputs: &syn::punctuated::Punctuated<FnArg, syn::Token![,]>) {
        self.option_variables.clear();
        for input in inputs {
            let FnArg::Typed(argument) = input else {
                continue;
            };
            self.bind(&argument.pat, self.forbidden_type(&argument.ty));
        }
    }
}

impl<'ast> Visit<'ast> for ShellScanner<'_> {
    fn visit_block(&mut self, block: &'ast syn::Block) {
        let saved = self.option_variables.clone();
        visit::visit_block(self, block);
        self.option_variables = saved;
    }

    fn visit_local(&mut self, local: &'ast Local) {
        let alias = local
            .init
            .as_ref()
            .and_then(|init| receiver_name(&init.expr))
            .is_some_and(|name| self.option_variables.contains(&name));
        let typed = match &local.pat {
            Pat::Type(pat) => self.forbidden_type(&pat.ty),
            _ => false,
        };
        visit::visit_local(self, local);
        self.bind(&local.pat, alias || typed);
    }

    fn visit_expr_closure(&mut self, closure: &'ast ExprClosure) {
        let saved = self.option_variables.clone();
        for pat in &closure.inputs {
            let typed = match pat {
                Pat::Type(pat) => self.forbidden_type(&pat.ty),
                _ => false,
            };
            self.bind(pat, typed);
        }
        visit::visit_expr_closure(self, closure);
        self.option_variables = saved;
    }

    fn visit_item(&mut self, item: &'ast Item) {
        if !skip(item_attrs(item)) {
            visit::visit_item(self, item);
        }
    }

    fn visit_impl_item(&mut self, item: &'ast ImplItem) {
        if !skip(impl_item_attrs(item)) {
            visit::visit_impl_item(self, item);
        }
    }

    fn visit_trait_item(&mut self, item: &'ast TraitItem) {
        if !skip(trait_item_attrs(item)) {
            visit::visit_trait_item(self, item);
        }
    }

    fn visit_field(&mut self, field: &'ast syn::Field) {
        if !skip(&field.attrs) {
            visit::visit_field(self, field);
        }
    }

    fn visit_variant(&mut self, variant: &'ast syn::Variant) {
        if !skip(&variant.attrs) {
            visit::visit_variant(self, variant);
        }
    }

    fn visit_item_mod(&mut self, item: &'ast ItemMod) {
        let Some((_, _)) = &item.content else {
            return;
        };
        self.scope.push(item.ident.to_string());
        self.emit_shell_uses();
        visit::visit_item_mod(self, item);
        self.scope.pop();
    }

    fn visit_item_fn(&mut self, item: &'ast ItemFn) {
        let saved = std::mem::take(&mut self.option_variables);
        self.enter_function(&item.sig.inputs);
        visit::visit_item_fn(self, item);
        self.option_variables = saved;
    }

    fn visit_item_impl(&mut self, item: &'ast ItemImpl) {
        if self.identity.is_owner(&self.scope, &item.self_ty)
            && item.items.iter().any(|item| {
                matches!(item, syn::ImplItem::Fn(item) if item.sig.ident == "values" && !skip(&item.attrs))
            })
        {
            self.reject("option-map-values-method", "values");
        }
        visit::visit_item_impl(self, item);
    }

    fn visit_impl_item_fn(&mut self, item: &'ast ImplItemFn) {
        let saved = std::mem::take(&mut self.option_variables);
        self.enter_function(&item.sig.inputs);
        visit::visit_impl_item_fn(self, item);
        self.option_variables = saved;
    }

    fn visit_trait_item_fn(&mut self, item: &'ast TraitItemFn) {
        let saved = std::mem::take(&mut self.option_variables);
        self.enter_function(&item.sig.inputs);
        visit::visit_trait_item_fn(self, item);
        self.option_variables = saved;
    }

    fn visit_type(&mut self, ty: &'ast Type) {
        if self.identity.is_slice_type(&self.scope, ty)
            && let Some(rendered) = self.rendered_type(ty)
        {
            self.reject("slice-options-type", rendered);
        }
        visit::visit_type(self, ty);
    }

    fn visit_expr_path(&mut self, expression: &'ast syn::ExprPath) {
        let path = self.resolver.resolve_path(&self.scope, &expression.path);
        if self.identity.is_slice_path(&self.scope, &expression.path) {
            self.reject("slice-options-path", path);
        }
        visit::visit_expr_path(self, expression);
    }

    fn visit_expr_call(&mut self, call: &'ast ExprCall) {
        if let Expr::Path(function) = call.func.as_ref() {
            let path = self.resolver.resolve_path(&self.scope, &function.path);
            if is_json_output(&path) {
                self.reject("json-round-trip", path);
            }
        }
        visit::visit_expr_call(self, call);
    }

    fn visit_expr_method_call(&mut self, call: &'ast ExprMethodCall) {
        if call.method == "values"
            && let Some(receiver) = receiver_name(&call.receiver)
            && self.option_variables.contains(&receiver)
        {
            self.reject("option-map-values", receiver);
        }
        visit::visit_expr_method_call(self, call);
    }
}

fn is_json_output(path: &str) -> bool {
    path.strip_prefix("serde_json::").is_some_and(|name| {
        matches!(
            name,
            "to_value"
                | "to_string"
                | "to_string_pretty"
                | "to_vec"
                | "to_vec_pretty"
                | "to_writer"
                | "to_writer_pretty"
        )
    })
}

fn receiver_name(expression: &Expr) -> Option<String> {
    match expression {
        Expr::Path(path) => path.path.get_ident().map(ToString::to_string),
        Expr::Reference(reference) => receiver_name(&reference.expr),
        Expr::Paren(paren) => receiver_name(&paren.expr),
        Expr::Group(group) => receiver_name(&group.expr),
        _ => None,
    }
}

#[derive(Default)]
struct Bindings {
    names: BTreeSet<String>,
}

impl<'ast> Visit<'ast> for Bindings {
    fn visit_pat_ident(&mut self, pat: &'ast PatIdent) {
        self.names.insert(pat.ident.to_string());
        visit::visit_pat_ident(self, pat);
    }
}

macro_rules! attrs {
    ($item:expr, $kind:ident; $($variant:ident),+ $(,)?) => {
        match $item {
            $(syn::$kind::$variant(item) => &item.attrs,)+
            _ => &[],
        }
    };
}

fn item_attrs(item: &Item) -> &[syn::Attribute] {
    attrs!(item, Item; Const, Enum, ExternCrate, Fn, ForeignMod, Impl, Macro, Mod, Static, Struct, Trait, TraitAlias, Type, Union, Use)
}

fn impl_item_attrs(item: &ImplItem) -> &[syn::Attribute] {
    attrs!(item, ImplItem; Const, Fn, Type, Macro)
}

fn trait_item_attrs(item: &TraitItem) -> &[syn::Attribute] {
    attrs!(item, TraitItem; Const, Fn, Type, Macro)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scan_sources(sources: &[(&str, &str)]) -> BTreeSet<String> {
        let sources = sources
            .iter()
            .map(|(path, source)| ((*path).to_owned(), (*source).to_owned()))
            .collect();
        scan_profile_shells(&sources).unwrap()
    }

    fn scan(file: &str, source: &str) -> BTreeSet<String> {
        scan_sources(&[
            ("crates/ares-core/src/lib.rs", "pub struct SliceOptions;"),
            (&format!("{PROFILE_ROOT}{file}"), source),
        ])
    }

    #[test]
    fn unrelated_and_private_nested_values_are_allowed() {
        let violations = scan(
            "composition.rs",
            r#"use serde::de::IgnoredAny; #[cfg(test)] type TestOnly = crate::SliceOptions; #[cfg(test)] struct TestStruct(crate::SliceOptions); #[cfg(test)] enum TestEnum { Only(crate::SliceOptions) } #[cfg(test)] const TEST_ONLY: Option<crate::SliceOptions> = None; #[cfg(test)] trait TestOnlyTrait { fn leak(x: &crate::SliceOptions) { x.values(); } } #[test] fn test_only_function(x: &crate::SliceOptions) { x.values(); } struct Typed { #[cfg(test)] test_only: crate::SliceOptions } enum TypedVariant { #[cfg(test)] TestOnly(crate::SliceOptions), Runtime } impl Typed { fn values(&self) {} #[cfg(test)] type TestOnly = crate::SliceOptions; #[cfg(test)] const TEST_ONLY: Option<crate::SliceOptions> = None; #[cfg(test)] fn leak(x: &crate::SliceOptions) { x.values(); } } trait TypedTrait { #[cfg(test)] type TestOnly = crate::SliceOptions; #[cfg(test)] const TEST_ONLY: Option<crate::SliceOptions> = None; #[cfg(test)] fn leak(x: &crate::SliceOptions) { x.values(); } } pub struct ComposedProfile; mod private { struct ComposedProfile; impl ComposedProfile { fn values(&self) {} } } fn f(options: &Typed, _: IgnoredAny) { options.values(); }"#,
        );
        assert!(violations.is_empty(), "{violations:#?}");
    }

    #[test]
    fn named_and_globbed_slice_options_aliases_are_rejected() {
        let violations = scan(
            "composition.rs",
            "mod aliases { pub type GlobConfig = crate::SliceOptions; } use crate::SliceOptions as NamedConfig; use aliases::*; fn f(named: &NamedConfig, globbed: &GlobConfig) { named.values(); globbed.values(); }",
        );
        let report = violations.into_iter().collect::<Vec<_>>().join("\n");
        assert!(
            ["named", "globbed"]
                .iter()
                .all(|receiver| report.contains(&format!("|option-map-values|{receiver}")))
                && report.contains("|slice-options-use|NamedConfig"),
            "{report}"
        );
    }

    #[test]
    fn root_fixed_owner_values_impl_in_child_is_rejected() {
        let violations = scan_sources(&[
            (
                "crates/ares-core/src/lib.rs",
                "pub struct ProfileFragment; pub struct ComposedProfile; impl ComposedProfile { pub fn values(&self) {} } mod profiles;",
            ),
            (
                "crates/ares-core/src/profiles/child.rs",
                "impl crate::ProfileFragment { pub fn values(&self) {} } fn leak(x: &crate::ComposedProfile) { x.values(); } trait Leak { fn leak(owner: &crate::ComposedProfile) { owner.values(); } } fn before_then_shadow(before: &crate::ComposedProfile) { before.values(); let before = std::collections::BTreeMap::<u8, u8>::new(); before.values(); } fn local_shadow(shadowed: &crate::ComposedProfile) { let shadowed = std::collections::BTreeMap::<u8, u8>::new(); shadowed.values(); } fn alias_source(source: &crate::ComposedProfile) { let alias = source; alias.values(); } fn closure_shadow(closure_arg: &crate::ComposedProfile) { let _ = |closure_arg| closure_arg.values(); }",
            ),
        ]);
        let report = violations.into_iter().collect::<Vec<_>>().join("\n");
        for receiver in ["x", "owner", "before", "alias"] {
            assert!(report.contains(&format!("|option-map-values|{receiver}")));
        }
        for receiver in ["shadowed", "closure_arg"] {
            assert!(!report.contains(&format!("|option-map-values|{receiver}")));
        }
        assert!(report.contains("|option-map-values-method|values"));
    }
}
