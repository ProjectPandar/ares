// Probe 5: run the vendored Clipper 6 ClipperOffset on ARES_DUMP_OFFSET
// records (input path + applied delta) and print the outputs for
// comparison with ares's offset port (the R lines).
#include "clipper/clipper.hpp"

#include <cstdio>
#include <fstream>
#include <iostream>
#include <sstream>
#include <string>
#include <vector>

int main(int argc, char **argv) {
    const char *path = argc > 1 ? argv[1] : "/tmp/off-1e6.txt";
    std::ifstream in(path);
    std::string line;
    long cases = 0, compared = 0, mismatched = 0;
    while (std::getline(in, line)) {
        if (line.compare(0, 2, "O ") != 0)
            continue;
        // Parse "O delta=<d> join=<m> n=<n> (x,y) ..."
        double delta = 0, miter = 0;
        int n = 0;
        {
            std::istringstream head(line.substr(2));
            std::string tok;
            head >> tok; // delta=...
            delta = std::stod(tok.substr(6));
            head >> tok; // join=...
            miter = std::stod(tok.substr(5));
            head >> tok; // n=...
            n = std::stoi(tok.substr(2));
        }
        size_t pos = 0;
        ClipperLib::Path subject;
        while ((pos = line.find('(', pos)) != std::string::npos) {
            size_t end = line.find(')', pos);
            std::string pair = line.substr(pos + 1, end - pos - 1);
            size_t comma = pair.find(',');
            subject.emplace_back(std::stoll(pair.substr(0, comma)),
                                 std::stoll(pair.substr(comma + 1)));
            pos = end;
        }
        // Collect the ares result lines that follow.
        std::vector<ClipperLib::Path> ares;
        std::streampos back = in.tellg();
        while (std::getline(in, line)) {
            if (line.compare(0, 2, "R ") == 0) {
                size_t p = 0;
                ClipperLib::Path poly;
                while ((p = line.find('(', p)) != std::string::npos) {
                    size_t end = line.find(')', p);
                    std::string pair = line.substr(p + 1, end - p - 1);
                    size_t comma = pair.find(',');
                    poly.emplace_back(std::stoll(pair.substr(0, comma)),
                                      std::stoll(pair.substr(comma + 1)));
                    p = end;
                }
                ares.push_back(poly);
                back = in.tellg();
            } else {
                break;
            }
        }
        in.seekg(back);

        ClipperLib::ClipperOffset co(miter);
        co.AddPath(subject, ClipperLib::jtMiter, ClipperLib::etClosedPolygon);
        ClipperLib::Paths solution;
        co.Execute(solution, delta);
        cases++;
        if (static_cast<int>(subject.size()) != n) {
            std::printf("PARSE MISMATCH\n");
            continue;
        }
        // Compare as point sets: same path count and identical point lists
        // in order (after matching by first-point lookup).
        bool same = solution.size() == ares.size();
        if (same) {
            for (const ClipperLib::Path &ares_path : ares) {
                bool found = false;
                for (const ClipperLib::Path &vendor : solution) {
                    if (vendor.size() == ares_path.size() &&
                        vendor[0] == ares_path[0] &&
                        std::equal(vendor.begin(), vendor.end(), ares_path.begin())) {
                        found = true;
                        break;
                    }
                }
                if (!found) {
                    same = false;
                    break;
                }
            }
        }
        compared++;
        if (!same) {
            mismatched++;
            if (mismatched <= 3) {
                std::printf(
                    "MISMATCH case %ld: delta=%.3f n=%d vendor=%zu ares=%zu\n",
                    cases, delta, n, solution.size(), ares.size());
                for (size_t i = 0; i < solution.size() && i < 2; ++i) {
                    std::printf("  V[0]=(%lld,%lld) n=%zu\n", (long long)solution[i][0].x(),
                                (long long)solution[i][0].y(), solution[i].size());
                }
                for (size_t i = 0; i < ares.size() && i < 2; ++i) {
                    std::printf("  A[0]=(%lld,%lld) n=%zu\n", (long long)ares[i][0].x(),
                                (long long)ares[i][0].y(), ares[i].size());
                }
            }
        }
    }
    std::printf("%ld offsets, %ld compared, %ld mismatched\n", cases, compared, mismatched);
    return 0;
}
