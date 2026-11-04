// Prepared feature-only accessor: actual patched FeatureExtractor, no model/session.
// Input is retained PCM16 little-endian; output is raw returned float32 features.
#include "sherpa-onnx/csrc/features.h"
#include <cmath>
#include <fstream>
#include <iostream>
#include <iterator>
#include <stdexcept>
#include <vector>
int main(int argc, char **argv) {
 try {
  if (argc!=3) throw std::runtime_error("usage: features PCM16 OUT.float32");
  std::ifstream in(argv[1],std::ios::binary);
  if (!in) throw std::runtime_error("PCM unavailable");
  in.seekg(0,std::ios::end);auto size=in.tellg();
  if (size<0 || size>800000 || size%2) throw std::runtime_error("invalid PCM size");
  in.seekg(0);std::vector<unsigned char> bytes(static_cast<size_t>(size));
  if (size && !in.read(reinterpret_cast<char *>(bytes.data()),size))
    throw std::runtime_error("PCM read failed");
  if (bytes.size()%2 || bytes.size()>800000) throw std::runtime_error("invalid PCM size");
  std::vector<float> pcm(bytes.size()/2);
  for (size_t i=0;i<pcm.size();++i) {
    const int raw=bytes[2*i] | (int(bytes[2*i+1])<<8);
    pcm[i]=(raw>=32768 ? raw-65536 : raw)/32768.0f;
  }
  sherpa_onnx::FeatureExtractorConfig c;
  c.sampling_rate=16000;c.feature_dim=64;c.low_freq=0;c.high_freq=8000;
  c.remove_dc_offset=false;c.preemph_coeff=0;c.window_type="hann";
  c.round_to_power_of_two=false;c.dither=0;c.normalize_samples=true;
  c.frame_length_ms=20;c.frame_shift_ms=10;c.snip_edges=true;c.is_librosa=true;
#ifdef SHERPA_ONNX_EXPERIMENT_GIGA344
  c.is_giga344=true;
#else
#error candidate feature harness requires explicit experimental define
#endif
  sherpa_onnx::FeatureExtractor first(c),second(c);
  if (!pcm.empty()) first.AcceptWaveform(16000,pcm.data(),pcm.size());
  first.InputFinished();
  int frames=first.NumFramesReady();
  auto a=frames ? first.GetFrames(0,frames) : std::vector<float>{};
  auto repeat=frames ? first.GetFrames(0,frames) : std::vector<float>{};
  // Deliberately split ingestion at a non-frame boundary in a fresh stream.
  size_t split=pcm.size()/3;
  if (split) second.AcceptWaveform(16000,pcm.data(),split);
  if (pcm.size()>split) second.AcceptWaveform(16000,pcm.data()+split,pcm.size()-split);
  second.InputFinished();
  auto b=frames ? second.GetFrames(0,frames) : std::vector<float>{};
  if (second.NumFramesReady()!=frames || a!=repeat || a!=b || a.size()!=size_t(frames)*64)
    throw std::runtime_error("repeat/stream isolation mismatch");
  for (float value:a) if (!std::isfinite(value)) throw std::runtime_error("nonfinite feature");
  std::ofstream out(argv[2],std::ios::binary|std::ios::trunc);
  out.write(reinterpret_cast<const char *>(a.data()),a.size()*sizeof(float));out.close();
  if (!out) throw std::runtime_error("feature readback write failed");
  std::cout << "frames=" << frames << " bins=64 repeated/stream identical; no model\n";
 } catch (const std::exception &e) {std::cerr << e.what() << '\n';return 1;}
}
