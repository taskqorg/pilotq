// Candidate-only feature accessor, not PilotQ runtime code. No model interface.
// Uses public kaldi-native-fbank v1.22.4 APIs; build/run awaits PM review.
#include "kaldi-native-fbank/csrc/feature-fbank.h"
#include <cmath>
#include <cstdint>
#include <filesystem>
#include <fstream>
#include <iostream>
#include <stdexcept>
#include <vector>

namespace fs = std::filesystem;
static void save(const fs::path &p, const std::vector<float> &v) {
  if (fs::exists(p)) throw std::runtime_error("output already exists");
  std::ofstream f(p, std::ios::binary);
  f.write(reinterpret_cast<const char *>(v.data()), v.size()*sizeof(float));
  if (!f) throw std::runtime_error("output write failed");
}
int main(int argc,char **argv) {
  try {
    if (argc!=3) throw std::runtime_error("usage: accessor PCM_FLOAT32LE OUTPUT_NEW_DIRECTORY");
    const uint32_t endian=1;
    if (sizeof(float)!=4 || *reinterpret_cast<const uint8_t *>(&endian)!=1)
      throw std::runtime_error("requires little-endian float32 host");
    const fs::path input(argv[1]), out(argv[2]);
    const auto bytes=fs::file_size(input);
    if (bytes%4 || bytes>400000*4) throw std::runtime_error("bounded PCM size invalid");
    std::vector<float> wave(bytes/4);
    std::ifstream f(input,std::ios::binary);
    if (bytes && !f.read(reinterpret_cast<char *>(wave.data()),bytes))
      throw std::runtime_error("PCM read failed");
    for (float v:wave) if (!std::isfinite(v) || v < -1 || v > 1)
      throw std::runtime_error("PCM finite/range guard failed");
    if (!fs::create_directory(out)) throw std::runtime_error("output directory must be new");
    knf::FbankOptions o;
    auto &w=o.frame_opts;
    w.samp_freq=16000;w.frame_length_ms=20;w.frame_shift_ms=10;
    w.snip_edges=true;w.window_type="hann";w.dither=0;
    w.remove_dc_offset=false;w.preemph_coeff=0;w.round_to_power_of_two=false;
    auto &m=o.mel_opts;
    m.num_bins=64;m.low_freq=0;m.high_freq=8000;m.is_librosa=true;
    m.use_slaney_mel_scale=false;m.norm="";m.floor_to_int_bin=false;
    o.use_energy=false;o.use_power=true;o.use_log_fbank=false;
    const int32_t frames=knf::NumFrames(wave.size(),w,true);
    if (frames!=static_cast<int32_t>(wave.size()<320?0:1+(wave.size()-320)/160))
      throw std::runtime_error("exact frame count mismatch");
    knf::FeatureWindowFunction window(w);
    knf::MelBanks banks(m,w,1.0f);
    knf::FbankComputer computer(o);
    std::vector<float> features(static_cast<size_t>(frames)*64), frame;
    for (int32_t i=0;i<frames;++i) {
      knf::ExtractWindow(0,wave,i,w,window,&frame);
      computer.Compute(0,1.0f,&frame,features.data()+i*64);
    }
    for(float v:features) if (!std::isfinite(v)) throw std::runtime_error("nonfinite mel");
    save(out/"mel.f32le",features);
    save(out/"window.f32le",window.GetWindow());
    auto matrix=banks.GetMatrix();
    if (matrix.size()!=64*161) throw std::runtime_error("mel matrix shape mismatch");
    save(out/"weights-64x161.f32le",matrix);
    std::ofstream settings(out/"options.txt");settings<<o.ToString();
    if (!settings) throw std::runtime_error("settings write failed");
    std::cout<<"samples="<<wave.size()<<" frames="<<frames<<" bins=64\n";
    return 0;
  } catch(const std::exception &e) {std::cerr<<e.what()<<'\n';return 1;}
}
