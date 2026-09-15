// Ordering of children cubes.
static const std::array<Vec3d, 8> child_centers {
    Vec3d(-1, -1, -1), Vec3d( 1, -1, -1), Vec3d(-1,  1, -1), Vec3d( 1,  1, -1),
    Vec3d(-1, -1,  1), Vec3d( 1, -1,  1), Vec3d(-1,  1,  1), Vec3d( 1,  1,  1)
};

// Traversal order of octree children cells for three infill directions,
// so that a single line will be discretized in a strictly monotonic order.
static constexpr std::array<std::array<int, 8>, 3> child_traversal_order {
    std::array<int, 8>{ 2, 3, 0, 1, 6, 7, 4, 5 },
    std::array<int, 8>{ 4, 0, 6, 2, 5, 1, 7, 3 },
    std::array<int, 8>{ 1, 5, 0, 4, 3, 7, 2, 6 },
};

struct Cube
{
    Vec3d center;
#ifndef NDEBUG
    Vec3d center_octree;
#endif // NDEBUG
    std::array<Cube*, 8> children {}; // initialized to nullptrs
    Cube(const Vec3d &center) : center(center) {}
};

struct CubeProperties
{
    double edge_length;     // Lenght of edge of a cube
    double height;          // Height of rotated cube (standing on the corner)
    double diagonal_length; // Length of diagonal of a cube a face
    double line_z_distance; // Defines maximal distance from a center of a cube on Z axis on which lines will be created
    double line_xy_distance;// Defines maximal distance from a center of a cube on X and Y axis on which lines will be created
};

struct Octree
{
    // Octree will allocate its Cubes from the pool. The pool only supports deletion of the complete pool,
    // perfect for building up our octree.
    object_pool<Cube>    pool;
    Cube*                       root_cube { nullptr };
    Vec3d                       origin;
    std::vector<CubeProperties> cubes_properties;

    Octree(const Vec3d &origin, const std::vector<CubeProperties> &cubes_properties)
        : root_cube(pool.construct(origin)), origin(origin), cubes_properties(cubes_properties) {}

    void insert_triangle(const Vec3d &a, const Vec3d &b, const Vec3d &c, Cube *current_cube, const BoundingBoxf3 &current_bbox, int depth);
};

void OctreeDeleter::operator()(Octree *p) {
    delete p;
}

// Context used by generate_infill_lines() when recursively traversing an octree in a DDA fashion
// (Digital Differential Analyzer).
struct FillContext
{
    // The angles have to agree with child_traversal_order.
    static constexpr double direction_angles[3] {
        0.,
        (2.0 * M_PI) / 3.0,
        -(2.0 * M_PI) / 3.0
    };

    FillContext(const Octree &octree, double z_position, int direction_idx) :
        cubes_properties(octree.cubes_properties),
        z_position(z_position),
        traversal_order(child_traversal_order[direction_idx]),
        cos_a(cos(direction_angles[direction_idx])),
        sin_a(sin(direction_angles[direction_idx]))
    {
        static constexpr auto unused = std::numeric_limits<coord_t>::max();
        temp_lines.assign((1 << octree.cubes_properties.size()) - 1, Line(Point(unused, unused), Point(unused, unused)));
    }

    // Rotate the point, uses the same convention as Point::rotate().
    Vec2d rotate(const Vec2d& v) { return Vec2d(this->cos_a * v.x() - this->sin_a * v.y(), this->sin_a * v.x() + this->cos_a * v.y()); }

    const std::vector<CubeProperties>  &cubes_properties;
    // Top of the current layer.
    const double                        z_position;
    // Order of traversal for this line direction.
    const std::array<int, 8>            traversal_order;
    // Rotation of the generated line for this line direction.
    const double                        cos_a;
    const double                        sin_a;

    // Linearized tree spanning a single Octree wall, used to connect lines spanning
    // neighboring Octree cells. Unused lines have the Line::a::x set to infinity.
    std::vector<Line>                   temp_lines;
    // Final output
    std::vector<Line>                   output_lines;
};




static void generate_infill_lines_recursive(
    FillContext     &context,
    const Cube      *cube,
    // Address of this wall in the octree,  used to address context.temp_lines.
    int              address,
    int              depth)
{
    assert(cube != nullptr);

    const std::vector<CubeProperties> &cubes_properties = context.cubes_properties;
    const double z_diff     = context.z_position - cube->center.z();
    const double z_diff_abs = std::abs(z_diff);

    if (z_diff_abs > cubes_properties[depth].height / 2.)
        return;

    if (z_diff_abs < cubes_properties[depth].line_z_distance) {
        // Discretize a single wall splitting the cube into two.
        const double zdist = cubes_properties[depth].line_z_distance;
        Vec2d from(
            0.5 * cubes_properties[depth].diagonal_length * (zdist - z_diff_abs) / zdist,
            cubes_properties[depth].line_xy_distance - (zdist + z_diff) / sqrt(2.));
        Vec2d to(-from.x(), from.y());
        from = context.rotate(from);
        to   = context.rotate(to);
        // Relative to cube center
        const Vec2d offset(cube->center.x(), cube->center.y());
        from += offset;
        to   += offset;
        // Verify that the traversal order of the octree children matches the line direction,
        // therefore the infill line may get extended with O(1) time & space complexity.
        assert(verify_traversal_order(context, cube, depth, from, to));
        // Either extend an existing line or start a new one.
        Line &last_line = context.temp_lines[address];
        Line  new_line(Point::new_scale(from), Point::new_scale(to));
        if (last_line.a.x() == std::numeric_limits<coord_t>::max()) {
            last_line.a = new_line.a;
        } else if ((new_line.a - last_line.b).cwiseAbs().maxCoeff() > 1000) { // SCALED_EPSILON is 100 and it is not enough
            context.output_lines.emplace_back(last_line);
            last_line.a = new_line.a;
        }
        last_line.b = new_line.b;
    }

    // left child index
    address = address * 2 + 1;
    -- depth;
    size_t i = 0;
    for (const int child_idx : context.traversal_order) {
        const Cube *child = cube->children[child_idx];
        if (child != nullptr)
            generate_infill_lines_recursive(context, child, address, depth);
        if (++ i == 4)
            // right child index
            ++ address;
    }
}


static std::vector<CubeProperties> make_cubes_properties(double max_cube_edge_length, double line_spacing)
{
    max_cube_edge_length += EPSILON;

    std::vector<CubeProperties> cubes_properties;
    for (double edge_length = line_spacing * 2.;; edge_length *= 2.)
    {
        CubeProperties props{};
        props.edge_length = edge_length;
        props.height = edge_length * sqrt(3);
        props.diagonal_length = edge_length * sqrt(2);
        props.line_z_distance = edge_length / sqrt(3);
        props.line_xy_distance = edge_length / sqrt(6);
        cubes_properties.emplace_back(props);
        if (edge_length > max_cube_edge_length)
            break;
    }
    // Orca: Ensure at least 2 levels so build_octree() will insert triangles.
    // Fixes scenario where adaptive fill is disconnected from walls on low densities
    if (cubes_properties.size() == 1) {
        CubeProperties p = cubes_properties.back();
        p.edge_length      *= 2.0;
        p.height           = p.edge_length * sqrt(3);
        p.diagonal_length  = p.edge_length * sqrt(2);
        p.line_z_distance  = p.edge_length / sqrt(3);
        p.line_xy_distance = p.edge_length / sqrt(6);
        cubes_properties.push_back(p);
    }
    return cubes_properties;
}

static inline bool is_overhang_triangle(const Vec3d &a, const Vec3d &b, const Vec3d &c, const Vec3d &up)
{
    // Calculate triangle normal.
    Vec3d n = (b - a).cross(c - b);
    return n.dot(up) > 0.707 * n.norm();
}

static void transform_center(Cube *current_cube, const P8M3 &rot)
{
#ifndef NDEBUG
    current_cube->center_octree = current_cube->center;
#endif // NDEBUG
    current_cube->center = p8_apply(rot, current_cube->center);
    for (auto *child : current_cube->children)
        if (child)
            transform_center(child, rot);
}

OctreePtr build_octree(
    // Mesh is rotated to the coordinate system of the octree.
    const P8Mesh &triangle_mesh,
    // Overhang triangles extracted from fill surfaces with stInternalBridge type,
    // rotated to the coordinate system of the octree.
    const std::vector<Vec3d>    &overhang_triangles, 
    coordf_t                     line_spacing,
    bool                         support_overhangs_only)
{
    assert(line_spacing > 0);
    assert(! std::isnan(line_spacing));

    Vec3d bmin(1e30,1e30,1e30), bmax(-1e30,-1e30,-1e30);
    for (const Vec3d &p8v : triangle_mesh.vertices) { bmin = bmin.cwiseMin(p8v); bmax = bmax.cwiseMax(p8v); }
    BoundingBoxf3 bbox(bmin, bmax);
    Vec3d                       cube_center      = bbox.center();
    std::vector<CubeProperties> cubes_properties = make_cubes_properties((std::max)({bbox.size().x(), bbox.size().y(), bbox.size().z()}), line_spacing);
    auto                        octree           = OctreePtr(new Octree(cube_center, cubes_properties));

    if (cubes_properties.size() > 1) {
        Octree *octree_ptr = octree.get();
        double edge_length_half = 0.5 * cubes_properties.back().edge_length;
        Vec3d  diag_half(edge_length_half, edge_length_half, edge_length_half);
        int    max_depth = int(cubes_properties.size()) - 1;
        auto process_triangle = [octree_ptr, max_depth, diag_half](const Vec3d &a, const Vec3d &b, const Vec3d &c) {
            octree_ptr->insert_triangle(
                a, b, c,
                octree_ptr->root_cube,
                BoundingBoxf3(octree_ptr->root_cube->center - diag_half, octree_ptr->root_cube->center + diag_half),
                max_depth);
        };
        auto up_vector = support_overhangs_only ? Vec3d(p8_to_octree_rotation() * Vec3d(0., 0., 1.)) : Vec3d();
        for (auto &tri : triangle_mesh.indices) {
            Vec3d a = triangle_mesh.vertices[tri[0]];
            Vec3d b = triangle_mesh.vertices[tri[1]];
            Vec3d c = triangle_mesh.vertices[tri[2]];
            if (! support_overhangs_only || is_overhang_triangle(a, b, c, up_vector))
                process_triangle(a, b, c);
        }
        for (size_t i = 0; i < overhang_triangles.size(); i += 3)
            process_triangle(overhang_triangles[i], overhang_triangles[i + 1], overhang_triangles[i + 2]);
        {
            p8_rotate_to_world(octree.get());
        }
    }

    return octree;
}

void Octree::insert_triangle(const Vec3d &a, const Vec3d &b, const Vec3d &c, Cube *current_cube, const BoundingBoxf3 &current_bbox, int depth)
{
    assert(current_cube);
    assert(depth > 0);

    --depth;

    // Squared radius of a sphere around the child cube.
    // const double r2_cube = Slic3r::sqr(0.5 * this->cubes_properties[depth].height + EPSILON);

    for (size_t i = 0; i < 8; ++ i) {
        const Vec3d &child_center_dir = child_centers[i];
        // Calculate a slightly expanded bounding box of a child cube to cope with triangles touching a cube wall and other numeric errors.
        // We will rather densify the octree a bit more than necessary instead of missing a triangle.
        BoundingBoxf3 bbox;
        for (int k = 0; k < 3; ++ k) {
            if (child_center_dir[k] == -1.) {
                bbox.min[k] = current_bbox.min[k];
                bbox.max[k] = current_cube->center[k] + EPSILON;
            } else {
                bbox.min[k] = current_cube->center[k] - EPSILON;
                bbox.max[k] = current_bbox.max[k];
            }
        }
        Vec3d child_center = current_cube->center + (child_center_dir * (this->cubes_properties[depth].edge_length / 2.));
        //if (dist2_to_triangle(a, b, c, child_center) < r2_cube) {
        // dist2_to_triangle and r2_cube are commented out too.
        if (triangle_AABB_intersects(a, b, c, bbox)) {
            if (! current_cube->children[i])
                current_cube->children[i] = this->pool.construct(child_center);
            if (depth > 0)
                this->insert_triangle(a, b, c, current_cube->children[i], bbox, depth);
        }
    }
}

