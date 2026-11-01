// Temporary API investigation, not production code or a quality reference.
#include "whisper.h"
#include <fstream>
#include <iostream>
#include <iterator>
#include <stdexcept>
#include <string>
#include <vector>
extern "C" int pilotq344_encode_count(whisper_context *);
extern "C" void pilotq344_invalidate_mel(whisper_context *);
struct callbacks { int begins = 0, aborts = 0; bool refuse = false, cancel = false, armed = false; };
static bool begin(whisper_context *, whisper_state *, void * opaque) {
    auto & c = *static_cast<callbacks *>(opaque);
    ++c.begins;
    c.armed = c.cancel;
    return !c.refuse;
}
static bool abort_compute(void * opaque) {
    auto & c = *static_cast<callbacks *>(opaque);
    ++c.aborts;
    return c.armed;
}
static std::vector<float> read_pcm(const char * path) {
    std::ifstream input(path, std::ios::binary | std::ios::ate);
    if (!input) throw std::runtime_error("PCM missing");
    auto bytes = input.tellg();
    if (bytes <= 0 || bytes % 4 != 0 || bytes > 300 * 16000 * 4)
        throw std::runtime_error("PCM size/cap");
    std::vector<float> pcm(static_cast<size_t>(bytes) / 4);
    input.seekg(0);
    input.read(reinterpret_cast<char *>(pcm.data()), bytes);
    if (!input) throw std::runtime_error("PCM read");
    return pcm;
}
int main(int argc, char ** argv) {
    if (argc != 5) return 64; // MODEL PCM-A PCM-B CASE
    const std::string mode = argv[4];
    auto a = read_pcm(argv[2]), b = read_pcm(argv[3]);
    auto cp = whisper_context_default_params(); cp.use_gpu = false;
    auto * ctx = whisper_init_from_file_with_params(argv[1], cp);
    if (!ctx) return 65;
    auto run = [&](const std::vector<float> & pcm, const std::string & label,
                   int offset, int audio_ctx, const char * language, bool refuse, bool cancel, bool empty) {
        auto p = whisper_full_default_params(WHISPER_SAMPLING_GREEDY);
        p.n_threads = 4; p.language = language; p.no_context = true;
        p.offset_ms = offset; p.audio_ctx = audio_ctx;
        p.greedy.best_of = 1; p.temperature_inc = 0;
        p.print_progress = p.print_realtime = p.print_timestamps = false;
#ifdef PILOTQ_344_PATCHED
        p.experimental_first_window_reuse = mode != "optout";
#endif
        callbacks c; c.refuse = refuse; c.cancel = cancel;
        p.encoder_begin_callback = begin; p.encoder_begin_callback_user_data = &c;
        p.abort_callback = abort_compute; p.abort_callback_user_data = &c;
        int before = pilotq344_encode_count(ctx);
        int rc = whisper_full(ctx, p, empty ? nullptr : pcm.data(), empty ? 0 : int(pcm.size()));
        std::string text;
        for (int i = 0; i < whisper_full_n_segments(ctx); ++i) text += whisper_full_get_segment_text(ctx, i);
        static const char hex[] = "0123456789abcdef";
        std::string encoded;
        for (unsigned char ch : text) { encoded += hex[ch >> 4]; encoded += hex[ch & 15]; }
        std::cout << "RESULT\t" << label << '\t' << rc << '\t' << whisper_full_lang_id(ctx)
                  << '\t' << pilotq344_encode_count(ctx) - before << '\t' << c.begins
                  << '\t' << c.aborts << '\t' << encoded << std::endl;
    };
    if (mode == "context") {
        run(a,"reduced",0,whisper_n_audio_ctx(ctx)/2,"auto",false,false,false);
        run(b,"restore",0,0,"auto",false,false,false);
        run(a,"restored",0,0,"auto",false,false,false);
    } else if (mode == "recovery") {
        run(a,"first",0,0,"auto",false,false,false);
        // Explicit test-only fault injection: detect must fail, not reuse old audio.
        pilotq344_invalidate_mel(ctx);
        run(a,"faildetect",0,0,"auto",false,false,true);
        run(b,"second",0,0,"auto",false,false,false);
    } else if (mode == "multi") {
        run(a,"first",0,0,"auto",false,false,false);
        run(std::vector<float>(3 * 16000, 0.0f),"silence",0,0,"auto",false,false,false);
        run(b,"second",0,0,"auto",false,false,false);
    } else if (mode == "optout" || mode == "default" || mode == "second" || mode == "offset" || mode == "explicit" ||
               mode == "refuse" || mode == "cancel" || mode == "faildetect") {
        run(mode == "second" ? b : a,mode,mode == "offset" ? 1000 : 0,0,
            mode == "explicit" ? "ru" : "auto",mode == "refuse",mode == "cancel",mode == "faildetect");
    } else { whisper_free(ctx); return 64; }
    whisper_free(ctx);
}
