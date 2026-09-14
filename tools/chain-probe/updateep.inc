template<typename QueueType, typename KDTreeType, typename ChainsType, typename EndPointType>
void update_end_point_in_queue(QueueType &queue, const KDTreeType &kdtree, ChainsType &chains, std::vector<EndPointType> &end_points, EndPointType &end_point, size_t first_point_idx, const EndPointType *first_point)
{
	// Updating an end point or a 2nd from an end point.
	size_t this_idx = end_point.index(end_points);
	// If this segment is not the starting segment, then this end point or the opposite is unconnected.
	assert(first_point_idx == this_idx || first_point_idx == (this_idx ^ 1) || end_point.chain_id == 0 || end_point.opposite(end_points).chain_id == 0);
	end_point.edge_candidate = nullptr;
	if (first_point_idx == this_idx || (end_point.chain_id > 0 && first_point_idx == (this_idx ^ 1)))
	{
		// One may never flip the 1st edge, don't try it again.
		if (! end_point.heap_idx_invalid())
			queue.remove(end_point.heap_idx);
	}
	else
	{
		// Update edge_candidate and distance.
		size_t chain1a    = end_point.chain_id;
		size_t chain1b    = end_points[this_idx ^ 1].chain_id;
		size_t this_chain = chains.equivalent(std::max(chain1a, chain1b));
		// Find the closest point to this end_point, which lies on a different extrusion path (filtered by the filter lambda).
		size_t next_idx = find_closest_point(kdtree, end_point.pos, [&end_points, &chains, this_idx, first_point_idx, first_point, this_chain](size_t idx) {
	    	assert(end_points[this_idx].edge_candidate == nullptr);
	    	// Either this end of the edge or the other end of the edge is not yet connected.
	    	assert((end_points[this_idx    ].chain_id == 0 && end_points[this_idx    ].edge_out == nullptr) ||
	    		   (end_points[this_idx ^ 1].chain_id == 0 && end_points[this_idx ^ 1].edge_out == nullptr));
			if ((idx ^ this_idx) <= 1 || idx == first_point_idx)
				// Points of the same segment shall not be connected.
				// Don't connect to the first point, we must not flip the 1st edge.
				return false;
			size_t chain2a = end_points[idx].chain_id;
			size_t chain2b = end_points[idx ^ 1].chain_id;
			if (chain2a > 0 && chain2b > 0)
				// Only unconnected end point or a point next to an unconnected end point may be connected to.
				// Ideally those would be removed from the KD tree, but the update is difficult.
				return false;
	    	assert(chain2a == 0 || chain2b == 0);
	    	size_t chain2 = chains.equivalent(std::max(chain2a, chain2b));
	    	if (this_chain == chain2)
	    		// Don't connect back to the same chain, don't create a loop.
	    		return this_chain == 0;
	    	// Don't connect to a segment requiring flipping if the segment starts or ends with the first point.
			if (chain2a > 0) {
				// Chain requires flipping.
				assert(chain2b == 0);
				auto &chain = chains.chain(chain2);
				if (chain.begin == first_point || chain.end == first_point)
					return false;
			}
			// Everything is all right, try to connect.
			return true;
		});
		assert(next_idx < end_points.size());
		assert(chains.equivalent(end_points[next_idx].chain_id) != chains.equivalent(end_points[next_idx ^ 1].chain_id) || end_points[next_idx].chain_id == 0);
		end_point.edge_candidate = &end_points[next_idx];
		end_point.distance_out   = (end_points[next_idx].pos - end_point.pos).norm();
		if (end_point.chain_id > 0)
			end_point.distance_out += chains.chain_flip_penalty(this_chain);
		if (end_points[next_idx].chain_id > 0)
			// The candidate chain is flipped.
			end_point.distance_out += chains.chain_flip_penalty(end_points[next_idx].chain_id);
		// Update position of this end point in the queue based on the distance calculated at the line above.
		if (end_point.heap_idx_invalid())
			queue.push(&end_point);
		else
			queue.update(end_point.heap_idx);
	}
}

template<typename PointType, typename SegmentEndPointFunc, bool REVERSE_COULD_FAIL, typename CouldReverseFunc>
