static inline std::pair<double, size_t> minimum_crossover_cost(
	const std::vector<FlipEdge>		  &edges,
	const std::pair<size_t, size_t>   &span1, const ConnectionCost &cost1,
	const std::pair<size_t, size_t>   &span2, const ConnectionCost &cost2,
	const std::pair<size_t, size_t>   &span3, const ConnectionCost &cost3,
	const std::pair<size_t, size_t>   &span4, const ConnectionCost &cost4,
	const double					   cost_current)
{
	auto connection_cost = [&edges](
		const std::pair<size_t, size_t> &span1, const ConnectionCost &cost1, bool reversed1, bool flipped1,
		const std::pair<size_t, size_t> &span2, const ConnectionCost &cost2, bool reversed2, bool flipped2,
		const std::pair<size_t, size_t> &span3, const ConnectionCost &cost3, bool reversed3, bool flipped3,
		const std::pair<size_t, size_t> &span4, const ConnectionCost &cost4, bool reversed4, bool flipped4) {
		auto first_point = [&edges](const std::pair<size_t, size_t> &span, bool flipped) { return flipped ? edges[span.first].p2 : edges[span.first].p1; };
		auto last_point  = [&edges](const std::pair<size_t, size_t> &span, bool flipped) { return flipped ? edges[span.second - 1].p1 : edges[span.second - 1].p2; };
		auto point       = [first_point, last_point](const std::pair<size_t, size_t> &span, bool start, bool flipped) { return start ? first_point(span, flipped) : last_point(span, flipped); };
		auto cost        = [](const ConnectionCost &acost, bool flipped) { 
			assert(acost.cost >= 0. && acost.cost_flipped >= 0.);
			return flipped ? acost.cost_flipped : acost.cost;
		};
		// Ignore reversed single segment spans.
		auto simple_span_ignore = [](const std::pair<size_t, size_t>& span, bool reversed) {
			return span.first + 1 == span.second && reversed;
		};
		assert(span1.first < span1.second);
		assert(span2.first < span2.second);
		assert(span3.first < span3.second);
		assert(span4.first < span4.second);
		return 
			simple_span_ignore(span1, reversed1) || simple_span_ignore(span2, reversed2) || simple_span_ignore(span3, reversed3) || simple_span_ignore(span4, reversed4) ?
				// Don't perform unnecessary calculations simulating reversion of single segment spans.
				std::numeric_limits<double>::max() :
				// Calculate the cost of reverting chains and / or flipping segment orientations.
				cost(cost1, flipped1) + cost(cost2, flipped2) + cost(cost3, flipped3) + cost(cost4, flipped4) +
					(point(span2, ! reversed2, flipped2) - point(span1, reversed1, flipped1)).norm() + 
					(point(span3, ! reversed3, flipped3) - point(span2, reversed2, flipped2)).norm() +
					(point(span4, ! reversed4, flipped4) - point(span3, reversed3, flipped3)).norm();
	};

#ifndef NDEBUG
	{
		double c = connection_cost(span1, cost1, false, false, span2, cost2, false, false, span3, cost3, false, false, span4, cost4, false, false);
		assert(std::abs(c - cost_current) < SCALED_EPSILON);
	}
#endif /* NDEBUG */

	double cost_min = cost_current;
	size_t flip_min = 0; // no flip, no improvement
	for (size_t i = 0; i < (1 << 8); ++ i) {
		// From the three combinations of 1,2,3 ordering, the other three are reversals of the first three.
		size_t permutation = 0;
		for (double c : {
				(i == 0) ? cost_current : 
				connection_cost(span1, cost1, (i & 1) != 0, (i & (1 << 1)) != 0, span2, cost2, (i & (1 << 2)) != 0, (i & (1 << 3)) != 0, span3, cost3, (i & (1 << 4)) != 0, (i & (1 << 5)) != 0, span4, cost4, (i & (1 << 6)) != 0, (i & (1 << 7)) != 0),
				connection_cost(span1, cost1, (i & 1) != 0, (i & (1 << 1)) != 0, span2, cost2, (i & (1 << 2)) != 0, (i & (1 << 3)) != 0, span4, cost4, (i & (1 << 4)) != 0, (i & (1 << 5)) != 0, span3, cost3, (i & (1 << 6)) != 0, (i & (1 << 7)) != 0),
				connection_cost(span1, cost1, (i & 1) != 0, (i & (1 << 1)) != 0, span3, cost3, (i & (1 << 2)) != 0, (i & (1 << 3)) != 0, span2, cost2, (i & (1 << 4)) != 0, (i & (1 << 5)) != 0, span4, cost4, (i & (1 << 6)) != 0, (i & (1 << 7)) != 0),
				connection_cost(span1, cost1, (i & 1) != 0, (i & (1 << 1)) != 0, span3, cost3, (i & (1 << 2)) != 0, (i & (1 << 3)) != 0, span4, cost4, (i & (1 << 4)) != 0, (i & (1 << 5)) != 0, span2, cost2, (i & (1 << 6)) != 0, (i & (1 << 7)) != 0),
				connection_cost(span1, cost1, (i & 1) != 0, (i & (1 << 1)) != 0, span4, cost4, (i & (1 << 2)) != 0, (i & (1 << 3)) != 0, span2, cost2, (i & (1 << 4)) != 0, (i & (1 << 5)) != 0, span3, cost3, (i & (1 << 6)) != 0, (i & (1 << 7)) != 0),
				connection_cost(span1, cost1, (i & 1) != 0, (i & (1 << 1)) != 0, span4, cost4, (i & (1 << 2)) != 0, (i & (1 << 3)) != 0, span3, cost3, (i & (1 << 4)) != 0, (i & (1 << 5)) != 0, span2, cost2, (i & (1 << 6)) != 0, (i & (1 << 7)) != 0),
				connection_cost(span2, cost2, (i & 1) != 0, (i & (1 << 1)) != 0, span1, cost1, (i & (1 << 2)) != 0, (i & (1 << 3)) != 0, span3, cost3, (i & (1 << 4)) != 0, (i & (1 << 5)) != 0, span4, cost4, (i & (1 << 6)) != 0, (i & (1 << 7)) != 0),
				connection_cost(span2, cost2, (i & 1) != 0, (i & (1 << 1)) != 0, span1, cost1, (i & (1 << 2)) != 0, (i & (1 << 3)) != 0, span4, cost4, (i & (1 << 4)) != 0, (i & (1 << 5)) != 0, span3, cost3, (i & (1 << 6)) != 0, (i & (1 << 7)) != 0),
				connection_cost(span2, cost2, (i & 1) != 0, (i & (1 << 1)) != 0, span3, cost3, (i & (1 << 2)) != 0, (i & (1 << 3)) != 0, span1, cost1, (i & (1 << 4)) != 0, (i & (1 << 5)) != 0, span4, cost4, (i & (1 << 6)) != 0, (i & (1 << 7)) != 0),
				connection_cost(span2, cost2, (i & 1) != 0, (i & (1 << 1)) != 0, span4, cost4, (i & (1 << 2)) != 0, (i & (1 << 3)) != 0, span1, cost1, (i & (1 << 4)) != 0, (i & (1 << 5)) != 0, span3, cost3, (i & (1 << 6)) != 0, (i & (1 << 7)) != 0),
				connection_cost(span3, cost3, (i & 1) != 0, (i & (1 << 1)) != 0, span1, cost1, (i & (1 << 2)) != 0, (i & (1 << 3)) != 0, span2, cost2, (i & (1 << 4)) != 0, (i & (1 << 5)) != 0, span4, cost4, (i & (1 << 6)) != 0, (i & (1 << 7)) != 0),
				connection_cost(span3, cost3, (i & 1) != 0, (i & (1 << 1)) != 0, span2, cost2, (i & (1 << 2)) != 0, (i & (1 << 3)) != 0, span1, cost1, (i & (1 << 4)) != 0, (i & (1 << 5)) != 0, span4, cost4, (i & (1 << 6)) != 0, (i & (1 << 7)) != 0)
			}) {
			if (c < cost_min) {
				cost_min = c;
				flip_min = i + (permutation << 8);
			}
			++ permutation;
		}
	}
	return std::make_pair(cost_min, flip_min);
}
