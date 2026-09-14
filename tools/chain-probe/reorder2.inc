static inline void reorder_by_two_exchanges_with_segment_flipping(std::vector<FlipEdge> &edges)
{
	if (edges.size() < 2)
		return;

	std::vector<ConnectionCost> 			connections(edges.size());
	std::vector<FlipEdge> 					edges_tmp(edges);
	std::vector<std::pair<double, size_t>>	connection_lengths(edges.size() - 1, std::pair<double, size_t>(0., 0));
	std::vector<char>						connection_tried(edges.size(), false);
	const size_t 							max_iterations = std::min(edges.size(), size_t(100));
	for (size_t iter = 0; iter < max_iterations; ++ iter) {
		// Initialize connection costs and connection lengths.
		for (size_t i = 1; i < edges.size(); ++ i) {
			const FlipEdge   	 &e1 = edges[i - 1];
			const FlipEdge   	 &e2 = edges[i];
			ConnectionCost	     &c  = connections[i];
			c = connections[i - 1];
			double l = (e2.p1 - e1.p2).norm();
			c.cost += l;
			c.cost_flipped += (e2.p2 - e1.p1).norm();
			connection_lengths[i - 1] = std::make_pair(l, i);
		}
		std::sort(connection_lengths.begin(), connection_lengths.end(), [](const std::pair<double, size_t> &l, const std::pair<double, size_t> &r) { return l.first > r.first; });
		std::fill(connection_tried.begin(), connection_tried.end(), false);
		size_t crossover1_pos_final = std::numeric_limits<size_t>::max();
		size_t crossover2_pos_final = std::numeric_limits<size_t>::max();
		size_t crossover_flip_final = 0;
        for (const std::pair<double, size_t>& first_crossover_candidate : connection_lengths) {
            size_t longest_connection_idx = first_crossover_candidate.second;
			connection_tried[longest_connection_idx] = true;
			// Find the second crossover connection with the lowest total chain cost.
			size_t crossover_pos_min  = std::numeric_limits<size_t>::max();
			double crossover_cost_min = connections.back().cost;
			size_t crossover_flip_min = 0;
			for (size_t j = 1; j < connections.size(); ++ j)
				if (! connection_tried[j]) {
					size_t a = j;
					size_t b = longest_connection_idx;
					if (a > b)
						std::swap(a, b);
					std::pair<double, size_t> cost_and_flip = minimum_crossover_cost(edges, 
						std::make_pair(size_t(0), a), connections[a - 1], std::make_pair(a, b), connections[b - 1] - connections[a], std::make_pair(b, edges.size()), connections.back() - connections[b],
						connections.back().cost);
					if (cost_and_flip.second > 0 && cost_and_flip.first < crossover_cost_min) {
						crossover_pos_min  = j;
						crossover_cost_min = cost_and_flip.first;
						crossover_flip_min = cost_and_flip.second;
						assert(crossover_cost_min < connections.back().cost + EPSILON);
					}
				}
			if (crossover_cost_min < connections.back().cost) {
				// The cost of the chain with the proposed two crossovers has a lower total cost than the current chain. Apply the crossover.
				crossover1_pos_final = longest_connection_idx;
				crossover2_pos_final = crossover_pos_min;
				crossover_flip_final = crossover_flip_min;
				break;
			} else {
				// Continue with another long candidate edge.
			}
		}
		if (crossover_flip_final > 0) {
			// Pair of cross over positions and flip / reverse constellation has been found, which improves the total cost of the connection.
			// Perform a crossover.
			if (crossover1_pos_final > crossover2_pos_final)
				std::swap(crossover1_pos_final, crossover2_pos_final);
			do_crossover(edges, edges_tmp, std::make_pair(size_t(0), crossover1_pos_final), std::make_pair(crossover1_pos_final, crossover2_pos_final), std::make_pair(crossover2_pos_final, edges.size()), crossover_flip_final);
			edges.swap(edges_tmp);
		} else {
			// No valid pair of cross over positions was found improving the total cost. Giving up.
			break;
		}
	}
}

