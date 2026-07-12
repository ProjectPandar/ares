use super::super::{OptionDefinition, OptionValueKind};

macro_rules! definition {
    ($key:literal, $kind:ident, $default_value:literal, $source:literal $(,)?) => {
        OptionDefinition {
            key: $key,
            kind: OptionValueKind::$kind,
            default_value: $default_value,
            source: $source,
        }
    };
}

mod early;
mod late;
mod late_tail;
mod late_tail_after_material;
mod late_tail_after_pad;
mod late_tail_final;
mod late_tail_material;
mod middle;
mod middle_independent;
mod middle_tail;
mod pre_middle;
mod pre_middle_defaults;
mod pre_middle_filament;
mod pre_middle_process;
mod pre_middle_tail;
mod tail;
mod tail_final;
mod tail_raft;
mod tail_raft_suffix;
mod tail_small;
mod tail_support;
mod tail_terminal;
mod tail_terminal_suffix;
mod tail_terminal_wipe;
mod tail_z;

const OPTION_DEFINITION_COUNT: usize = early::EARLY_OPTION_DEFINITIONS.len()
    + pre_middle::PRE_MIDDLE_OPTION_DEFINITIONS.len()
    + pre_middle_defaults::PRE_MIDDLE_DEFAULT_OPTION_DEFINITIONS.len()
    + pre_middle_process::PRE_MIDDLE_PROCESS_OPTION_DEFINITIONS.len()
    + pre_middle_filament::PRE_MIDDLE_FILAMENT_OPTION_DEFINITIONS.len()
    + pre_middle_tail::PRE_MIDDLE_TAIL_OPTION_DEFINITIONS.len()
    + middle::MIDDLE_OPTION_DEFINITIONS.len()
    + middle_independent::MIDDLE_INDEPENDENT_OPTION_DEFINITIONS.len()
    + middle_tail::MIDDLE_TAIL_OPTION_DEFINITIONS.len()
    + late::LATE_OPTION_DEFINITIONS.len()
    + late_tail::LATE_TAIL_OPTION_DEFINITIONS.len()
    + late_tail_material::LATE_TAIL_MATERIAL_OPTION_DEFINITIONS.len()
    + late_tail_after_material::LATE_TAIL_AFTER_MATERIAL_OPTION_DEFINITIONS.len()
    + late_tail_after_pad::LATE_TAIL_AFTER_PAD_OPTION_DEFINITIONS.len()
    + late_tail_final::LATE_TAIL_FINAL_OPTION_DEFINITIONS.len()
    + tail::TAIL_OPTION_DEFINITIONS.len()
    + tail_raft::TAIL_RAFT_OPTION_DEFINITIONS.len()
    + tail_raft_suffix::TAIL_RAFT_SUFFIX_OPTION_DEFINITIONS.len()
    + tail_small::TAIL_SMALL_OPTION_DEFINITIONS.len()
    + tail_final::TAIL_FINAL_OPTION_DEFINITIONS.len()
    + tail_terminal::TAIL_TERMINAL_OPTION_DEFINITIONS.len()
    + tail_support::TAIL_SUPPORT_OPTION_DEFINITIONS.len()
    + tail_terminal_suffix::TAIL_TERMINAL_SUFFIX_OPTION_DEFINITIONS.len()
    + tail_terminal_wipe::TAIL_TERMINAL_WIPE_OPTION_DEFINITIONS.len()
    + tail_z::TAIL_Z_OPTION_DEFINITIONS.len();
const EMPTY_OPTION_DEFINITION: OptionDefinition = OptionDefinition {
    key: "",
    kind: OptionValueKind::String,
    default_value: "",
    source: "",
};
static OPTION_DEFINITIONS_ARRAY: [OptionDefinition; OPTION_DEFINITION_COUNT] = merge_definitions();

pub(in crate::options::registry) const OPTION_DEFINITIONS: &[OptionDefinition] =
    &OPTION_DEFINITIONS_ARRAY;

const fn merge_definitions() -> [OptionDefinition; OPTION_DEFINITION_COUNT] {
    let mut definitions = [EMPTY_OPTION_DEFINITION; OPTION_DEFINITION_COUNT];
    let mut index = 0;

    while index < early::EARLY_OPTION_DEFINITIONS.len() {
        definitions[index] = early::EARLY_OPTION_DEFINITIONS[index];
        index += 1;
    }

    let mut pre_middle_index = 0;
    while pre_middle_index < pre_middle::PRE_MIDDLE_OPTION_DEFINITIONS.len() {
        definitions[index] = pre_middle::PRE_MIDDLE_OPTION_DEFINITIONS[pre_middle_index];
        index += 1;
        pre_middle_index += 1;
    }

    let mut pre_middle_default_index = 0;
    while pre_middle_default_index
        < pre_middle_defaults::PRE_MIDDLE_DEFAULT_OPTION_DEFINITIONS.len()
    {
        definitions[index] =
            pre_middle_defaults::PRE_MIDDLE_DEFAULT_OPTION_DEFINITIONS[pre_middle_default_index];
        index += 1;
        pre_middle_default_index += 1;
    }

    let mut pre_middle_process_index = 0;
    while pre_middle_process_index < pre_middle_process::PRE_MIDDLE_PROCESS_OPTION_DEFINITIONS.len()
    {
        definitions[index] =
            pre_middle_process::PRE_MIDDLE_PROCESS_OPTION_DEFINITIONS[pre_middle_process_index];
        index += 1;
        pre_middle_process_index += 1;
    }

    let mut pre_middle_filament_index = 0;
    while pre_middle_filament_index
        < pre_middle_filament::PRE_MIDDLE_FILAMENT_OPTION_DEFINITIONS.len()
    {
        definitions[index] =
            pre_middle_filament::PRE_MIDDLE_FILAMENT_OPTION_DEFINITIONS[pre_middle_filament_index];
        index += 1;
        pre_middle_filament_index += 1;
    }

    let mut pre_middle_tail_index = 0;
    while pre_middle_tail_index < pre_middle_tail::PRE_MIDDLE_TAIL_OPTION_DEFINITIONS.len() {
        definitions[index] =
            pre_middle_tail::PRE_MIDDLE_TAIL_OPTION_DEFINITIONS[pre_middle_tail_index];
        index += 1;
        pre_middle_tail_index += 1;
    }

    let mut middle_index = 0;
    while middle_index < middle::MIDDLE_OPTION_DEFINITIONS.len() {
        definitions[index] = middle::MIDDLE_OPTION_DEFINITIONS[middle_index];
        index += 1;
        middle_index += 1;
    }

    let mut middle_independent_index = 0;
    while middle_independent_index < middle_independent::MIDDLE_INDEPENDENT_OPTION_DEFINITIONS.len()
    {
        definitions[index] =
            middle_independent::MIDDLE_INDEPENDENT_OPTION_DEFINITIONS[middle_independent_index];
        index += 1;
        middle_independent_index += 1;
    }

    let mut middle_tail_index = 0;
    while middle_tail_index < middle_tail::MIDDLE_TAIL_OPTION_DEFINITIONS.len() {
        definitions[index] = middle_tail::MIDDLE_TAIL_OPTION_DEFINITIONS[middle_tail_index];
        index += 1;
        middle_tail_index += 1;
    }

    let mut late_index = 0;
    while late_index < late::LATE_OPTION_DEFINITIONS.len() {
        definitions[index] = late::LATE_OPTION_DEFINITIONS[late_index];
        index += 1;
        late_index += 1;
    }

    let mut late_tail_index = 0;
    while late_tail_index < late_tail::LATE_TAIL_OPTION_DEFINITIONS.len() {
        definitions[index] = late_tail::LATE_TAIL_OPTION_DEFINITIONS[late_tail_index];
        index += 1;
        late_tail_index += 1;
    }

    let mut late_tail_material_index = 0;
    while late_tail_material_index < late_tail_material::LATE_TAIL_MATERIAL_OPTION_DEFINITIONS.len()
    {
        definitions[index] =
            late_tail_material::LATE_TAIL_MATERIAL_OPTION_DEFINITIONS[late_tail_material_index];
        index += 1;
        late_tail_material_index += 1;
    }

    let mut late_tail_after_material_index = 0;
    while late_tail_after_material_index
        < late_tail_after_material::LATE_TAIL_AFTER_MATERIAL_OPTION_DEFINITIONS.len()
    {
        definitions[index] = late_tail_after_material::LATE_TAIL_AFTER_MATERIAL_OPTION_DEFINITIONS
            [late_tail_after_material_index];
        index += 1;
        late_tail_after_material_index += 1;
    }

    let mut late_tail_after_pad_index = 0;
    while late_tail_after_pad_index
        < late_tail_after_pad::LATE_TAIL_AFTER_PAD_OPTION_DEFINITIONS.len()
    {
        definitions[index] =
            late_tail_after_pad::LATE_TAIL_AFTER_PAD_OPTION_DEFINITIONS[late_tail_after_pad_index];
        index += 1;
        late_tail_after_pad_index += 1;
    }

    let mut late_tail_final_index = 0;
    while late_tail_final_index < late_tail_final::LATE_TAIL_FINAL_OPTION_DEFINITIONS.len() {
        definitions[index] =
            late_tail_final::LATE_TAIL_FINAL_OPTION_DEFINITIONS[late_tail_final_index];
        index += 1;
        late_tail_final_index += 1;
    }

    let mut tail_index = 0;
    while tail_index < tail::TAIL_OPTION_DEFINITIONS.len() {
        definitions[index] = tail::TAIL_OPTION_DEFINITIONS[tail_index];
        index += 1;
        tail_index += 1;
    }

    let mut tail_raft_index = 0;
    while tail_raft_index < tail_raft::TAIL_RAFT_OPTION_DEFINITIONS.len() {
        definitions[index] = tail_raft::TAIL_RAFT_OPTION_DEFINITIONS[tail_raft_index];
        index += 1;
        tail_raft_index += 1;
    }

    let mut tail_raft_suffix_index = 0;
    while tail_raft_suffix_index < tail_raft_suffix::TAIL_RAFT_SUFFIX_OPTION_DEFINITIONS.len() {
        definitions[index] =
            tail_raft_suffix::TAIL_RAFT_SUFFIX_OPTION_DEFINITIONS[tail_raft_suffix_index];
        index += 1;
        tail_raft_suffix_index += 1;
    }

    let mut tail_small_index = 0;
    while tail_small_index < tail_small::TAIL_SMALL_OPTION_DEFINITIONS.len() {
        definitions[index] = tail_small::TAIL_SMALL_OPTION_DEFINITIONS[tail_small_index];
        index += 1;
        tail_small_index += 1;
    }

    let mut tail_final_index = 0;
    while tail_final_index < tail_final::TAIL_FINAL_OPTION_DEFINITIONS.len() {
        definitions[index] = tail_final::TAIL_FINAL_OPTION_DEFINITIONS[tail_final_index];
        index += 1;
        tail_final_index += 1;
    }

    let mut tail_terminal_index = 0;
    while tail_terminal_index < tail_terminal::TAIL_TERMINAL_OPTION_DEFINITIONS.len() {
        definitions[index] = tail_terminal::TAIL_TERMINAL_OPTION_DEFINITIONS[tail_terminal_index];
        index += 1;
        tail_terminal_index += 1;
    }

    let mut tail_support_index = 0;
    while tail_support_index < tail_support::TAIL_SUPPORT_OPTION_DEFINITIONS.len() {
        definitions[index] = tail_support::TAIL_SUPPORT_OPTION_DEFINITIONS[tail_support_index];
        index += 1;
        tail_support_index += 1;
    }

    let mut tail_terminal_suffix_index = 0;
    while tail_terminal_suffix_index
        < tail_terminal_suffix::TAIL_TERMINAL_SUFFIX_OPTION_DEFINITIONS.len()
    {
        definitions[index] = tail_terminal_suffix::TAIL_TERMINAL_SUFFIX_OPTION_DEFINITIONS
            [tail_terminal_suffix_index];
        index += 1;
        tail_terminal_suffix_index += 1;
    }

    let mut tail_terminal_wipe_index = 0;
    while tail_terminal_wipe_index < tail_terminal_wipe::TAIL_TERMINAL_WIPE_OPTION_DEFINITIONS.len()
    {
        definitions[index] =
            tail_terminal_wipe::TAIL_TERMINAL_WIPE_OPTION_DEFINITIONS[tail_terminal_wipe_index];
        index += 1;
        tail_terminal_wipe_index += 1;
    }

    let mut tail_z_index = 0;
    while tail_z_index < tail_z::TAIL_Z_OPTION_DEFINITIONS.len() {
        definitions[index] = tail_z::TAIL_Z_OPTION_DEFINITIONS[tail_z_index];
        index += 1;
        tail_z_index += 1;
    }

    definitions
}
