constexpr double epsilon() { return 0.0011; }

static int find_close_layer_idx(const std::vector<double> &zs, double &z, double eps)
{
    if (zs.empty()) return -1;
    auto it_h = std::lower_bound(zs.begin(), zs.end(), z);
    if (it_h == zs.end()) {
        auto it_l = it_h;
        --it_l;
        if (z - *it_l < eps) return int(zs.size() - 1);
    } else if (it_h == zs.begin()) {
        if (*it_h - z < eps) return 0;
    } else {
        auto it_l = it_h;
        --it_l;
        double dist_l = z - *it_l;
        double dist_h = *it_h - z;
        if (std::min(dist_l, dist_h) < eps) { return (dist_l < dist_h) ? int(it_l - zs.begin()) : int(it_h - zs.begin()); }
    }
    return -1;
}

    template <class Viewer> std::vector<double> get_layers_zs(const Viewer &m_viewer) {
        // Belt printers: the layer Z the slider labels and the colour-change ticks
        // use is the layer's print Z (see load_as_gcode), not a toolpath height.
        if (! m_belt_layer_zs.empty())
            return m_belt_layer_zs;
        const std::vector<float> zs = m_viewer.get_layers_zs();
        std::vector<double> ret;
        std::transform(zs.begin(), zs.end(), std::back_inserter(ret), [](float z) { return static_cast<double>(z); });
        return ret;
    }

template <class Viewer, class Imgui>
void render_custom_gcode_overview(const std::vector<Slic3r::CustomGCode::Item> &custom_gcode_per_print_z, const Viewer &m_viewer, Imgui &imgui, float window_padding, float max_len)
{
        // Types of custom gcode
        std::string cgcode_pause_str = _u8L("Pause");
        std::string cgcode_template_str= _u8L("Template");
        std::string cgcode_toolchange_str = _u8L("Tool Change");
        std::string cgcode_custom_str = _u8L("Custom");
        std::string cgcode_unknown_str = _u8L("Unknown");

        // ORCA: Get layer Zs as doubles
        std::vector<double> layer_zs = get_layers_zs(m_viewer);
        // loop invariant, same reason as the layer Zs above
        const std::vector<float> layer_times = m_viewer.get_layers_estimated_times();

        for (Slic3r::CustomGCode::Item custom_gcode : custom_gcode_per_print_z) {
            ImGui::Dummy({window_padding, window_padding});
            ImGui::SameLine();

            switch (custom_gcode.type) {
            case CustomGCode::PausePrint: imgui.text(cgcode_pause_str); break;
            case CustomGCode::Template: imgui.text(cgcode_template_str); break;
            case CustomGCode::ToolChange: imgui.text(cgcode_toolchange_str); break;
            case CustomGCode::Custom: imgui.text(cgcode_custom_str); break;
            default: imgui.text(cgcode_unknown_str); break;
            }
            ImGui::SameLine(max_len);
            char buf[64];
            int layer = find_close_layer_idx(layer_zs, custom_gcode.print_z, epsilon()); // ORCA: find layer index by Z
            ::sprintf(buf, "%d", layer + 1); // +1 because layer 0 is the first layer
            imgui.text(buf);
            ImGui::SameLine(max_len * 1.5);

            float custom_gcode_time = 0;
            if (layer > 0)
            {
                for (int i = 0; i < layer; i++) { // sum all previous layers time
                    custom_gcode_time += layer_times[i];
                }
            }
            imgui.time(custom_gcode_time);
            imgui.text(short_time(get_time_dhms(custom_gcode_time)));

        }

}
