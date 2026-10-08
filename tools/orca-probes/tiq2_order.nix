let
  pkgs = import <nixpkgs> {};
  # Without instrumentation this reproduces the oracle's exact .drv identity:
  # /nix/store/72d4hac8qmg5p9m22v2kifbp1hkxjk63-orca-slicer-seed-replay-2.4.2.drv
  seedOracle = pkgs.orca-slicer.overrideAttrs (old: {
    pname = "orca-slicer-seed-replay";
    src = builtins.storePath "/nix/store/g3wj93lyhjsvc32fmgvw0cdbd6191q55-orca-slicer-2.4.2-seed-replay-source";
  });
in
seedOracle.overrideAttrs (old: {
  postPatch = (old.postPatch or "") + ''
    ${pkgs.python3}/bin/python3 - <<'PY'
from pathlib import Path

def replace(path, old, new):
    p = Path(path)
    text = p.read_text()
    assert text.count(old) == 1, (path, old)
    p.write_text(text.replace(old, new))

replace('src/libslic3r/Print.cpp',
        'm_tool_ordering.clear();\n        if (this->has_wipe_tower()) {',
        """m_tool_ordering.clear();
        if (const char* dump = getenv("ORCA_DUMP_TIQ2_ORDER")) {
            FILE* f = fopen(dump, "a");
            if (f) {
                fprintf(f, "PRINT_GATE enable=%d has_tower=%d type=%d diameters=%zu colours=%zu nozzles=%zu spiral=%d timelapse=%d\\n",
                    int(m_config.enable_prime_tower.value), int(this->has_wipe_tower()), int(this->wipe_tower_type()),
                    m_config.filament_diameter.values.size(), m_config.filament_colour.values.size(),
                    m_config.nozzle_diameter.values.size(), int(m_config.spiral_mode.value), int(m_config.timelapse_type.value));
                fclose(f);
            }
        }
        if (this->has_wipe_tower()) {""")
replace('src/libslic3r/PrintConfig.cpp',
        't_config_option_keys DynamicPrintConfig::normalize_fdm_2(int num_objects, int used_filaments)\n{\n    t_config_option_keys changed_keys;',
        """t_config_option_keys DynamicPrintConfig::normalize_fdm_2(int num_objects, int used_filaments)
{
    if (const char* dump = getenv("ORCA_DUMP_TIQ2_ORDER")) {
        FILE* f = fopen(dump, "a");
        if (f) {
            fprintf(f, "NORMALIZE_FDM2_ENTER objects=%d used=%d enable=%d sequence=%d timelapse=%d wrapping=%d\\n",
                num_objects, used_filaments, int(this->opt_bool("enable_prime_tower")),
                int(this->opt_enum<PrintSequence>("print_sequence")),
                int(this->opt_enum<TimelapseType>("timelapse_type")), int(this->opt_bool("enable_wrapping_detection")));
            fclose(f);
        }
    }
    t_config_option_keys changed_keys;""")
replace('src/libslic3r/PrintConfig.cpp',
        '    return changed_keys;\n}\n\nvoid  handle_legacy_sla(DynamicPrintConfig &config)',
        """    if (const char* dump = getenv("ORCA_DUMP_TIQ2_ORDER")) {
        FILE* f = fopen(dump, "a");
        if (f) {
            fprintf(f, "NORMALIZE_FDM2_EXIT objects=%d used=%d enable=%d changed=", num_objects,
                used_filaments, int(this->opt_bool("enable_prime_tower")));
            for (const auto& key : changed_keys) fprintf(f, "%s,", key.c_str());
            fprintf(f, "\\n");
            fclose(f);
        }
    }
    return changed_keys;
}

void  handle_legacy_sla(DynamicPrintConfig &config)""")
replace('src/libslic3r/GCode/ToolOrdering.cpp',
        '\n}\n\nvoid ToolOrdering::cal_most_used_extruder(const PrintConfig &config)',
        """
    if (const char* dump = getenv("ORCA_DUMP_TIQ2_ORDER")) {
        FILE* f = fopen(dump, "a");
        if (f) {
            fprintf(f, "STATISTICS prime=%d first=%u last=%u has_tower=%d all=", int(prime_multi_material),
                m_first_printing_extruder, m_last_printing_extruder, int(this->has_wipe_tower()));
            for (auto id : m_all_printing_extruders) fprintf(f, "%u,", id);
            fprintf(f, "\\n");
            for (size_t i = 0; i < m_layer_tools.size(); ++i) {
                const auto& lt = m_layer_tools[i];
                fprintf(f, "LAYER index=%zu z=%.9g object=%d support=%d tower=%d partitions=%zu extruders=",
                    i, double(lt.print_z), int(lt.has_object), int(lt.has_support), int(lt.has_wipe_tower), lt.wipe_tower_partitions);
                for (auto id : lt.extruders) fprintf(f, "%u,", id);
                fprintf(f, "\\n");
            }
            fclose(f);
        }
    }
}

void ToolOrdering::cal_most_used_extruder(const PrintConfig &config)""")
PY
  '';
})
