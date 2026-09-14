static inline void improve_ordering_by_two_exchanges_with_segment_flipping(Polylines &polylines, bool fixed_start)
{
#ifndef NDEBUG
	auto cost = [&polylines]() {
		double sum = 0.;
		for (size_t i = 1; i < polylines.size(); ++i)
			sum += (polylines[i].first_point() - polylines[i - 1].last_point()).norm();
		return sum;
	};
	double cost_initial = cost();

	static int iRun = 0;
	++ iRun;
#ifdef DEBUG_SVG_OUTPUT
	svg_draw_polyline_chain("improve_ordering_by_two_exchanges_with_segment_flipping-initial", iRun, polylines);
#endif /* DEBUG_SVG_OUTPUT */
#endif /* NDEBUG */

	std::vector<FlipEdge> edges;
	edges.reserve(polylines.size());
    std::transform(polylines.begin(), polylines.end(), std::back_inserter(edges), 
    	[&polylines](const Polyline &pl){ return FlipEdge(pl.first_point(), pl.last_point(), &pl - polylines.data()); });
#if 1
	reorder_by_two_exchanges_with_segment_flipping(edges);
#else
	// reorder_by_three_exchanges_with_segment_flipping(edges);
	reorder_by_three_exchanges_with_segment_flipping2(edges);
#endif
	Polylines out;
	out.reserve(polylines.size());
	for (const FlipEdge &edge : edges) {
		Polyline &pl = polylines[edge.source_index];
		out.emplace_back(std::move(pl));
		if (edge.p2 == out.back().first_point()) {
			// Polyline is flipped.
			out.back().reverse();
		} else {
			// Polyline is not flipped.
			assert(edge.p1 == out.back().first_point());
		}
	}
	polylines = out;

#ifndef NDEBUG
	double cost_final = cost();
#ifdef DEBUG_SVG_OUTPUT
	svg_draw_polyline_chain("improve_ordering_by_two_exchanges_with_segment_flipping-final", iRun, out);
#endif /* DEBUG_SVG_OUTPUT */
	assert(cost_final <= cost_initial);
#endif /* NDEBUG */
}

// Used to optimize order of infill lines and brim lines.
