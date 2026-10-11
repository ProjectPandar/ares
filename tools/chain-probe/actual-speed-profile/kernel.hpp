void update_actual_speed(const std::vector<PathVertex>& vertices, const libvgcode::Interval& enabled_range,
    size_t visible_last, const libvgcode::ColorRange& actual_speed_range, Widget& widget)
{
    if (enabled_range[1] != visible_last) {
        const libvgcode::PathVertex& curr_vertex = vertices.at(visible_last);
        if (curr_vertex.is_extrusion() || curr_vertex.is_travel() || curr_vertex.is_wipe() ||
            curr_vertex.type == libvgcode::EMoveType::Seam) {
            const libvgcode::ColorRange& color_range = actual_speed_range;
            const std::array<float, 2>& interval = color_range.get_range();
            const size_t vertices_count = vertices.size();
            std::vector<Item> actual_speed_data;
            // collect vertices sharing the same gcode_id
            const size_t curr_id = visible_last;
            size_t start_id = curr_id;
            while (start_id > 0) {
                --start_id;
                if (curr_vertex.gcode_id != vertices.at(start_id).gcode_id)
                    break;
            }
            size_t end_id = curr_id;
            while (end_id < vertices_count - 1) {
                ++end_id;
                if (curr_vertex.gcode_id != vertices.at(end_id).gcode_id)
                    break;
            }

            if (vertices.at(end_id - 1).type == libvgcode::convert(EMoveType::Seam))
                --end_id;

            assert(end_id - start_id >= 2);

            float total_len = 0.0f;
            for (size_t i = start_id; i < end_id; ++i) {
                const libvgcode::PathVertex& v = vertices.at(i);
                const float len = (i > start_id) ?
                    (libvgcode::convert(v.position) - libvgcode::convert(vertices.at(i - 1).position)).norm() : 0.0f;
                total_len += len;
                if (i == start_id || len > EPSILON)
                    actual_speed_data.push_back({ total_len, v.actual_feedrate, v.times[0] == 0.0f });
            }

            std::vector<std::pair<float, ColorRGBA>> levels;
            const std::vector<float> values = color_range.get_values();
            for (float value : values) {
                levels.push_back(std::make_pair(value, libvgcode::convert(color_range.get_color_at(value))));
                levels.back().second.a(0.5f);
            }

            // ORCA Compress consecutive duplicate speeds with 0.1 precision
            auto sameSpeed = [](float a, float b) {
                return static_cast<int>(std::roundf(a * 10.0f)) == static_cast<int>(std::roundf(b * 10.0f));
            };
            std::vector<Item> compressed;
            if (!actual_speed_data.empty()) {
                compressed.push_back(actual_speed_data[0]);
                for (int i = 1; i < (int)actual_speed_data.size(); ++i) {
                    const bool same_as_prev = sameSpeed(actual_speed_data[i].speed, actual_speed_data[i - 1].speed);
                    const bool same_as_next = (i + 1 < (int)actual_speed_data.size()) && sameSpeed(actual_speed_data[i].speed, actual_speed_data[i + 1].speed);
                    if (!same_as_prev) {
                        if (!sameSpeed(compressed.back().speed, actual_speed_data[i - 1].speed))
                            compressed.push_back(actual_speed_data[i - 1]);
                        compressed.push_back(actual_speed_data[i]);
                    } else if (!same_as_next)
                        compressed.push_back(actual_speed_data[i]);
                }
                if (compressed.back().pos != actual_speed_data.back().pos)
                    compressed.push_back(actual_speed_data.back());
            }

            widget.data = compressed;
            widget.y_range = std::make_pair(interval[0], interval[1]);
            widget.levels = levels;
        }
    }
}
