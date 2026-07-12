use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

use serde::Deserialize;
use syn::{
    Expr, ExprField, FnArg, ImplItemFn, ItemFn, ItemImpl, ItemMod, Local, Member, Pat, PatIdent,
    PatStruct, Type,
    visit::{self, Visit},
};

use crate::{
    classification::{dynamic_type, skip},
    finding::{Finding, normalize_path, qualified},
};

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Allowlist {
    pub open_field: Vec<OpenField>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct OpenField {
    pub path: String,
    pub containing_struct: String,
    pub field: String,
    pub upstream_source: String,
    pub rationale: String,
}

pub(super) fn parse(text: &str) -> Result<Allowlist, String> {
    let allowlist: Allowlist = toml::from_str(text).map_err(|error| error.to_string())?;
    for entry in &allowlist.open_field {
        for (name, value) in [
            ("path", &entry.path),
            ("containing_struct", &entry.containing_struct),
            ("field", &entry.field),
            ("upstream_source", &entry.upstream_source),
            ("rationale", &entry.rationale),
        ] {
            if value.trim().is_empty() {
                return Err(format!("allowlist field `{name}` must not be empty"));
            }
        }
    }
    Ok(allowlist)
}

pub(super) fn apply(
    mut findings: Vec<Finding>,
    allowlist: &Allowlist,
    sources: &BTreeMap<String, String>,
) -> Result<Vec<Finding>, String> {
    let mut remove = BTreeSet::new();
    let helpers = collect_helpers(sources.values().map(String::as_str))?;
    for entry in &allowlist.open_field {
        let path = normalize_path(&entry.path);
        let source = sources
            .get(&path)
            .ok_or_else(|| format!("allowlist entry is not a reachable production file: {path}"))?;
        let matches = findings
            .iter()
            .enumerate()
            .filter(|(_, finding)| {
                finding.path == path
                    && finding.field_owner(&entry.containing_struct, &entry.field)
                    && dynamic_type(&finding.detail)
            })
            .map(|(index, _)| index)
            .collect::<Vec<_>>();
        if matches.len() != 1 {
            return Err(format!(
                "allowlist entry {}.{} matched {} dynamic struct fields",
                entry.containing_struct,
                entry.field,
                matches.len()
            ));
        }
        let dispatch = dispatch_violations_with_helpers(source, entry, &helpers)?;
        if !dispatch.is_empty() {
            return Err(format!(
                "allowlisted field {}.{} controls slicing dispatch: {}",
                entry.containing_struct,
                entry.field,
                dispatch.join(", ")
            ));
        }
        remove.insert(matches[0]);
    }
    let mut index = 0;
    findings.retain(|_| {
        let keep = !remove.contains(&index);
        index += 1;
        keep
    });
    Ok(findings)
}

pub(super) fn dispatch_violations(source: &str, entry: &OpenField) -> Result<Vec<String>, String> {
    let helpers = collect_helpers([source])?;
    dispatch_violations_with_helpers(source, entry, &helpers)
}

fn collect_helpers<'a>(
    sources: impl IntoIterator<Item = &'a str>,
) -> Result<HashSet<String>, String> {
    let mut names = HashSet::new();
    for source in sources {
        let file = syn::parse_file(source).map_err(|error| error.to_string())?;
        let mut helpers = HelperCollector::default();
        helpers.visit_file(&file);
        names.extend(helpers.names);
    }
    Ok(names)
}

fn dispatch_violations_with_helpers(
    source: &str,
    entry: &OpenField,
    helpers: &HashSet<String>,
) -> Result<Vec<String>, String> {
    let file = syn::parse_file(source).map_err(|error| error.to_string())?;
    let mut scanner = DispatchScanner {
        entry,
        helpers,
        scope: Vec::new(),
        open_vars: HashSet::new(),
        aliases: HashMap::new(),
        impl_open: false,
        violations: BTreeSet::new(),
        error: None,
    };
    scanner.visit_file(&file);
    match scanner.error {
        Some(error) => Err(error),
        None => Ok(scanner.violations.into_iter().collect()),
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Hop {
    Direct,
    Alias,
    Helper,
}

#[derive(Default)]
struct HelperCollector {
    names: HashSet<String>,
}
impl<'ast> Visit<'ast> for HelperCollector {
    fn visit_item_fn(&mut self, node: &'ast ItemFn) {
        if !skip(&node.attrs) {
            self.names.insert(node.sig.ident.to_string());
        }
    }
    fn visit_impl_item_fn(&mut self, node: &'ast ImplItemFn) {
        if !skip(&node.attrs) {
            self.names.insert(node.sig.ident.to_string());
        }
    }
}

struct DispatchScanner<'a> {
    entry: &'a OpenField,
    helpers: &'a HashSet<String>,
    scope: Vec<String>,
    open_vars: HashSet<String>,
    aliases: HashMap<String, Hop>,
    impl_open: bool,
    violations: BTreeSet<String>,
    error: Option<String>,
}

impl DispatchScanner<'_> {
    fn owner_name(&self, name: &str) -> bool {
        let name = name.strip_prefix("crate::").unwrap_or(name);
        if let Some(name) = name.strip_prefix("self::") {
            return self.entry.containing_struct == qualified(&self.scope, name);
        }
        self.entry.containing_struct == name
            || (!name.contains("::")
                && self.entry.containing_struct == qualified(&self.scope, name))
    }
    fn type_is_open(&mut self, ty: &Type) -> bool {
        self.type_name(ty)
            .is_some_and(|name| self.owner_name(&name))
    }
    fn signature_open_var(&mut self, input: &FnArg) -> Option<String> {
        match input {
            FnArg::Receiver(_) if self.impl_open => Some("self".to_owned()),
            FnArg::Typed(argument) if self.type_is_open(&argument.ty) => {
                let Pat::Ident(ident) = argument.pat.as_ref() else {
                    return None;
                };
                Some(ident.ident.to_string())
            }
            _ => None,
        }
    }
    fn type_name(&mut self, ty: &Type) -> Option<String> {
        match type_name(ty) {
            Ok(name) => name,
            Err(error) => {
                self.error.get_or_insert(error);
                None
            }
        }
    }
    fn enter_signature(&mut self, inputs: &syn::punctuated::Punctuated<FnArg, syn::Token![,]>) {
        self.open_vars = inputs
            .iter()
            .filter_map(|input| self.signature_open_var(input))
            .collect();
        self.aliases.clear();
    }
    fn hop(&self, expression: &Expr) -> Option<Hop> {
        match expression {
            Expr::Field(field) if self.target_field(field) => Some(Hop::Direct),
            Expr::Path(path) if path.path.segments.len() == 1 => self
                .aliases
                .get(&path.path.segments[0].ident.to_string())
                .copied(),
            Expr::Reference(reference) => self.hop(&reference.expr),
            Expr::Paren(paren) => self.hop(&paren.expr),
            Expr::Group(group) => self.hop(&group.expr),
            Expr::Unary(unary) => self.hop(&unary.expr),
            Expr::MethodCall(call) => self.hop(&call.receiver),
            Expr::Binary(binary) => self.hop(&binary.left).or_else(|| self.hop(&binary.right)),
            Expr::Let(let_expression) => self.hop(&let_expression.expr),
            Expr::Call(call) => {
                let Expr::Path(function) = call.func.as_ref() else {
                    return None;
                };
                let name = function.path.segments.last()?.ident.to_string();
                if !self.helpers.contains(&name) {
                    return None;
                }
                call.args
                    .iter()
                    .filter_map(|arg| self.hop(arg))
                    .min()
                    .filter(|hop| *hop <= Hop::Alias)
                    .map(|_| Hop::Helper)
            }
            _ => None,
        }
    }
    fn target_field(&self, field: &ExprField) -> bool {
        let Member::Named(member) = &field.member else {
            return false;
        };
        if member != &self.entry.field {
            return false;
        }
        base_ident(&field.base).is_some_and(|ident| self.open_vars.contains(&ident))
    }
    fn bind_destructured_fields(&mut self, pattern: &Pat) {
        let Pat::Struct(PatStruct { path, fields, .. }) = pattern else {
            return;
        };
        let owner = path
            .segments
            .iter()
            .map(|part| part.ident.to_string())
            .collect::<Vec<_>>()
            .join("::");
        if !self.owner_name(&owner) {
            return;
        }
        for field in fields {
            if !matches!(&field.member, Member::Named(name) if name == self.entry.field.as_str()) {
                continue;
            }
            let Pat::Ident(ident) = field.pat.as_ref() else {
                continue;
            };
            self.aliases.insert(ident.ident.to_string(), Hop::Direct);
        }
    }
    fn bind_local(&mut self, local: &Local) {
        self.bind_destructured_fields(&local.pat);
        let Some(init) = &local.init else {
            return;
        };
        let Pat::Ident(PatIdent { ident, .. }) = &local.pat else {
            return;
        };
        if let Expr::Path(path) = init.expr.as_ref()
            && path
                .path
                .get_ident()
                .is_some_and(|name| self.open_vars.contains(&name.to_string()))
        {
            self.open_vars.insert(ident.to_string());
        }
        if let Some(hop) = self.hop(&init.expr) {
            self.aliases.insert(ident.to_string(), hop.max(Hop::Alias));
        }
    }
}
impl<'ast> Visit<'ast> for DispatchScanner<'_> {
    fn visit_item_mod(&mut self, node: &'ast ItemMod) {
        if skip(&node.attrs) {
            return;
        }
        self.scope.push(node.ident.to_string());
        visit::visit_item_mod(self, node);
        self.scope.pop();
    }
    fn visit_item_impl(&mut self, node: &'ast ItemImpl) {
        if skip(&node.attrs) {
            return;
        }
        let old = self.impl_open;
        self.impl_open = self.type_is_open(&node.self_ty);
        visit::visit_item_impl(self, node);
        self.impl_open = old;
    }
    fn visit_item_fn(&mut self, node: &'ast ItemFn) {
        if skip(&node.attrs) {
            return;
        }
        let saved_vars = std::mem::take(&mut self.open_vars);
        let saved_aliases = std::mem::take(&mut self.aliases);
        self.enter_signature(&node.sig.inputs);
        self.visit_block(&node.block);
        self.open_vars = saved_vars;
        self.aliases = saved_aliases;
    }
    fn visit_impl_item_fn(&mut self, node: &'ast ImplItemFn) {
        if skip(&node.attrs) {
            return;
        }
        let saved_vars = std::mem::take(&mut self.open_vars);
        let saved_aliases = std::mem::take(&mut self.aliases);
        self.enter_signature(&node.sig.inputs);
        self.visit_block(&node.block);
        self.open_vars = saved_vars;
        self.aliases = saved_aliases;
    }
    fn visit_local(&mut self, node: &'ast Local) {
        if !skip(&node.attrs) {
            self.bind_local(node);
            visit::visit_local(self, node);
        }
    }
    fn visit_expr_if(&mut self, node: &'ast syn::ExprIf) {
        if self.hop(&node.cond).is_some() {
            self.violations.insert(
                if matches!(node.cond.as_ref(), Expr::Let(_)) {
                    "if-let"
                } else {
                    "if"
                }
                .to_owned(),
            );
        }
        visit::visit_expr_if(self, node);
    }
    fn visit_expr_match(&mut self, node: &'ast syn::ExprMatch) {
        if self.hop(&node.expr).is_some() {
            self.violations.insert("match".to_owned());
        }
        visit::visit_expr_match(self, node);
    }
}
fn type_name(ty: &Type) -> Result<Option<String>, String> {
    match ty {
        Type::Path(path) => Ok(Some(
            path.path
                .segments
                .iter()
                .map(|part| part.ident.to_string())
                .collect::<Vec<_>>()
                .join("::"),
        )),
        Type::Reference(reference) => type_name(&reference.elem),
        Type::Paren(paren) => type_name(&paren.elem),
        Type::Group(group) => type_name(&group.elem),
        Type::Macro(_) => Err("unsupported allowlist receiver type syntax `macro`".to_owned()),
        Type::Verbatim(_) => {
            Err("unsupported allowlist receiver type syntax `verbatim`".to_owned())
        }
        _ => Ok(None),
    }
}
fn base_ident(expression: &Expr) -> Option<String> {
    match expression {
        Expr::Path(path) => path.path.get_ident().map(ToString::to_string),
        Expr::Reference(reference) => base_ident(&reference.expr),
        Expr::Paren(paren) => base_ident(&paren.expr),
        Expr::Group(group) => base_ident(&group.expr),
        _ => None,
    }
}
