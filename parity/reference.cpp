// Local validation only; excluded from the published Rust crate.
// NVIDIA FLIP v1.7, BSD-3-Clause; see ../LICENSE for the full notice.
#include "FLIP.h"
#include "tool/pooling.h"
#include <chrono>
#include <cstdint>
#include <stdexcept>

template<class T> void write(std::ofstream& out, T value) {
    out.write(reinterpret_cast<const char*>(&value), sizeof(T));
}
std::vector<float> read(const char* path, size_t n) {
    std::ifstream in(path, std::ios::binary);
    std::vector<float> v(n);
    in.read(reinterpret_cast<char*>(v.data()), n * sizeof(float));
    if (!in || in.peek() != std::char_traits<char>::eof()) throw std::runtime_error("input length mismatch");
    return v;
}
int main(int argc, char** argv) {
    try {
        if (argc != 13) throw std::runtime_error("usage: reference ldr|hdr w h ppd tm start|auto stop|auto count|auto ref.f32 test.f32 out.bin repeats");
        bool hdr = std::string(argv[1]) == "hdr";
        int w = std::stoi(argv[2]), h = std::stoi(argv[3]);
        if (w <= 0 || h <= 0) throw std::runtime_error("invalid dimensions");
        size_t n = size_t(w) * size_t(h);
        auto r = read(argv[9], n * 3), t = read(argv[10], n * 3);
        FLIP::image<FLIP::color3> ref, test;
        ref.setPixels(r.data(), w, h); test.setPixels(t.data(), w, h);
        if (!hdr) { ref.clamp(); test.clamp(); ref.sRGBToLinearRGB(); test.sRGBToLinearRGB(); }
        FLIP::Parameters options;
        options.PPD = std::stof(argv[4]); options.tonemapper = argv[5];
        if (std::string(argv[6]) != "auto") options.startExposure = std::stof(argv[6]);
        if (std::string(argv[7]) != "auto") options.stopExposure = std::stof(argv[7]);
        if (std::string(argv[8]) != "auto") options.numExposures = std::stoi(argv[8]);
        int repeats = std::stoi(argv[12]);
        if (repeats <= 0) throw std::runtime_error("invalid repeat count");
        FLIP::image<float> error(w, h, 0.0f), exposure(w, h, 0.0f);
        std::vector<double> times;
        FLIP::Parameters used;
        // One warmup for benchmark mode; no warmup for one-shot parity.
        for (int i = (repeats > 1 ? -1 : 0); i < repeats; ++i) {
            error.clear(); exposure.clear(); used = options;
            auto start = std::chrono::steady_clock::now();
            // Include sRGB conversion in LDR timing, as in the Rust API.
            if (!hdr) {
                ref.setPixels(r.data(), w, h); test.setPixels(t.data(), w, h);
                ref.clamp(); test.clamp(); ref.sRGBToLinearRGB(); test.sRGBToLinearRGB();
            }
            FLIP::evaluate(ref, test, hdr, used, error, exposure);
            auto stop = std::chrono::steady_clock::now();
            if (i >= 0) times.push_back(std::chrono::duration<double>(stop - start).count());
        }
        std::sort(times.begin(), times.end());
        FLIPPooling::pooling<float> pool;
        for (int y = 0; y < h; ++y) for (int x = 0; x < w; ++x) pool.update(x, y, error.get(x, y));
        std::ofstream out(argv[11], std::ios::binary);
        if (!out) throw std::runtime_error("cannot open output");
        // Little-endian native protocol, checked by the Rust reader; no text rounding.
        write<uint32_t>(out, 0x464c1737);
        write<float>(out, hdr ? used.startExposure : 0.0f);
        write<float>(out, hdr ? used.stopExposure : 0.0f);
        write<uint32_t>(out, hdr ? used.numExposures : 0);
        write<double>(out, times[times.size() / 2]);
        write<float>(out, pool.getMean());
        write<float>(out, pool.getPercentile(0.5f, true));
        write<float>(out, pool.getPercentile(0.25f, true));
        write<float>(out, pool.getPercentile(0.75f, true));
        write<float>(out, pool.getMinValue()); write<float>(out, pool.getMaxValue());
        for (size_t i = 0; i < 100; ++i) write<uint64_t>(out, pool.getHistogram().getBucketValue(i));
        for (int y = 0; y < h; ++y) for (int x = 0; x < w; ++x) write<float>(out, error.get(x, y));
        if (hdr) for (int y = 0; y < h; ++y) for (int x = 0; x < w; ++x) write<float>(out, exposure.get(x, y));
        if (!out) throw std::runtime_error("write failed");
        return 0;
    } catch (const std::exception& e) { std::cerr << e.what() << '\n'; return 2; }
}
