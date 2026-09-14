template<typename PointType, typename SegmentEndPointFunc, bool REVERSE_COULD_FAIL, typename CouldReverseFunc>
std::vector<std::pair<size_t, bool>> chain_segments_greedy_constrained_reversals2_(SegmentEndPointFunc end_point_func, CouldReverseFunc could_reverse_func, size_t num_segments, const PointType *start_near)
{
	std::vector<std::pair<size_t, bool>> out;

	if (num_segments == 0) {
		// Nothing to do.
	} 
	else if (num_segments == 1)
	{
		// Just sort the end points so that the first point visited is closest to start_near.
		out.emplace_back(0, start_near != nullptr && 
            (end_point_func(0, true) - *start_near).template cast<double>().squaredNorm() < (end_point_func(0, false) - *start_near).template cast<double>().squaredNorm());
	} 
	else
	{
		// End points of segments for the KD tree closest point search.
		// A single end point is inserted into the search structure for loops, two end points are entered for open paths.
		struct EndPoint {
			EndPoint(const Vec2d &pos) : pos(pos) {}
			Vec2d     pos;

			// Candidate for a new connection link.
			EndPoint *edge_candidate = nullptr;
			// Distance to the next end point following the link.
			// Zero value -> start of the final path.
			double    distance_out = std::numeric_limits<double>::max();

			size_t    heap_idx = std::numeric_limits<size_t>::max();
			bool 	  heap_idx_invalid() const { return this->heap_idx == std::numeric_limits<size_t>::max(); }

			// Identifier of the chain, to which this end point belongs. Zero means unassigned.
			size_t    chain_id = 0;
			// Double linked chain of segment end points in current path.
			EndPoint *edge_out = nullptr;

			size_t 				index(std::vector<EndPoint>          &endpoints) const { return this - endpoints.data(); }
			// Opposite end point of the same segment.
			EndPoint& 			opposite(std::vector<EndPoint>       &endpoints)       { return endpoints[(this - endpoints.data()) ^ 1]; }
			const EndPoint& 	opposite(const std::vector<EndPoint> &endpoints) const { return endpoints[(this - endpoints.data()) ^ 1]; }
		};

	    std::vector<EndPoint> end_points;
	    end_points.reserve(num_segments * 2);
	    for (size_t i = 0; i < num_segments; ++ i) {
            end_points.emplace_back(end_point_func(i, true ).template cast<double>());
            end_points.emplace_back(end_point_func(i, false).template cast<double>());
	    }

	    // Construct the closest point KD tree over end points of segments.
		auto coordinate_fn = [&end_points](size_t idx, size_t dimension) -> double { return end_points[idx].pos[dimension]; };
		KDTreeIndirect<2, double, decltype(coordinate_fn)> kdtree(coordinate_fn, end_points.size());

	    // Chained segments with their sum of connection lengths.
	    // The chain supports flipping all the segments, connecting the segments at the opposite ends.
	    // (this is a very useful path optimization for infill lines).
	    struct Chain {
	    	size_t		 num_segments	 = 0;
	    	double 		 cost 			 = 0.;
	    	double 		 cost_flipped	 = 0.;
	    	EndPoint 	*begin 			 = nullptr;
	    	EndPoint 	*end 			 = nullptr;
	    	size_t  	 equivalent_with = 0;

	    	// Flipping the chain has a time complexity of O(n).
	    	void flip(std::vector<EndPoint> &endpoints)
	    	{
				assert(this->num_segments > 1);
				assert(this->begin->edge_out == nullptr);
	    		assert(this->end  ->edge_out == nullptr);
				assert(this->begin->opposite(endpoints).edge_out != nullptr);
				assert(this->end  ->opposite(endpoints).edge_out != nullptr);
				// Start of the current segment processed.
	    		EndPoint *ept      = this->begin;
	    		// Previous end point to connect the other side of ept to.
	    		EndPoint *ept_prev = nullptr;
	    		do {
	    			EndPoint *ept_end      = &ept->opposite(endpoints);
	    			EndPoint *ept_next     = ept_end->edge_out;
	    			assert(ept_next == nullptr || ept_next->edge_out == ept_end);
	    			// Connect to the preceding segment.
	    			ept_end->edge_out = ept_prev;
	    			if (ept_prev != nullptr)
	    				ept_prev->edge_out = ept_end;
	    			ept_prev = ept;
	    			ept = ept_next;
	    		} while (ept != nullptr);
	    		ept_prev->edge_out = nullptr;
	    		// Swap the costs.
	    		std::swap(this->cost, this->cost_flipped);
				// Swap the ends.
				EndPoint *new_begin = &this->begin->opposite(endpoints);
				EndPoint *new_end   = &this->end->opposite(endpoints);
				std::swap(this->begin->chain_id, new_begin->chain_id);
				std::swap(this->end  ->chain_id, new_end  ->chain_id);
				this->begin = new_begin;
				this->end   = new_end;
				assert(this->begin->edge_out == nullptr);
	    		assert(this->end  ->edge_out == nullptr);
				assert(this->begin->opposite(endpoints).edge_out != nullptr);
				assert(this->end  ->opposite(endpoints).edge_out != nullptr);
			}

	    	double flip_penalty() const { return this->cost_flipped - this->cost; }
	    };

		// Helper to detect loops in already connected paths and to accomodate flipping of chains.
		//	
		// Unique chain IDs are assigned to paths. If paths are connected, end points will not have their chain IDs updated, but the chain IDs
		// will remember an "equivalent" chain ID, which is the lowest ID of all the IDs in the path, and the lowest ID is equivalent to itself.
		// Chain IDs are indexed starting with 1.
		// 
		// Chains remember their lengths and their lengths when each segment of the chain is flipped.
		class Chains {
		public:
			// Zero'th chain ID is invalid.
			Chains(size_t reserve) { 
				m_chains.reserve(reserve / 2);
				// Indexing starts with 1.
				m_chains.emplace_back();
			}

			// Generate next equivalence class.
			size_t 				next_id() {
				m_chains.emplace_back();
				m_chains.back().equivalent_with = ++ m_last_chain_id;
				return m_last_chain_id;
			}

			// Get equivalence class for chain ID, update the "equivalent_with" along the equivalence path.
			size_t 				equivalent(size_t chain_id) {
				if (chain_id != 0) {
					for (size_t last = chain_id;;) {
						size_t lower = m_chains[last].equivalent_with;
						if (lower == last) {
							m_chains[chain_id].equivalent_with = lower;
							chain_id = lower;
							break;
						}
						last = lower;
					}
				}
				return chain_id;
			}

			// Return a lowest chain ID of the two input chains.
			// Produce a new chain ID of both chain IDs are zero.
			size_t 			  	merge(size_t chain_id1, size_t chain_id2) {
				if (chain_id1 == 0)
					return (chain_id2 == 0) ? this->next_id() : chain_id2;
				if (chain_id2 == 0)
					return chain_id1;
				assert(m_chains[chain_id1].equivalent_with == chain_id1);
				assert(m_chains[chain_id2].equivalent_with == chain_id2);
				size_t chain_id = std::min(chain_id1, chain_id2);
				m_chains[chain_id1].equivalent_with = chain_id;
				m_chains[chain_id2].equivalent_with = chain_id;
				return chain_id;
			}

			Chain& 				chain(size_t chain_id) 		 { return m_chains[chain_id]; }
			const Chain&		chain(size_t chain_id) const { return m_chains[chain_id]; }

			double 				chain_flip_penalty(size_t chain_id) {
				chain_id = this->equivalent(chain_id);
				return m_chains[chain_id].flip_penalty();
			}

#ifndef NDEBUG
			bool				validate()
			{
				// Validate that the segments merged chain IDs make up a directed acyclic graph
				// with edges oriented towards the lower chain ID, therefore all ending up
				// in the lowest chain ID of all of them.
				assert(m_last_chain_id >= 0);
				assert(m_last_chain_id + 1 == m_chains.size());
				for (size_t i = 0; i < m_chains.size(); ++ i) {
					for (size_t last = i;;) {
						size_t lower = m_chains[last].equivalent_with;
						assert(lower <= last);
						if (lower == last)
							break;
						last = lower;
					}
				}
				return true;
			}
#endif /* NDEBUG */

		private:
			std::vector<Chain> 	m_chains;
			// Unique chain ID assigned to chains of end points of segments.
			size_t              m_last_chain_id = 0;
		} chains(num_segments);

		// Find the first end point closest to start_near.
		EndPoint *first_point = nullptr;
		size_t    first_point_idx = std::numeric_limits<size_t>::max();
		if (start_near != nullptr) {
            size_t idx = find_closest_point(kdtree, start_near->template cast<double>());
			assert(idx < end_points.size());
			first_point = &end_points[idx];
			first_point->distance_out = 0.;
			first_point->chain_id = chains.next_id();
			Chain &chain = chains.chain(first_point->chain_id);
			chain.begin = first_point;
			chain.end   = &first_point->opposite(end_points);
			first_point_idx = idx;
		}
		EndPoint *initial_point = first_point;
		EndPoint *last_point = nullptr;

		// Assign the closest point and distance to the end points.
		for (EndPoint &end_point : end_points) {
	    	assert(end_point.edge_candidate == nullptr);
	    	if (&end_point != first_point) {
		    	size_t this_idx = end_point.index(end_points);
		    	// Find the closest point to this end_point, which lies on a different extrusion path (filtered by the lambda).
		    	// Ignore the starting point as the starting point is considered to be occupied, no end point coud connect to it.
				size_t next_idx = find_closest_point(kdtree, end_point.pos, 
					[this_idx, first_point_idx](size_t idx){ return idx != first_point_idx && (idx ^ this_idx) > 1; });
				assert(next_idx < end_points.size());
				EndPoint &end_point2 = end_points[next_idx];
				end_point.edge_candidate = &end_point2;
				end_point.distance_out = (end_point2.pos - end_point.pos).norm();
			}
		}

	    // Initialize a heap of end points sorted by the lowest distance to the next valid point of a path.
	    auto queue = make_mutable_priority_queue<EndPoint*, true>(
			[](EndPoint *ep, size_t idx){ ep->heap_idx = idx; }, 
	    	[](EndPoint *l, EndPoint *r){ return l->distance_out < r->distance_out; });
		queue.reserve(end_points.size() * 2);
	    for (EndPoint &ep : end_points)
	    	if (first_point != &ep)
				queue.push(&ep);

#ifndef NDEBUG
		auto validate_graph_and_queue = [&chains, &end_points, &queue, first_point]() -> bool {
			assert(chains.validate());
			for (EndPoint &ep : end_points) {
				if (ep.heap_idx < queue.size()) {
					// End point is on the heap.
					assert(*(queue.cbegin() + ep.heap_idx) == &ep);
					// One side or the other of the segment is not yet connected.
					assert(ep.chain_id == 0 || ep.opposite(end_points).chain_id == 0);
				} else {
					// End point is NOT on the heap, therefore it must part of the output path.
					assert(ep.heap_idx_invalid());
					assert(ep.chain_id != 0);
					if (&ep == first_point) {
						assert(ep.edge_out == nullptr);
					} else {
						assert(ep.edge_out != nullptr);
						// Detect loops.
						for (EndPoint *pt = &ep; pt != nullptr;) {
							// Out of queue. It is a final point.
							EndPoint *pt_other = &pt->opposite(end_points);
							if (pt_other->heap_idx < queue.size()) {
								// The other side of this segment is undecided yet.
								// assert(pt_other->edge_out == nullptr);
								break;
							}
							pt = pt_other->edge_out;
						}
					}
				}
			}
			for (EndPoint *ep : queue)
				// Points in the queue or the opposites of the same segment are not connected yet.
				assert(ep->chain_id == 0 || ep->opposite(end_points).chain_id == 0);
			return true;
		};
#endif /* NDEBUG */

	    // Chain the end points: find (num_segments - 1) shortest links not forming bifurcations or loops.
		assert(num_segments >= 2);
#ifndef NDEBUG
		double distance_taken_last = 0.;
#endif /* NDEBUG */
		// Some links stored onto the priority queue are being invalidated during the calculation and they are not
		// updated immediately. If such a situation is detected for an end point pulled from the priority queue,
		// the end point is being updated and re-inserted into the priority queue. Therefore the number of iterations
		// required is higher than expected (it would be the number of links, num_segments - 1).
		// The limit here may not be necessary, but it guards us against an endless loop if something goes wrong.
		size_t num_iter = num_segments * 16;
		for (size_t num_connections_to_end = num_segments - 1; num_iter > 0; -- num_iter) {
			
	    	// Take the first end point, for which the link points to the currently closest valid neighbor.
	    	EndPoint *end_point1       = queue.top();
	    	assert(end_point1 != first_point);
	    	EndPoint *end_point1_other = &end_point1->opposite(end_points);
	    	// true if end_point1 is not the end of its chain, but the 2nd point. When connecting to the 2nd point, this chain needs
	    	// to be flipped first.
	    	bool      chain1_flip      = end_point1->chain_id > 0;
	    	// Either this point at the queue is not connected, or it is the 2nd point of a chain.
	    	// If connecting to a 2nd point of a chain, the 1st point shall not yet be connected and this chain will need
	    	// to be flipped.
	    	assert(  chain1_flip || (end_point1->chain_id == 0 && end_point1->edge_out == nullptr));
	    	assert(! chain1_flip || (end_point1_other->chain_id == 0 && end_point1_other->edge_out == nullptr));
			assert(end_point1->edge_candidate != nullptr);
#ifndef NDEBUG 
			// Each edge added shall be longer than the previous one taken.
			//assert(end_point1->distance_out > distance_taken_last - SCALED_EPSILON);
			if (end_point1->distance_out < distance_taken_last - SCALED_EPSILON) {
//				printf("Warning: taking shorter length than previously is suspicious\n");
			}
			distance_taken_last = end_point1->distance_out;
#endif /* NDEBUG */
	    	// Take the closest end point to the first end point,
	    	EndPoint *end_point2 	   = end_point1->edge_candidate;
	    	EndPoint *end_point2_other = &end_point2->opposite(end_points);
	    	bool      chain2_flip      = end_point2->chain_id > 0;
	    	// Is the link from end_point1 to end_point2 still valid? If yes, the link may be taken. Otherwise the link needs to be refreshed.
	    	bool      valid            = true;
	    	size_t 	  end_point1_chain_id = 0;
	    	size_t    end_point2_chain_id = 0;
	    	if (end_point2->chain_id > 0 && end_point2_other->chain_id > 0) {
	    		// The other side is part of the output path. Don't connect to end_point2, update end_point1 and try another one.
	    		valid = false;
	    	} else {
				// End points of the opposite ends of the segments.
				end_point1_chain_id = chains.equivalent((chain1_flip ? end_point1 : end_point1_other)->chain_id);
				end_point2_chain_id = chains.equivalent((chain2_flip ? end_point2 : end_point2_other)->chain_id);
				if (end_point1_chain_id == end_point2_chain_id && end_point1_chain_id != 0)
					// This edge forms a loop. Update end_point1 and try another one.
					valid = false;
				else {
					// Verify whether end_point1.distance_out still matches the current state of the two end points to be connected and their chains.
					// Namely, the other chain may have been flipped in the meantime.
					double dist = (end_point2->pos - end_point1->pos).norm();
					if (chain1_flip)
						dist += chains.chain_flip_penalty(end_point1_chain_id);
					if (chain2_flip)
						dist += chains.chain_flip_penalty(end_point2_chain_id);
					if (std::abs(dist - end_point1->distance_out) > SCALED_EPSILON)
						// The distance changed due to flipping of one of the chains. Refresh this end point in the queue.
						valid = false;
				}
				if (valid && first_point != nullptr) {
					// Verify that a chain starting or ending with the first_point does not get flipped.
					if (chain1_flip) {
						Chain &chain = chains.chain(end_point1_chain_id);
						if (chain.begin == first_point || chain.end == first_point)
							valid = false;
					}
					if (valid && chain2_flip) {
						Chain &chain = chains.chain(end_point2_chain_id);
						if (chain.begin == first_point || chain.end == first_point)
							valid = false;
					}
				}
	    	}
	    	if (valid) {
		    	// Remove the first and second point from the queue.
				queue.pop();
		    	queue.remove(end_point2->heap_idx);
		    	assert(end_point1->edge_candidate == end_point2);
		    	end_point1->edge_candidate = nullptr;
		    	Chain *chain1 = (end_point1_chain_id == 0) ? nullptr : &chains.chain(end_point1_chain_id);
		    	Chain *chain2 = (end_point2_chain_id == 0) ? nullptr : &chains.chain(end_point2_chain_id);
				assert(chain1 == nullptr || (chain1_flip ? (chain1->begin == end_point1_other || chain1->end == end_point1_other) : (chain1->begin == end_point1 || chain1->end == end_point1)));
				assert(chain2 == nullptr || (chain2_flip ? (chain2->begin == end_point2_other || chain2->end == end_point2_other) : (chain2->begin == end_point2 || chain2->end == end_point2)));
				if (chain1_flip)
		    		chain1->flip(end_points);
		    	if (chain2_flip)
		    		chain2->flip(end_points);
				assert(chain1 == nullptr || chain1->begin == end_point1 || chain1->end == end_point1);
				assert(chain2 == nullptr || chain2->begin == end_point2 || chain2->end == end_point2);
		    	size_t chain_id = chains.merge(end_point1_chain_id, end_point2_chain_id);
				Chain &chain    = chains.chain(chain_id);
				{
				    Chain chain_dst;
					chain_dst.begin = (chain1 == nullptr) ? end_point1_other : (chain1->begin == end_point1) ? chain1->end : chain1->begin;
			    	chain_dst.end   = (chain2 == nullptr) ? end_point2_other : (chain2->begin == end_point2) ? chain2->end : chain2->begin;
			    	chain_dst.cost  = (chain1 == 0 ? 0. : chain1->cost) + (chain2 == 0 ? 0. : chain2->cost) + (end_point2->pos - end_point1->pos).norm();
			    	chain_dst.cost_flipped = (chain1 == 0 ? 0. : chain1->cost_flipped) + (chain2 == 0 ? 0. : chain2->cost_flipped) + (end_point2_other->pos - end_point1_other->pos).norm();
			    	chain_dst.num_segments = (chain1 == 0 ? 1 : chain1->num_segments) + (chain2 == 0 ? 1 : chain2->num_segments);
				    chain_dst.equivalent_with = chain_id;
					chain = chain_dst;
			    }
				if (chain.begin != end_point1_other && ! end_point1_other->heap_idx_invalid())
					queue.remove(end_point1_other->heap_idx);
				if (chain.end != end_point2_other && ! end_point2_other->heap_idx_invalid())
					queue.remove(end_point2_other->heap_idx);
				end_point1->edge_out = end_point2;
				end_point2->edge_out = end_point1;
				end_point1->chain_id = chain_id;
				end_point2->chain_id = chain_id;
				end_point1_other->chain_id = chain_id;
				end_point2_other->chain_id = chain_id;
				if (chain.begin != first_point)
					chain.begin->chain_id = 0;
				if (chain.end != first_point)
					chain.end->chain_id = 0;
				if (-- num_connections_to_end == 0) {
					
					// Last iteration. There shall be exactly one or two end points waiting to be connected.
					assert(queue.size() <= ((first_point == nullptr) ? 4 : 2));
					if (first_point == nullptr) {
						// Find the first remaining end point.
						do {
							first_point = queue.top();
							queue.pop();
						} while (first_point->edge_out != nullptr);
						assert(first_point->edge_out == nullptr);
					}
					// Find the first remaining end point.
					do {
						last_point = queue.top();
						queue.pop();
					} while (last_point->edge_out != nullptr);
					assert(last_point->edge_out == nullptr);
#ifndef NDEBUG
					while (! queue.empty()) {
						assert(queue.top()->edge_out != nullptr && queue.top()->chain_id > 0);
						queue.pop();
					}
#endif /* NDEBUG */
					break;
				} else {
					//FIXME update the 2nd end points on the queue.
					// Update end points of the flipped segments.
					update_end_point_in_queue(queue, kdtree, chains, end_points, chain.begin->opposite(end_points), first_point_idx, first_point);
					update_end_point_in_queue(queue, kdtree, chains, end_points, chain.end->opposite(end_points),   first_point_idx, first_point);
					if (chain1_flip)
						update_end_point_in_queue(queue, kdtree, chains, end_points, *chain.begin, first_point_idx, first_point);
					if (chain2_flip)
						update_end_point_in_queue(queue, kdtree, chains, end_points, *chain.end,   first_point_idx, first_point);
					// End points of chains shall certainly stay in the queue.
					assert(chain.begin == first_point || chain.begin->heap_idx < queue.size());
					assert(chain.end   == first_point || chain.end  ->heap_idx < queue.size());
					assert(&chain.begin->opposite(end_points) != first_point &&
						(chain.begin == first_point ? chain.begin->opposite(end_points).heap_idx_invalid() : chain.begin->opposite(end_points).heap_idx < queue.size()));
					assert(&chain.end  ->opposite(end_points) != first_point &&
						(chain.end   == first_point ? chain.end  ->opposite(end_points).heap_idx_invalid() : chain.end  ->opposite(end_points).heap_idx < queue.size()));

				}
			} else {
				// This edge forms a loop. Update end_point1 and try another one.
				update_end_point_in_queue(queue, kdtree, chains, end_points, *end_point1, first_point_idx, first_point);
#ifndef NDEBUG
				// Each edge shall be longer than the last one removed from the queue.
				//assert(end_point1->distance_out > distance_taken_last - SCALED_EPSILON);
				if (end_point1->distance_out < distance_taken_last - SCALED_EPSILON) {
//					printf("Warning: taking shorter length than previously is suspicious\n");
				}
#endif /* NDEBUG */
		    	//FIXME Remove the other end point from the KD tree.
		    	// As the KD tree update is expensive, do it only after some larger number of points is removed from the queue.
		    }
			
		}
		assert(queue.empty());

		// Now interconnect pairs of segments into a chain.
		assert(first_point != nullptr);
		out.reserve(num_segments);
		bool      failed = false;
		do {
			assert(out.size() < num_segments);
			size_t    		 first_point_id = first_point - &end_points.front();
			size_t           segment_id 	= first_point_id >> 1;
			bool             reverse        = (first_point_id & 1) != 0;
			EndPoint 		*second_point   = &end_points[first_point_id ^ 1];
			if (REVERSE_COULD_FAIL) {
				if (reverse && ! could_reverse_func(segment_id)) {
					failed = true;
					break;
				}
			} else {
				assert(! reverse || could_reverse_func(segment_id));
			}
			out.emplace_back(segment_id, reverse);
			first_point = second_point->edge_out;
		} while (first_point != nullptr);
		if (REVERSE_COULD_FAIL) {
			if (failed) {
				if (start_near == nullptr) {
					// We may try the reverse order.
					out.clear();
					first_point = last_point;
					failed = false;
					do {
						assert(out.size() < num_segments);
						size_t    		 first_point_id = first_point - &end_points.front();
						size_t           segment_id 	= first_point_id >> 1;
						bool             reverse        = (first_point_id & 1) != 0;
						EndPoint 		*second_point   = &end_points[first_point_id ^ 1];
						if (reverse && ! could_reverse_func(segment_id)) {
							failed = true;
							break;
						}
						out.emplace_back(segment_id, reverse);
						first_point = second_point->edge_out;
					} while (first_point != nullptr);
				}
			}
			if (false) {}
		} else {
			assert(! failed);
		}
	}

	assert(out.size() == num_segments);
	return out;
}

template<typename PointType, typename SegmentEndPointFunc>
std::vector<std::pair<size_t, bool>> chain_segments_greedy2(SegmentEndPointFunc end_point_func, size_t num_segments, const PointType *start_near)
{
	auto could_reverse_func = [](size_t /* idx */) -> bool { return true; };
	return chain_segments_greedy_constrained_reversals2_<PointType, SegmentEndPointFunc, false, decltype(could_reverse_func)>(end_point_func, could_reverse_func, num_segments, start_near);
}
