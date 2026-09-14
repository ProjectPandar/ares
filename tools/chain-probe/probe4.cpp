#include <functional>
// Probe 4: run upstream intersection_pl (ClipperLib + PolyTreeToPolylines)
// on the ARES_DUMP_PLANESUBJECT subject and ARES_DUMP_PLANEBOUND clip region,
// printing the fragment list order/orientation for comparison with ares's
// classic_clip output.
#include "clipper/clipper.hpp"
#include "clipper/clipper.cpp"

#include <cstdio>
#include <fstream>
#include <iostream>
#include <sstream>
#include <string>
#include <vector>

struct DumpPolyline {
    std::vector<ClipperLib::IntPoint> points;
};

int main(int argc, char **argv) {
    const char *subject_path = argc > 1 ? argv[1] : "/tmp/oct-subj.txt";
    const char *bound_path = argc > 2 ? argv[2] : "/tmp/oct-bound.txt";
    const char *ares_clip = argc > 3 ? argv[3] : "/tmp/oct-clip2.txt";

    std::vector<DumpPolyline> subjects;
    {
        std::ifstream in(subject_path);
        std::string line;
        DumpPolyline current;
        bool open = false;
        while (std::getline(in, line)) {
            if (line.compare(0, 2, "S ") == 0) {
                if (open && current.points.size() >= 2)
                    subjects.push_back(current);
                current = DumpPolyline();
                open = true;
            } else if (line.compare(0, 2, "P ") == 0) {
                size_t pos = 0;
                while ((pos = line.find('(', pos)) != std::string::npos) {
                    size_t end = line.find(')', pos);
                    std::string pair = line.substr(pos + 1, end - pos - 1);
                    size_t comma = pair.find(',');
                    current.points.emplace_back(ClipperLib::IntPoint2d(std::stoll(pair.substr(0, comma)), std::stoll(pair.substr(comma + 1))));
                    pos = end;
                }
            }
        }
        if (open && current.points.size() >= 2)
            subjects.push_back(current);
    }
    ClipperLib::Paths clip;
    {
        std::ifstream in(bound_path);
        std::string line;
        ClipperLib::Path polygon;
        while (std::getline(in, line)) {
            if (line.compare(0, 2, "B ") == 0) {
                std::istringstream fields(line.substr(2));
                long long x, y;
                double dx, dy;
                if (fields >> dx >> dy) {
                    x = std::llround(dx * 1000000.0);
                    y = std::llround(dy * 1000000.0);
                    polygon.emplace_back(ClipperLib::IntPoint2d(x, y));
                }
            }
        }
        clip.push_back(polygon);
    }
    std::cout << "subjects: " << subjects.size()
              << " clip points: " << clip[0].size() << std::endl;

    ClipperLib::Clipper clipper;
    ClipperLib::Paths subject_paths;
    for (const DumpPolyline &subject : subjects)
        subject_paths.push_back(subject.points);
    clipper.AddPaths(subject_paths, ClipperLib::ptSubject, false);
    clipper.AddPaths(clip, ClipperLib::ptClip, true);
    ClipperLib::PolyTree tree;
    clipper.Execute(ClipperLib::ctIntersection, tree, ClipperLib::pftNonZero, ClipperLib::pftNonZero);
    // PolyTreeToPolylines (ClipperUtils.cpp:207-223): contour per node, DFS.
    std::vector<ClipperLib::Path> out;
    std::function<void(ClipperLib::PolyNode &)> visit = [&](ClipperLib::PolyNode &node) {
        if (!node.Contour.empty())
            out.push_back(node.Contour);
        for (ClipperLib::PolyNode *child : node.Childs)
            visit(*child);
    };
    visit(tree);
    std::printf("upstream fragments: %zu\n", out.size());
    for (size_t i = 0; i < out.size(); ++i) {
        std::printf("U %zu n=%zu", i, out[i].size());
        for (const ClipperLib::IntPoint &pt : out[i])
            std::printf(" (%lld,%lld)", (long long)pt.x(), (long long)pt.y());
        std::printf("\n");
    }
    return 0;
}
