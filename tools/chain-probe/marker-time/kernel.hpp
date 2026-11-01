float get_estimated_time_at(const std::vector<PathVertex>& vertices, const std::vector<uint32_t>& layer_first_vertex, const std::array<std::vector<float>, TIME_MODES_COUNT>& layer_start_times, size_t id, ETimeMode time_mode)
{
    const size_t mode = static_cast<size_t>(time_mode);
    if (mode >= TIME_MODES_COUNT || id >= vertices.size())
        return 0.0f;
    size_t first = 0;
    float time = 0.0f;
    const size_t layer = static_cast<size_t>(vertices[id].layer_id);
    if (layer < layer_first_vertex.size() && layer_first_vertex[layer] <= id) {
        first = layer_first_vertex[layer];
        time = layer_start_times[mode][layer];
    }
    for (size_t i = first; i <= id; ++i)
        time += vertices[i].times[mode];
    return time;
}
