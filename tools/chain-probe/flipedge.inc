struct FlipEdge {
	FlipEdge(const Vec2d &p1, const Vec2d &p2, size_t source_index) : p1(p1), p2(p2), source_index(source_index) {}
	void	flip() { std::swap(this->p1, this->p2); }
	Vec2d	p1;
	Vec2d	p2;
	size_t	source_index;
};

struct ConnectionCost {
	ConnectionCost(double cost, double cost_flipped) : cost(cost), cost_flipped(cost_flipped) {}
	ConnectionCost() : cost(0.), cost_flipped(0.) {}
	void	flip() { std::swap(this->cost, this->cost_flipped); }
	double	cost = 0;
	double	cost_flipped = 0;
};
static inline ConnectionCost operator-(const ConnectionCost &lhs, const ConnectionCost& rhs) { return ConnectionCost(lhs.cost - rhs.cost, lhs.cost_flipped - rhs.cost_flipped); }

