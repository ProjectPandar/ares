float ViewerImpl::get_estimated_time_at(size_t id) const
{
    const size_t mode = static_cast<size_t>(m_settings.time_mode);
    if (mode >= TIME_MODES_COUNT || id >= m_vertices.size())
        return 0.0f;
    size_t first = 0;
    float time = 0.0f;
    const size_t layer = static_cast<size_t>(m_vertices[id].layer_id);
    if (layer < m_layer_first_vertex.size() && m_layer_first_vertex[layer] <= id) {
        first = m_layer_first_vertex[layer];
        time = m_layer_start_times[mode][layer];
    }
    for (size_t i = first; i <= id; ++i)
        time += m_vertices[i].times[mode];
    return time;
}
