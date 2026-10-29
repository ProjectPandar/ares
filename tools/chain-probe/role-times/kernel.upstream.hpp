    for (size_t i = 0; i < m_vertices.size(); ++i) {
        const PathVertex& v = m_vertices[i];
        for (size_t j = 0; j < TIME_MODES_COUNT; ++j) {
            m_total_time[j] += v.times[j];
            if (v.type == EMoveType::Travel)
                m_travels_time[j] += v.times[j];
        }
        if (v.type == EMoveType::Extrude) {
            m_extrusion_roles.add(v.role, v.times);
    const float total_estimated_time = time_mode.time > 0.0f ? time_mode.time : m_viewer.get_estimated_time();
    auto role_time_and_percent = [this, total_estimated_time](libvgcode::EGCodeExtrusionRole role) {
        const float time = m_viewer.get_extrusion_role_estimated_time(role);
        return std::make_pair(time, total_estimated_time > 0.0f ? time / total_estimated_time : 0.0f);
    };

    auto travel_time_and_percent = [this, total_estimated_time]() {
        const float time = m_viewer.get_travels_estimated_time();
        return std::make_pair(time, total_estimated_time > 0.0f ? time / total_estimated_time : 0.0f);
    };
        char buffer[64];
        const std::vector<libvgcode::EGCodeExtrusionRole>& roles = m_viewer.get_extrusion_roles();
        for (libvgcode::EGCodeExtrusionRole role : roles) {
            assert(static_cast<size_t>(role) < libvgcode::GCODE_EXTRUSION_ROLES_COUNT);
            if (static_cast<size_t>(role) < libvgcode::GCODE_EXTRUSION_ROLES_COUNT) {
                labels.push_back(_u8L(ExtrusionEntity::role_to_string(convert(role))));
                auto [time, percent] = role_time_and_percent(role);
                times.push_back((time > 0.0f) ? short_time(get_time_dhms(time)) : "");
                if (percent == 0) // ORCA remove % symbol from rows
                    ::sprintf(buffer, "0");
                else
                    percent > 0.001 ? ::sprintf(buffer, "%.1f", percent * 100) : ::sprintf(buffer, "<0.1");
                percents.push_back(buffer);
        //BBS: get travel time and percent
        {
            auto [time, percent] = travel_time_and_percent();
            travel_time = (time > 0.0f) ? short_time(get_time_dhms(time)) : "";
            if (percent == 0) // ORCA remove % symbol from rows
                ::sprintf(buffer, "0");
            else
                percent > 0.001 ? ::sprintf(buffer, "%.1f", percent * 100) : ::sprintf(buffer, "<0.1");
            travel_percent = buffer;
            percents.push_back(travel_percent);
