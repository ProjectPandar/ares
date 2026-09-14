Polylines chain_polylines(Polylines &&polylines, const Point *start_near)
{
#ifdef DEBUG_SVG_OUTPUT
	static int iRun = 0;
	++ iRun;
	svg_draw_polyline_chain("chain_polylines-initial", iRun, polylines);
#endif /* DEBUG_SVG_OUTPUT */

	Polylines out;
	if (! polylines.empty()) {
		auto segment_end_point = [&polylines](size_t idx, bool first_point) -> const Point& { return first_point ? polylines[idx].first_point() : polylines[idx].last_point(); };
		std::vector<std::pair<size_t, bool>> ordered = chain_segments_greedy2<Point, decltype(segment_end_point)>(segment_end_point, polylines.size(), start_near);
		out.reserve(polylines.size()); 
		for (auto &segment_and_reversal : ordered) {
			out.emplace_back(std::move(polylines[segment_and_reversal.first]));
			if (segment_and_reversal.second)
				out.back().reverse();
		}
		if (out.size() > 1 && start_near == nullptr) {
			improve_ordering_by_two_exchanges_with_segment_flipping(out, start_near != nullptr);
			//improve_ordering_by_segment_flipping(out, start_near != nullptr);
		}
	}

#ifdef DEBUG_SVG_OUTPUT
	svg_draw_polyline_chain("chain_polylines-final", iRun, out);
#endif /* DEBUG_SVG_OUTPUT */
	return out;
}

