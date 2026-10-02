// Local validation only; excluded from the published Rust crate.
// NVIDIA FLIP v1.7, BSD-3-Clause; see ../LICENSE for the full notice.
#include "FLIP.h"
#include "tool/pooling.h"
#include <chrono>
#include <cstdint>
#include <cstring>
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
float fromBits(uint32_t bits) {
    float value;
    std::memcpy(&value, &bits, sizeof(value));
    return value;
}
// Search positive f32 values below 1. Never invoke the reference at an
// out-of-bounds unweighted index, including on one-pixel maps.
float largestDefinedPercentile(size_t n) {
    uint32_t low = 0, high = 0x3f7fffff;
    while (low < high) {
        uint32_t mid = low + (high - low + 1) / 2;
        if (size_t(std::ceil(n * fromBits(mid))) < n) low = mid;
        else high = mid - 1;
    }
    return fromBits(low);
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
            // Input restoration is outside timing on both sides.
            if (!hdr) { ref.setPixels(r.data(), w, h); test.setPixels(t.data(), w, h); }
            auto start = std::chrono::steady_clock::now();
            // Include sRGB conversion in LDR timing, as in the Rust API.
            if (!hdr) {
                ref.clamp(); test.clamp(); ref.sRGBToLinearRGB(); test.sRGBToLinearRGB();
            }
            FLIP::evaluate(ref, test, hdr, used, error, exposure);
            auto stop = std::chrono::steady_clock::now();
            if (i >= 0) times.push_back(std::chrono::duration<double>(stop - start).count());
        }
        std::sort(times.begin(), times.end());
        // Preserve the reference calculation, but do not feed a NaN to the
        // pooling histogram's float-to-integer conversion (undefined C++).
        for (int y = 0; y < h; ++y) for (int x = 0; x < w; ++x) {
            if (!std::isfinite(error.get(x, y))) {
                std::cerr << "reference produced nonfinite error pixels; pooling would be undefined\n";
                return 3;
            }
        }
        FLIPPooling::pooling<float> pool;
        for (int y = 0; y < h; ++y) for (int x = 0; x < w; ++x) pool.update(x, y, error.get(x, y));
        std::ofstream out(argv[11], std::ios::binary);
        if (!out) throw std::runtime_error("cannot open output");
        // Little-endian native protocol, checked by the Rust reader; no text rounding.
        write<uint32_t>(out, 0x464c1738);
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
        const float fractions[] = {0.0f, 0.01f, 0.25f, 0.5f, 0.75f, 0.9f, 0.95f, 0.99f, 0.999f};
        write<uint32_t>(out, 10);
        for (size_t i = 0; i < 10; ++i) {
            float weightedP = i < 9 ? fractions[i] : fromBits(0x3f7fffff);
            float unweightedP = i < 9 ? fractions[i] : largestDefinedPercentile(n);
            bool defined = size_t(std::ceil(n * unweightedP)) < n;
            write<float>(out, weightedP);
            write<float>(out, unweightedP);
            write<float>(out, pool.getPercentile(weightedP, true));
            write<uint32_t>(out, defined ? 1 : 0);
            write<float>(out, defined ? pool.getPercentile(unweightedP, false) : 0.0f);
        }
        for (int y = 0; y < h; ++y) for (int x = 0; x < w; ++x) write<float>(out, error.get(x, y));
        if (hdr) for (int y = 0; y < h; ++y) for (int x = 0; x < w; ++x) write<float>(out, exposure.get(x, y));
        if (!out) throw std::runtime_error("write failed");
        return 0;
    } catch (const std::exception& e) { std::cerr << e.what() << '\n'; return 2; }
}
