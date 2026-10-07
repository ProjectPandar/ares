float ViewerImpl::get_estimated_time_at(size_t id) const
{
    return std::accumulate(m_vertices.begin(), m_vertices.begin() + id + 1, 0.0f, 
        [this](float a, const PathVertex& v) { return a + v.times[static_cast<size_t>(m_settings.time_mode)]; });
}
