    if (enabled_range[1] != m_viewer.get_view_visible_range()[1]) {
        const libvgcode::PathVertex& curr_vertex = m_viewer.get_current_vertex();
        if (curr_vertex.is_extrusion() || curr_vertex.is_travel() || curr_vertex.is_wipe() ||
            curr_vertex.type == libvgcode::EMoveType::Seam) {
            const libvgcode::ColorRange& color_range = m_viewer.get_color_range(libvgcode::EViewType::ActualSpeed);
            const std::array<float, 2>& interval = color_range.get_range();
            const size_t vertices_count = m_viewer.get_vertices_count();
            std::vector<SequentialView::ActualSpeedImguiWidget::Item> actual_speed_data;
            // collect vertices sharing the same gcode_id
            const size_t curr_id = m_viewer.get_current_vertex_id();
            size_t start_id = curr_id;
            while (start_id > 0) {
                --start_id;
                if (curr_vertex.gcode_id != m_viewer.get_vertex_at(start_id).gcode_id)
                    break;
            }
            size_t end_id = curr_id;
            while (end_id < vertices_count - 1) {
                ++end_id;
                if (curr_vertex.gcode_id != m_viewer.get_vertex_at(end_id).gcode_id)
                    break;
            }

            if (m_viewer.get_vertex_at(end_id - 1).type == libvgcode::convert(EMoveType::Seam))
                --end_id;

            assert(end_id - start_id >= 2);

            float total_len = 0.0f;
            for (size_t i = start_id; i < end_id; ++i) {
                const libvgcode::PathVertex& v = m_viewer.get_vertex_at(i);
                const float len = (i > start_id) ?
                    (libvgcode::convert(v.position) - libvgcode::convert(m_viewer.get_vertex_at(i - 1).position)).norm() : 0.0f;
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
            std::vector<SequentialView::ActualSpeedImguiWidget::Item> compressed;
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

            m_sequential_view.marker.set_actual_speed_data(compressed);
            m_sequential_view.marker.set_actual_speed_y_range(std::make_pair(interval[0], interval[1]));
            m_sequential_view.marker.set_actual_speed_levels(levels);
        }
    }
