# Read-only extension of the whole-byte qualified seed-replay TiQ2 probe.
let
  pkgs = import <nixpkgs> {};
  oracle = import ./tiq2_order.nix;
in
oracle.overrideAttrs (old: {
  postPatch = (old.postPatch or "") + ''
    ${pkgs.python3}/bin/python3 - <<'PY'
from pathlib import Path

def patch_function(path, begin, end, edits):
    p = Path(path)
    text = p.read_text()
    start = text.index(begin)
    stop = text.index(end, start)
    body = text[start:stop]
    for before, after in edits:
        assert body.count(before) == 1, (path, before)
        body = body.replace(before, after)
    p.write_text('#include <cstdio>\n#include <cstdlib>\n#include <cmath>\n' + text[:start] + body + text[stop:])

patch_function('src/libslic3r/GCode.cpp',
    'std::string GCode::travel_to(const Point& point,',
    '//BBS\nLiftType GCode::to_lift_type', [
    ('    Polyline travel { this->last_pos(), point };', r"""    Polyline travel { this->last_pos(), point };
    const Vec2d tiq2_destination = this->point_to_gcode(point);
    const char* tiq2_dump_path = std::getenv("ORCA_DUMP_TIQ2_TRAVEL");
    const bool tiq2_trace = tiq2_dump_path && m_layer && m_layer->print_z < 0.21
        && std::fabs(tiq2_destination.x() - 169.301) < 0.005
        && std::fabs(tiq2_destination.y() - 169.301) < 0.005;
    const auto tiq2_dump = [&](const char* stage, const Polyline& path) {
        if (!tiq2_trace) return;
        FILE* f = std::fopen(tiq2_dump_path, "a");
        if (!f) return;
        std::fprintf(f, "TIQ2_TRAVEL stage=%s role=%d z=%.17g origin=%.17g,%.17g last=%lld,%lld external=%d once=%d disabled=%d points=%zu\n",
            stage, int(role), double(m_layer->print_z), m_origin.x(), m_origin.y(),
            (long long)this->last_pos().x(), (long long)this->last_pos().y(),
            int(m_avoid_crossing_perimeters.used_external_mp()),
            int(m_avoid_crossing_perimeters.used_external_mp_once()),
            int(m_avoid_crossing_perimeters.disabled_once()), path.points.size());
        for (const auto& p : path.points) {
            const Vec2d xy = this->point_to_gcode(p);
            std::fprintf(f, "TIQ2_TRAVEL_POINT stage=%s raw=%lld,%lld gcode=%.17g,%.17g\n",
                stage, (long long)p.x(), (long long)p.y(), xy.x(), xy.y());
        }
        std::fclose(f);
    };
    tiq2_dump("ENTER", travel);"""),
    ('    // Re-allow reduce_crossing_wall for the next travel moves',
     '    tiq2_dump("PRE_RETRACT", travel);\n\n    // Re-allow reduce_crossing_wall for the next travel moves'),
    ('    // use G1 because we rely on paths being straight (G0 may make round paths)',
     '    tiq2_dump("FINAL", travel);\n\n    // use G1 because we rely on paths being straight (G0 may make round paths)'),
])

patch_function('src/libslic3r/GCode/AvoidCrossingPerimeters.cpp',
    'Polyline AvoidCrossingPerimeters::travel_to(const GCode &gcodegen,',
    '// ************************************* AvoidCrossingPerimeters::init_layer()', [
    ('    const ConfigOptionFloatOrPercent &opt_max_detour', r"""    const char* tiq2_dump_path = std::getenv("ORCA_DUMP_TIQ2_TRAVEL");
    const double tiq2_x = unscale<double>(point.x()) + gcodegen.origin().x();
    const double tiq2_y = unscale<double>(point.y()) + gcodegen.origin().y();
    const bool tiq2_trace = tiq2_dump_path && gcodegen.layer()->print_z < 0.21
        && std::fabs(tiq2_x - 169.301) < 0.005 && std::fabs(tiq2_y - 169.301) < 0.005;
    if (tiq2_trace) {
        FILE* f = std::fopen(tiq2_dump_path, "a");
        if (f) {
            std::fprintf(f, "TIQ2_ACP external=%d support=%d origin=%.17g,%.17g start=%lld,%lld end=%lld,%lld intersections=%zu spacing=%.17g\n",
                int(use_external), int(is_support_layer), gcodegen.origin().x(), gcodegen.origin().y(),
                (long long)start.x(), (long long)start.y(), (long long)end.x(), (long long)end.y(),
                travel_intersection_count, double(get_perimeter_spacing(*gcodegen.layer())));
            const auto polygons = [&](const char* label, const Polygons& ps) {
                for (size_t i = 0; i < ps.size(); ++i) {
                    std::fprintf(f, "TIQ2_POLYGON kind=%s index=%zu points=", label, i);
                    for (const auto& p : ps[i].points)
                        std::fprintf(f, "%lld,%lld;", (long long)p.x(), (long long)p.y());
                    std::fprintf(f, "\n");
                }
            };
            polygons("lslices", to_polygons(lslices));
            polygons("safe_zone", to_polygons(m_lslices_offset));
            polygons("internal", m_internal.boundaries);
            polygons("external", m_external.boundaries);
            for (const auto& p : result_pl.points)
                std::fprintf(f, "TIQ2_ACP_POINT stage=PLANNED raw=%lld,%lld\n", (long long)p.x(), (long long)p.y());
            std::fclose(f);
        }
    }
    const ConfigOptionFloatOrPercent &opt_max_detour"""),
    ('    return result_pl;', r"""    if (tiq2_trace) {
        FILE* f = std::fopen(tiq2_dump_path, "a");
        if (f) {
            std::fprintf(f, "TIQ2_ACP_FINAL exceeded=%d wipe_disabled=%d\n", int(max_detour_length_exceeded), int(*could_be_wipe_disabled));
            for (const auto& p : result_pl.points)
                std::fprintf(f, "TIQ2_ACP_POINT stage=FINAL raw=%lld,%lld\n", (long long)p.x(), (long long)p.y());
            std::fclose(f);
        }
    }
    return result_pl;"""),
])
PY
  '';
})
