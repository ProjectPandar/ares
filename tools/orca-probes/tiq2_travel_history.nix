# First-layer boundary construction/cache history, extending qualified travel probe.
let
  pkgs = import <nixpkgs> {};
  oracle = import ./tiq2_travel.nix;
in
oracle.overrideAttrs (old: {
  postPatch = (old.postPatch or "") + ''
    ${pkgs.python3}/bin/python3 - <<'PY'
from pathlib import Path
p = Path('src/libslic3r/GCode/AvoidCrossingPerimeters.cpp')
text = p.read_text()
start = text.index('Polyline AvoidCrossingPerimeters::travel_to(const GCode &gcodegen,')
stop = text.index('// ************************************* AvoidCrossingPerimeters::init_layer()', start)
body = text[start:stop]
anchor = '    bool                            is_support_layer = (dynamic_cast<const SupportLayer *>(gcodegen.layer()) != nullptr);'
assert body.count(anchor) == 1
body = body.replace(anchor, anchor + r"""
    const auto tiq2_cache_dump = [&](const char* phase, bool input) {
        const char* dump = std::getenv("ORCA_DUMP_TIQ2_TRAVEL");
        if (!dump || gcodegen.layer()->print_z >= 0.21) return;
        FILE* f = std::fopen(dump, "a");
        if (!f) return;
        std::fprintf(f, "TIQ2_CACHE phase=%s support=%d external=%d z=%.17g spacing=%.17g regions=%zu start=%lld,%lld end=%lld,%lld\n",
            phase, int(is_support_layer), int(use_external), double(gcodegen.layer()->print_z),
            double(get_perimeter_spacing(*gcodegen.layer())), gcodegen.layer()->regions().size(),
            (long long)start.x(), (long long)start.y(), (long long)end.x(), (long long)end.y());
        const auto polygons = [&](const char* label, const Polygons& ps) {
            for (size_t i = 0; i < ps.size(); ++i) {
                std::fprintf(f, "TIQ2_CACHE_POLYGON phase=%s kind=%s index=%zu points=", phase, label, i);
                for (const auto& p : ps[i].points)
                    std::fprintf(f, "%lld,%lld;", (long long)p.x(), (long long)p.y());
                std::fprintf(f, "\n");
            }
        };
        if (input) {
            polygons("lslices", to_polygons(gcodegen.layer()->lslices));
            if (is_support_layer) {
                const auto* below = gcodegen.layer()->object()->get_first_layer_bellow_printz(gcodegen.layer()->print_z, EPSILON);
                if (below) polygons("below_lslices", to_polygons(below->lslices));
            }
            for (const auto* region : gcodegen.layer()->regions())
                for (const auto& surface : region->fill_surfaces.surfaces)
                    if (surface.is_top()) polygons("top", to_polygons(surface.expolygon));
        } else {
            polygons("internal", m_internal.boundaries);
        }
        std::fclose(f);
    };
""")
call = '            init_boundary(&m_internal, to_polygons(get_boundary(*gcodegen.layer(), get_perimeter_spacing(*gcodegen.layer()))), {start, end});'
assert body.count(call) == 2
body = body.replace(call, '            tiq2_cache_dump("CONSTRUCT", true);\n' + call + '\n            tiq2_cache_dump("CACHED", false);')
text = text[:start] + body + text[stop:]
anchor = 'void AvoidCrossingPerimeters::init_layer(const Layer &layer)\n{\n    m_internal.clear();'
assert text.count(anchor) == 1
text = text.replace(anchor, r"""void AvoidCrossingPerimeters::init_layer(const Layer &layer)
{
    if (const char* dump = std::getenv("ORCA_DUMP_TIQ2_TRAVEL")) {
        if (layer.print_z < 0.21) {
            FILE* f = std::fopen(dump, "a");
            if (f) {
                std::fprintf(f, "TIQ2_INIT_LAYER support=%d z=%.17g spacing=%.17g regions=%zu previous_internal=%zu\n",
                    int(dynamic_cast<const SupportLayer*>(&layer) != nullptr), double(layer.print_z),
                    double(get_perimeter_spacing(layer)), layer.regions().size(), m_internal.boundaries.size());
                std::fclose(f);
            }
        }
    }
    m_internal.clear();""")
p.write_text(text)
PY
  '';
})
