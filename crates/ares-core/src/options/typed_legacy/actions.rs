mod rules;

pub(crate) use rules::EXPLICIT_RULES;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct Replacement {
    pub(crate) from: &'static str,
    pub(crate) to: &'static str,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Comparison {
    Exact,
    AsciiCaseInsensitive,
    Leading,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum LegacyAction {
    Rename {
        target: &'static str,
    },
    FeatureFilament {
        target: &'static str,
        legacy_inherit: &'static str,
        canonical_inherit: &'static str,
    },
    ConsumeIfContains {
        needle: &'static str,
    },
    TopOneWall {
        target: &'static str,
        consume: &'static str,
        replacement: &'static str,
    },
    PrimeTowerRib {
        target: &'static str,
        trigger: &'static str,
        replacement: &'static str,
    },
    Rewrite {
        target: &'static str,
        comparison: Comparison,
        replacements: &'static [Replacement],
    },
    WallOrder {
        target: &'static str,
        replacements: &'static [Replacement],
    },
    ReplaceAll {
        target: &'static str,
        replacements: &'static [Replacement],
    },
    FilamentTokenRebuild {
        target: &'static str,
        from: &'static str,
        to: &'static str,
    },
    DeferredProfileBookkeeping {
        target: Option<&'static str>,
        recursive: bool,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum VectorType {
    Ints,
    Bools,
    Enums,
    Strings,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum StringAllowance {
    Execute,
    Deferred,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum JsonArrayAllowance {
    RejectAfterFirstPass,
    Flatten(VectorType),
    ConsumeFirstPass,
    Deferred,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum EmptyValueAction {
    Retain {
        target: &'static str,
        value: &'static str,
    },
    Consume,
    Deferred,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct WireContract {
    pub(crate) json_string: StringAllowance,
    pub(crate) xml_string: StringAllowance,
    pub(crate) json_array: JsonArrayAllowance,
    pub(crate) vector: Option<VectorType>,
    pub(crate) empty_first_pass: EmptyValueAction,
}

impl WireContract {
    const fn scalar(target: &'static str, value: &'static str) -> Self {
        Self {
            json_string: StringAllowance::Execute,
            xml_string: StringAllowance::Execute,
            json_array: JsonArrayAllowance::RejectAfterFirstPass,
            vector: None,
            empty_first_pass: EmptyValueAction::Retain { target, value },
        }
    }

    const fn vector(target: &'static str, vector: VectorType) -> Self {
        Self {
            json_array: JsonArrayAllowance::Flatten(vector),
            vector: Some(vector),
            ..Self::scalar(target, "")
        }
    }

    pub(crate) const fn deferred() -> Self {
        Self {
            json_string: StringAllowance::Deferred,
            xml_string: StringAllowance::Deferred,
            json_array: JsonArrayAllowance::Deferred,
            vector: None,
            empty_first_pass: EmptyValueAction::Deferred,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct JsonDerivedEffect {
    pub(crate) triggers: &'static [&'static str],
    pub(crate) target: &'static str,
    pub(crate) value: &'static str,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum RecursionContract {
    SinglePass,
    RecursiveBookkeeping,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct LegacyRule {
    pub(crate) source: &'static str,
    pub(crate) action: LegacyAction,
    pub(crate) wire: WireContract,
    pub(crate) json_effect: Option<JsonDerivedEffect>,
    pub(crate) recursion: RecursionContract,
}

const fn scalar(source: &'static str, action: LegacyAction, target: &'static str) -> LegacyRule {
    LegacyRule {
        source,
        action,
        wire: WireContract::scalar(target, ""),
        json_effect: None,
        recursion: RecursionContract::SinglePass,
    }
}

const fn vector(
    source: &'static str,
    action: LegacyAction,
    target: &'static str,
    kind: VectorType,
) -> LegacyRule {
    LegacyRule {
        wire: WireContract::vector(target, kind),
        ..scalar(source, action, target)
    }
}

const fn rename(source: &'static str, target: &'static str) -> LegacyRule {
    scalar(source, LegacyAction::Rename { target }, target)
}

const fn feature(source: &'static str, target: &'static str) -> LegacyRule {
    scalar(
        source,
        LegacyAction::FeatureFilament {
            target,
            legacy_inherit: "1",
            canonical_inherit: "0",
        },
        target,
    )
}
