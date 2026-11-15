#include "libslic3r/GCode/Thumbnails.hpp"

#include <fstream>
#include <iostream>
#include <iterator>
#include <nlohmann/json.hpp>

// Runtime observer of v2.5.0 GCodeThumbnails; no encoder or output-layout copy.
int main(int argc, char** argv)
{
    if (argc != 4) {
        std::cerr << "usage: gcode-thumbnail-observer request.json blocks.gcode result.json\n";
        return 2;
    }
    std::ifstream request_file(argv[1]);
    const auto request = nlohmann::json::parse(request_file);
    const auto [definitions, errors] = Slic3r::GCodeThumbnails::make_and_check_thumbnail_list(
        request.at("thumbnails").get<std::string>());
    nlohmann::json result = {{"calls", nlohmann::json::array()}, {"cancellation_checks", 0}};
    std::string blocks;
    if (errors != Slic3r::ThumbnailErrors()) {
        result["error"] = "Invalid thumbnails value:" + Slic3r::GCodeThumbnails::get_error_string(errors);
    } else {
        Slic3r::ThumbnailsGeneratorCallback callback = [&](const Slic3r::ThumbnailsParams& params) {
            nlohmann::json call = {
                {"sizes", nlohmann::json::array()}, {"plate_id", params.plate_id},
                {"printable_only", params.printable_only}, {"parts_only", params.parts_only},
                {"show_bed", params.show_bed}, {"transparent_background", params.transparent_background},
                {"use_plate_box", params.use_plate_box}
            };
            Slic3r::ThumbnailsList images;
            for (const auto& size : params.sizes) {
                call["sizes"].push_back({size.x(), size.y()});
                for (const auto& frame : request.at("frames")) {
                    if (frame.at("size").at(0).get<double>() != size.x() ||
                        frame.at("size").at(1).get<double>() != size.y())
                        continue;
                    Slic3r::ThumbnailData data;
                    data.width = frame.at("width").get<unsigned int>();
                    data.height = frame.at("height").get<unsigned int>();
                    std::ifstream pixels(frame.at("rgba").get<std::string>(), std::ios::binary);
                    data.pixels.assign(std::istreambuf_iterator<char>(pixels), std::istreambuf_iterator<char>());
                    images.push_back(std::move(data));
                }
            }
            result["calls"].push_back(std::move(call));
            return images;
        };
        Slic3r::GCodeThumbnails::export_thumbnails_to_file(
            callback, request.at("plate_id").get<int>(), definitions,
            [&](const char* text) { blocks.append(text); },
            [&] { result["cancellation_checks"] = result["cancellation_checks"].get<int>() + 1; });
    }
    std::ofstream output(argv[2], std::ios::binary);
    output.write(blocks.data(), blocks.size());
    result["bytes"] = blocks.size();
    std::ofstream metadata(argv[3]);
    metadata << result.dump(2) << '\n';
    return 0;
}
