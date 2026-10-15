# ARD: Config resolution is observable parity, not a prefiltered prerequisite

Status: amended proposal, docs-only [SPEC](perimeter-input-dispatch-spec.md) review
required. Seam v2 NO-RED closure is already accepted and remains unchanged.

## Source owner and Rust destination

Orca8500fcdcc: bbs_3mf.cpp::_BBS_3MF_Importer::_load_model_from_file1923-1939 /
_extract_project_config_from_archive2632-2654, Config.cpp::ConfigBase::load_from_json
820/925-935, PresetBundle.cpp::load_vendor_configs_from_json4886-4927 and
load_config_file_config4301-4319, PrintConfig.cpp::init_fff_params defaults.
Native CLI update_full_config2750-2812/process3011 supplies entry precedence;
that glue is provenance, not an independent Ares architecture goal.
Rust: project/load.rs::load_project_with_preparation118-150 plus typed builders;
loaded-source route profiles/load_settings.rs69-124/catalog/compose.rs108-136 and
project/domain/profile_input.rs::from_3mf_with_profiles44-110. Reuse byte-oriented
existing APIs and composition; no new resolver, filesystem or public API.

## Non-negotiable decisions

- Effective config divergence under the same requested source/precedence contract
  IS a RED candidate. Do not require equal actual configs before comparing output;
  that would hide config-resolution bugs. Retained single-control stays the
  effective-config plus complete ordered G-code candidate.
- Record ALL supplied sources: 3MF, loaded preset files, caller overrides, entry
  and source-selection metadata. Identical 3MF bytes do not erase additional
  Native --load-settings inputs. Request provenance is not actual-config matching.
- Here five Ares values equal explicit embedded JSON; Native values equal explicit
  loaded process JSON. Defaults/inherits are not a demonstrated substitute source.
  Do not invent missing parent lookup, overwrite explicit document fields from
  named preset IDs, or repair downstream perimeter choice on this evidence.
- Review must resolve standalone versus loaded-source contract before a new RED
  run. Do not silently replay bare API against an overlaid cached Native oracle,
  or add fixture/override manipulation to force actual fields equal.
- Preserve effective-config witnesses AND all movement/arcs/M73/statistics/footer
  in acceptance. Native primary streams remain immutable; motion projection is
  diagnostic only. Failed harness histories are retained, never retry-to-success.
- Production scope is only a demonstrated config-source/precedence rewrite at the
  cited boundary after SPEC/test-only RED approval. Seam, ordering, brim geometry,
  travel and broad preset migration are deferred. No guards or fallback.

## Trade-off

The rejected matched-effective-config admission gate would hide the very bug
under investigation. Removing it must not replace evidence with a causal claim:
retained invocations supply different sources, so the exact contract remains a
review decision. Config delta is preserved rather than normalized or discarded;
no production correction is preselected and no new execution budget is implied.
