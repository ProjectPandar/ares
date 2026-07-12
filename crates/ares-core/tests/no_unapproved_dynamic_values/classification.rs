use syn::{
    AngleBracketedGenericArguments, Attribute, Expr, GenericArgument, Lit, Path, PathArguments,
    ReturnType, Type, TypeParamBound,
};

use crate::imports::ImportResolver;

const CUSTOM_TYPES: [&str; 4] = [
    "ConfigValue",
    "DynamicValue",
    "ErasedValue",
    "ErasedPayload",
];
const DOM_ROOTS: [&str; 8] = [
    "roxmltree::Document",
    "roxmltree::Node",
    "xmltree::Element",
    "xmltree::XMLNode",
    "simd_json::OwnedValue",
    "simd_json::BorrowedValue",
    "json::JsonValue",
    "sonic_rs::Value",
];

pub(super) fn skip(attrs: &[Attribute]) -> bool {
    attrs.iter().any(|attr| {
        let path = attr.path();
        (path.is_ident("test") || path.segments.iter().map(|s| s.ident.to_string()).eq(["tokio", "test"]))
            || (path.is_ident("cfg")
                && matches!(&attr.meta, syn::Meta::List(list) if compact(&list.tokens.to_string()) == "test"))
    })
}

pub(super) fn canonical_dynamic_path(path: &str) -> Option<String> {
    let path = path.trim_start_matches("::");
    let canonical = match path {
        "serde_json::value::Value" => "serde_json::Value",
        "serde_json::map::Map" => "serde_json::Map",
        "serde_json::RawValue" => "serde_json::value::RawValue",
        other => other,
    };
    let serde = [
        "serde_json::Value",
        "serde_json::Map",
        "serde_json::value::RawValue",
    ];
    if serde.iter().any(|root| path_boundary(canonical, root))
        || DOM_ROOTS.iter().any(|root| path_boundary(canonical, root))
        || matches!(canonical, "std::any::TypeId" | "core::any::TypeId")
        || canonical
            .split("::")
            .any(|segment| CUSTOM_TYPES.contains(&segment))
    {
        Some(canonical.to_owned())
    } else {
        None
    }
}

pub(super) fn canonical_call(path: &str) -> Option<&'static str> {
    matches!(path, "serde_json::from_value").then_some("serde_json::from_value")
}

pub(super) fn canonical_macro(path: &str) -> Option<&'static str> {
    matches!(path, "serde_json::json" | "json").then_some("serde_json::json!")
}

pub(super) fn runtime_method(method: &str, typed: bool) -> bool {
    matches!(
        method,
        "downcast" | "downcast_ref" | "downcast_mut" | "type_id"
    ) || (method == "is" && typed)
}

pub(super) fn dynamic_type(rendered: &str) -> bool {
    canonical_dynamic_path(rendered).is_some()
        || rendered
            .split(|character: char| !character.is_alphanumeric() && character != '_')
            .any(|part| CUSTOM_TYPES.contains(&part))
        || rendered.contains("dynstd::any::Any")
        || rendered.contains("dyncore::any::Any")
        || rendered.contains("dynAny")
        || rendered.contains("dynerased_serde::Serialize")
        || [
            "serde_json::Value",
            "serde_json::Map",
            "serde_json::value::RawValue",
        ]
        .iter()
        .chain(DOM_ROOTS.iter())
        .any(|root| rendered.contains(root))
        || rendered.contains("std::any::TypeId")
        || rendered.contains("core::any::TypeId")
}

pub(super) fn render_type(
    ty: &Type,
    resolver: &ImportResolver,
    scope: &[String],
) -> Result<String, String> {
    match ty {
        Type::Path(path) if path.qself.is_none() => render_path(&path.path, resolver, scope),
        Type::Path(_) => unsupported_type("qualified path"),
        Type::Reference(reference) => Ok(format!(
            "&{}{}{}",
            reference
                .lifetime
                .as_ref()
                .map(ToString::to_string)
                .unwrap_or_default(),
            if reference.mutability.is_some() {
                "mut"
            } else {
                ""
            },
            render_type(&reference.elem, resolver, scope)?
        )),
        Type::TraitObject(object) => Ok(format!(
            "dyn{}",
            render_bounds(&object.bounds, resolver, scope)?
        )),
        Type::ImplTrait(object) => Ok(format!(
            "impl{}",
            render_bounds(&object.bounds, resolver, scope)?
        )),
        Type::Slice(slice) => Ok(format!("[{}]", render_type(&slice.elem, resolver, scope)?)),
        Type::Array(array) => Ok(format!(
            "[{};{}]",
            render_type(&array.elem, resolver, scope)?,
            render_const_expr(&array.len, resolver, scope)?
        )),
        Type::Tuple(tuple) => Ok(format!(
            "({})",
            tuple
                .elems
                .iter()
                .map(|ty| render_type(ty, resolver, scope))
                .collect::<Result<Vec<_>, _>>()?
                .join(",")
        )),
        Type::Ptr(pointer) => Ok(format!(
            "*{}{}",
            if pointer.mutability.is_some() {
                "mut"
            } else {
                "const"
            },
            render_type(&pointer.elem, resolver, scope)?
        )),
        Type::Paren(paren) => Ok(format!("({})", render_type(&paren.elem, resolver, scope)?)),
        Type::Group(group) => render_type(&group.elem, resolver, scope),
        Type::BareFn(function)
            if function.lifetimes.is_none()
                && function.unsafety.is_none()
                && function.abi.is_none()
                && function.variadic.is_none() =>
        {
            Ok(format!(
                "fn({}){}",
                function
                    .inputs
                    .iter()
                    .map(|arg| render_type(&arg.ty, resolver, scope))
                    .collect::<Result<Vec<_>, _>>()?
                    .join(","),
                render_return(&function.output, resolver, scope)?
            ))
        }
        Type::BareFn(_) => unsupported_type("qualified bare function"),
        Type::Infer(_) => Ok("_".to_owned()),
        Type::Never(_) => Ok("!".to_owned()),
        Type::Macro(_) => unsupported_type("macro"),
        Type::Verbatim(_) => unsupported_type("verbatim"),
        _ => unsupported_type("unknown"),
    }
}

fn render_path(path: &Path, resolver: &ImportResolver, scope: &[String]) -> Result<String, String> {
    let mut rendered = resolver.resolve_path(scope, path);
    for segment in &path.segments {
        rendered.push_str(&render_arguments(&segment.arguments, resolver, scope)?);
    }
    Ok(rendered)
}

fn render_arguments(
    args: &PathArguments,
    resolver: &ImportResolver,
    scope: &[String],
) -> Result<String, String> {
    match args {
        PathArguments::None => Ok(String::new()),
        PathArguments::AngleBracketed(args) => render_angle_arguments(args, resolver, scope),
        PathArguments::Parenthesized(args) => Ok(format!(
            "({}){}",
            args.inputs
                .iter()
                .map(|ty| render_type(ty, resolver, scope))
                .collect::<Result<Vec<_>, _>>()?
                .join(","),
            render_return(&args.output, resolver, scope)?
        )),
    }
}

fn render_angle_arguments(
    args: &AngleBracketedGenericArguments,
    resolver: &ImportResolver,
    scope: &[String],
) -> Result<String, String> {
    let rendered = args
        .args
        .iter()
        .map(|arg| render_generic_argument(arg, resolver, scope))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(format!("<{}>", rendered.join(",")))
}

fn render_generic_argument(
    arg: &GenericArgument,
    resolver: &ImportResolver,
    scope: &[String],
) -> Result<String, String> {
    match arg {
        GenericArgument::Lifetime(lifetime) => Ok(lifetime.to_string()),
        GenericArgument::Type(ty) => render_type(ty, resolver, scope),
        GenericArgument::Const(expr) => render_const_expr(expr, resolver, scope),
        GenericArgument::AssocType(assoc) => Ok(format!(
            "{}{}={}",
            assoc.ident,
            render_optional_arguments(assoc.generics.as_ref(), resolver, scope)?,
            render_type(&assoc.ty, resolver, scope)?
        )),
        GenericArgument::AssocConst(assoc) => Ok(format!(
            "{}{}={}",
            assoc.ident,
            render_optional_arguments(assoc.generics.as_ref(), resolver, scope)?,
            render_const_expr(&assoc.value, resolver, scope)?
        )),
        GenericArgument::Constraint(constraint) => Ok(format!(
            "{}{}:{}",
            constraint.ident,
            render_optional_arguments(constraint.generics.as_ref(), resolver, scope)?,
            render_bounds(&constraint.bounds, resolver, scope)?
        )),
        _ => Err("unsupported generic argument syntax".to_owned()),
    }
}

fn render_optional_arguments(
    args: Option<&AngleBracketedGenericArguments>,
    resolver: &ImportResolver,
    scope: &[String],
) -> Result<String, String> {
    match args {
        Some(args) => render_angle_arguments(args, resolver, scope),
        None => Ok(String::new()),
    }
}

fn render_bounds(
    bounds: &syn::punctuated::Punctuated<TypeParamBound, syn::Token![+]>,
    resolver: &ImportResolver,
    scope: &[String],
) -> Result<String, String> {
    bounds
        .iter()
        .map(|bound| match bound {
            TypeParamBound::Trait(bound) if bound.lifetimes.is_none() => {
                let modifier = match bound.modifier {
                    syn::TraitBoundModifier::None => "",
                    syn::TraitBoundModifier::Maybe(_) => "?",
                };
                let rendered = format!("{modifier}{}", render_path(&bound.path, resolver, scope)?);
                Ok(if bound.paren_token.is_some() {
                    format!("({rendered})")
                } else {
                    rendered
                })
            }
            TypeParamBound::Trait(_) => Err("unsupported higher-ranked trait bound".to_owned()),
            TypeParamBound::Lifetime(lifetime) => Ok(lifetime.to_string()),
            TypeParamBound::PreciseCapture(_) | TypeParamBound::Verbatim(_) => {
                Err("unsupported type bound syntax".to_owned())
            }
            _ => Err("unsupported type bound syntax".to_owned()),
        })
        .collect::<Result<Vec<_>, _>>()
        .map(|bounds| bounds.join("+"))
}

fn render_return(
    output: &ReturnType,
    resolver: &ImportResolver,
    scope: &[String],
) -> Result<String, String> {
    match output {
        ReturnType::Default => Ok(String::new()),
        ReturnType::Type(_, ty) => Ok(format!("->{}", render_type(ty, resolver, scope)?)),
    }
}

fn render_const_expr(
    expr: &Expr,
    resolver: &ImportResolver,
    scope: &[String],
) -> Result<String, String> {
    match expr {
        Expr::Lit(literal) => match &literal.lit {
            Lit::Int(value) => Ok(value.to_string()),
            Lit::Bool(value) => Ok(value.value.to_string()),
            _ => Err("unsupported const literal syntax".to_owned()),
        },
        Expr::Path(path) if path.qself.is_none() => render_path(&path.path, resolver, scope),
        Expr::Path(_) => Err("unsupported const expression qualified path".to_owned()),
        Expr::Paren(paren) => Ok(format!(
            "({})",
            render_const_expr(&paren.expr, resolver, scope)?
        )),
        Expr::Group(group) => render_const_expr(&group.expr, resolver, scope),
        Expr::Unary(unary) => {
            let operator = match unary.op {
                syn::UnOp::Deref(_) => "*",
                syn::UnOp::Not(_) => "!",
                syn::UnOp::Neg(_) => "-",
                _ => return Err("unsupported const unary operator".to_owned()),
            };
            Ok(format!(
                "{operator}{}",
                render_const_expr(&unary.expr, resolver, scope)?
            ))
        }
        Expr::Infer(_) => Ok("_".to_owned()),
        _ => Err("unsupported const expression syntax".to_owned()),
    }
}

fn unsupported_type(kind: &str) -> Result<String, String> {
    Err(format!("unsupported type syntax `{kind}`"))
}

fn path_boundary(path: &str, root: &str) -> bool {
    path == root
        || path
            .strip_prefix(root)
            .is_some_and(|suffix| suffix.starts_with(['<', ':']))
}

fn compact(text: &str) -> String {
    text.split_whitespace().collect()
}

#[test]
fn scope_canonicalizer_supports_approved_syntax_and_fails_closed() {
    let supported = r#"
        use serde_json::Value;
        type Payload<'a, const N: usize> =
            &'a mut std::borrow::Cow<'a, [Value; N]>;
    "#;
    let rendered = crate::visitor::fingerprints("canonical.rs", supported).unwrap();
    assert!(rendered.iter().any(|finding| {
        finding.ends_with("#Payload@1|alias|&'amutstd::borrow::Cow<'a,[serde_json::Value;N]>")
    }));

    let unsupported = "type Hidden = opaque!(serde_json::Value);";
    let error = crate::visitor::fingerprints("canonical.rs", unsupported).unwrap_err();
    assert!(error.contains("unsupported type syntax `macro`"));
}
