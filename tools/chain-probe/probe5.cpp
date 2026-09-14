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
        std::string join = "miter";
        int n = 0;
        {
            std::istringstream head(line.substr(2));
            std::string tok, jointok;
            head >> tok; // delta=...
            delta = std::stod(tok.substr(6));
            head >> jointok; // join=...
            join = jointok.substr(5);
            head >> tok; // miter=...
            miter = std::stod(tok.substr(6));
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

        ClipperLib::JoinType jt = join == "round" ? ClipperLib::jtRound : (join == "square" ? ClipperLib::jtSquare : ClipperLib::jtMiter);
        ClipperLib::ClipperOffset co(miter);
        co.AddPath(subject, jt, ClipperLib::etClosedPolygon);
        ClipperLib::Paths solution;
        co.Execute(solution, delta);
        cases++;
        if (static_cast<int>(subject.size()) != n) {
            std::printf("PARSE MISMATCH\n");
            continue;
        }
        // Compare as point sets: same path count and identical point lists
        // in order (after matching by first-point lookup).
        auto cyclic_equal = [](const ClipperLib::Path &a, const ClipperLib::Path &b, bool allow_reverse) {
            if (a.size() != b.size() || a.empty()) return a.size() == b.size();
            for (size_t off = 0; off < a.size(); ++off) {
                bool ok = true;
                for (size_t i = 0; i < a.size(); ++i)
                    if (b[(off + i) % a.size()] != a[i]) { ok = false; break; }
                if (ok) return true;
                if (allow_reverse) {
                    ok = true;
                    for (size_t i = 0; i < a.size(); ++i)
                        if (b[(off + a.size() - i) % a.size()] != a[i]) { ok = false; break; }
                    if (ok) return true;
                }
            }
            return false;
        };
        bool same = solution.size() == ares.size();
        const bool reversed_ok = argc > 2 && std::string(argv[2]) == "allow-reversed";
        if (same) {
            for (const ClipperLib::Path &ares_path : ares) {
                bool found = false;
                for (const ClipperLib::Path &vendor : solution) {
                    if (cyclic_equal(vendor, ares_path, reversed_ok)) { found = true; break; }
                }
                if (!found) { same = false; break; }
            }
        }
        compared++;
        if (!same) {
            mismatched++;
            if (mismatched <= 3) {
                std::printf(
                    "MISMATCH case %ld: delta=%.3f n=%d vendor=%zu ares=%zu\n",
                    cases, delta, n, solution.size(), ares.size());
                for (size_t i = 0; i < solution.size() && i < 1; ++i) {
                    std::printf("  V n=%zu", solution[i].size());
                    for (auto &pt : solution[i]) std::printf(" (%lld,%lld)", (long long)pt.x(), (long long)pt.y());
                    std::printf("\n");
                }
                for (size_t i = 0; i < ares.size() && i < 1; ++i) {
                    std::printf("  A n=%zu", ares[i].size());
                    for (auto &pt : ares[i]) std::printf(" (%lld,%lld)", (long long)pt.x(), (long long)pt.y());
                    std::printf("\n");
                }
            }
        }
    }
    std::printf("%ld offsets, %ld compared, %ld mismatched\n", cases, compared, mismatched);
    return 0;
}
