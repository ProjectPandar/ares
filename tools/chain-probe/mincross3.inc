static inline void do_crossover(const std::vector<FlipEdge> &edges_in, std::vector<FlipEdge> &edges_out,
	const std::pair<size_t, size_t> &span1, const std::pair<size_t, size_t> &span2, const std::pair<size_t, size_t> &span3,
	size_t i)
{
	assert(edges_in.size() == edges_out.size());
	auto do_it = [&edges_in, &edges_out](
		const std::pair<size_t, size_t> &span1, bool reversed1, bool flipped1,
		const std::pair<size_t, size_t> &span2, bool reversed2, bool flipped2,
		const std::pair<size_t, size_t> &span3, bool reversed3, bool flipped3) {
		auto it_edges_out = edges_out.begin();
        auto copy_span = [&edges_in, &it_edges_out](std::pair<size_t, size_t> span, bool reversed, bool flipped) {
			assert(span.first < span.second);
			auto it = it_edges_out;
			if (reversed)
				std::reverse_copy(edges_in.begin() + span.first, edges_in.begin() + span.second, it_edges_out);
			else
				std::copy        (edges_in.begin() + span.first, edges_in.begin() + span.second, it_edges_out);
			it_edges_out += span.second - span.first;
			if (reversed != flipped) {
				for (; it != it_edges_out; ++ it)
					it->flip();
			}
		};
		copy_span(span1, reversed1, flipped1);
		copy_span(span2, reversed2, flipped2);
		copy_span(span3, reversed3, flipped3);
	};
	switch (i >> 6) {
	case 0:
		do_it(span1, (i & 1) != 0, (i & (1 << 1)) != 0, span2, (i & (1 << 2)) != 0, (i & (1 << 3)) != 0, span3, (i & (1 << 4)) != 0, (i & (1 << 5)) != 0);
		break;
	case 1:
		do_it(span1, (i & 1) != 0, (i & (1 << 1)) != 0, span3, (i & (1 << 2)) != 0, (i & (1 << 3)) != 0, span2, (i & (1 << 4)) != 0, (i & (1 << 5)) != 0);
		break;
	default:
		assert((i >> 6) == 2);
		do_it(span2, (i & 1) != 0, (i & (1 << 1)) != 0, span1, (i & (1 << 2)) != 0, (i & (1 << 3)) != 0, span3, (i & (1 << 4)) != 0, (i & (1 << 5)) != 0);
	}
	assert(edges_in.size() == edges_out.size());
}

#if 0
static inline void do_crossover(const std::vector<FlipEdge> &edges_in, std::vector<FlipEdge> &edges_out,
	const std::pair<size_t, size_t> &span1, const std::pair<size_t, size_t> &span2, const std::pair<size_t, size_t> &span3, const std::pair<size_t, size_t> &span4,
	size_t i)
{
	assert(edges_in.size() == edges_out.size());
	auto do_it = [&edges_in, &edges_out](
		const std::pair<size_t, size_t> &span1, bool reversed1, bool flipped1,
		const std::pair<size_t, size_t> &span2, bool reversed2, bool flipped2,
		const std::pair<size_t, size_t> &span3, bool reversed3, bool flipped3,
		const std::pair<size_t, size_t> &span4, bool reversed4, bool flipped4) {
		auto it_edges_out = edges_out.begin();
        auto copy_span = [&edges_in, &it_edges_out](std::pair<size_t, size_t> span, bool reversed, bool flipped) {
			assert(span.first < span.second);
			auto it = it_edges_out;
			if (reversed)
				std::reverse_copy(edges_in.begin() + span.first, edges_in.begin() + span.second, it_edges_out);
			else
				std::copy        (edges_in.begin() + span.first, edges_in.begin() + span.second, it_edges_out);
			it_edges_out += span.second - span.first;
			if (reversed != flipped) {
				for (; it != it_edges_out; ++ it)
					it->flip();
			}
		};
		copy_span(span1, reversed1, flipped1);
		copy_span(span2, reversed2, flipped2);
		copy_span(span3, reversed3, flipped3);
		copy_span(span4, reversed4, flipped4);
	};
	switch (i >> 8) {
	case 0:
		assert(i != 0); // otherwise it would be a no-op
		do_it(span1, (i & 1) != 0, (i & (1 << 1)) != 0, span2, (i & (1 << 2)) != 0, (i & (1 << 3)) != 0, span3, (i & (1 << 4)) != 0, (i & (1 << 5)) != 0, span4, (i & (1 << 6)) != 0, (i & (1 << 7)) != 0);
		break;
	case 1:
		do_it(span1, (i & 1) != 0, (i & (1 << 1)) != 0, span2, (i & (1 << 2)) != 0, (i & (1 << 3)) != 0, span4, (i & (1 << 4)) != 0, (i & (1 << 5)) != 0, span3, (i & (1 << 6)) != 0, (i & (1 << 7)) != 0);
		break;
	case 2:
		do_it(span1, (i & 1) != 0, (i & (1 << 1)) != 0, span3, (i & (1 << 2)) != 0, (i & (1 << 3)) != 0, span2, (i & (1 << 4)) != 0, (i & (1 << 5)) != 0, span4, (i & (1 << 6)) != 0, (i & (1 << 7)) != 0);
		break;
	case 3:
		do_it(span1, (i & 1) != 0, (i & (1 << 1)) != 0, span3, (i & (1 << 2)) != 0, (i & (1 << 3)) != 0, span4, (i & (1 << 4)) != 0, (i & (1 << 5)) != 0, span2, (i & (1 << 6)) != 0, (i & (1 << 7)) != 0);
		break;
	case 4:
		do_it(span1, (i & 1) != 0, (i & (1 << 1)) != 0, span4, (i & (1 << 2)) != 0, (i & (1 << 3)) != 0, span2, (i & (1 << 4)) != 0, (i & (1 << 5)) != 0, span3, (i & (1 << 6)) != 0, (i & (1 << 7)) != 0);
		break;
	case 5:
		do_it(span1, (i & 1) != 0, (i & (1 << 1)) != 0, span4, (i & (1 << 2)) != 0, (i & (1 << 3)) != 0, span3, (i & (1 << 4)) != 0, (i & (1 << 5)) != 0, span2, (i & (1 << 6)) != 0, (i & (1 << 7)) != 0);
		break;
	case 6:
		do_it(span2, (i & 1) != 0, (i & (1 << 1)) != 0, span1, (i & (1 << 2)) != 0, (i & (1 << 3)) != 0, span3, (i & (1 << 4)) != 0, (i & (1 << 5)) != 0, span4, (i & (1 << 6)) != 0, (i & (1 << 7)) != 0);
		break;
	case 7:
		do_it(span2, (i & 1) != 0, (i & (1 << 1)) != 0, span1, (i & (1 << 2)) != 0, (i & (1 << 3)) != 0, span4, (i & (1 << 4)) != 0, (i & (1 << 5)) != 0, span3, (i & (1 << 6)) != 0, (i & (1 << 7)) != 0);
		break;
	case 8:
		do_it(span2, (i & 1) != 0, (i & (1 << 1)) != 0, span3, (i & (1 << 2)) != 0, (i & (1 << 3)) != 0, span1, (i & (1 << 4)) != 0, (i & (1 << 5)) != 0, span4, (i & (1 << 6)) != 0, (i & (1 << 7)) != 0);
		break;
	case 9:
		do_it(span2, (i & 1) != 0, (i & (1 << 1)) != 0, span4, (i & (1 << 2)) != 0, (i & (1 << 3)) != 0, span1, (i & (1 << 4)) != 0, (i & (1 << 5)) != 0, span3, (i & (1 << 6)) != 0, (i & (1 << 7)) != 0);
		break;
	case 10:
		do_it(span3, (i & 1) != 0, (i & (1 << 1)) != 0, span1, (i & (1 << 2)) != 0, (i & (1 << 3)) != 0, span2, (i & (1 << 4)) != 0, (i & (1 << 5)) != 0, span4, (i & (1 << 6)) != 0, (i & (1 << 7)) != 0);
		break;
	default:
		assert((i >> 8) == 11);
		do_it(span3, (i & 1) != 0, (i & (1 << 1)) != 0, span2, (i & (1 << 2)) != 0, (i & (1 << 3)) != 0, span1, (i & (1 << 4)) != 0, (i & (1 << 5)) != 0, span4, (i & (1 << 6)) != 0, (i & (1 << 7)) != 0);
		break;
	}
	assert(edges_in.size() == edges_out.size());
}
#endif

// Worst time complexity:    O(min(n, 100) * (n * log n + n^2)
// Expected time complexity: O(min(n, 100) * (n * log n + k * n)
// where n is the number of edges and k is the number of connection_lengths candidates after the first one
// is found that improves the total cost.
//FIXME there are likley better heuristics to lower the time complexity.
