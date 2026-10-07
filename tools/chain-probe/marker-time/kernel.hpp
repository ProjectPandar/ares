float get_estimated_time_at(const std::vector<PathVertex>& vertices, size_t id, ETimeMode mode)
{
    return std::accumulate(vertices.begin(), vertices.begin() + id + 1, 0.0f, 
        [mode](float a, const PathVertex& v) { return a + v.times[static_cast<size_t>(mode)]; });
}
