// Prepared only: no model, session, graph, decoder or inference.
// Tests the exact candidate validation seam used by Forward, using real ORT tensors.
#include "sherpa-onnx/csrc/giga344-validation.h"
#include "sherpa-onnx/csrc/features.h"
#include <algorithm>
#include <functional>
#include <iostream>
#include <limits>
using sherpa_onnx::giga344::Input;
using sherpa_onnx::giga344::Output;
Ort::AllocatorWithDefaultOptions allocator;
int cases = 0;
void require(bool ok) { if (!ok) throw std::runtime_error("boundary assertion failed"); }
void reject(const std::function<void()> &call) {
  bool refused = false;
  try { call(); } catch (const std::runtime_error &) { refused = true; }
  require(refused); ++cases;
}
template<class T> Ort::Value tensor(std::vector<int64_t> shape, T value) {
  auto t = Ort::Value::CreateTensor<T>(allocator, shape.data(), shape.size());
  std::fill_n(t.template GetTensorMutableData<T>(),
              t.GetTensorTypeAndShapeInfo().GetElementCount(), value);
  return t;
}
std::vector<Ort::Value> output(int64_t time, int32_t length) {
  std::vector<Ort::Value> v;
  v.push_back(tensor<float>({1,time,71}, -1));
  v.push_back(tensor<int32_t>({1}, length));
  return v;
}
int main() {
 try {
  for (float bad:{-1.0f,std::numeric_limits<float>::infinity(),
                  -std::numeric_limits<float>::infinity(),std::numeric_limits<float>::quiet_NaN()})
    reject([&]{sherpa_onnx::Giga344LogEnergy(bad);});
  require(sherpa_onnx::Giga344LogEnergy(0)==std::log(1e-9f));
  require(sherpa_onnx::Giga344LogEnergy(1)==0);
  require(sherpa_onnx::Giga344LogEnergy(1e10f)==std::log(1e9f));++cases;
  // Explicit source-derived cases, not a duplicated formula oracle.
  const std::vector<int64_t> frames{1,2,3,4,5,6,7,8,9,10,11,12,99,100,101,2499};
  const std::vector<int64_t> encoded{1,1,1,1,2,2,2,2,3,3,3,3,25,25,26,625};
  for (size_t i=0; i<frames.size(); ++i) {
    auto f=tensor<float>({1,frames[i],64},0);
    auto l=tensor<int64_t>({1},frames[i]);
    auto expected=Input(f,l); require(expected==std::vector<int64_t>{encoded[i]});
    auto v=output(encoded[i],encoded[i]);
    require(Output(v,1,frames[i],expected)==expected); ++cases;
    // The input is valid; only returned graph length changes. Must be refused.
    v[1].GetTensorMutableData<int32_t>()[0]-=1;
    reject([&]{Output(v,1,frames[i],expected);});
  }
  auto f=tensor<float>({1,101,64},0); auto l=tensor<int64_t>({1},101);
  for (auto n:{0,-1,102}) {l.GetTensorMutableData<int64_t>()[0]=n; reject([&]{Input(f,l);});}
  l.GetTensorMutableData<int64_t>()[0]=101;
  for (auto shape:std::vector<std::vector<int64_t>>{{101,64},{2,101,64},{1,101,63},{1,2500,64},{1,0,64}}) {
    auto bad=tensor<float>(shape,0);reject([&]{Input(bad,l);});
  }
  auto wrong=tensor<int32_t>({1},101);reject([&]{Input(f,wrong);});
  auto rank=tensor<int64_t>({1,1},101);reject([&]{Input(f,rank);});
  auto integer=tensor<int32_t>({1,101,64},0);reject([&]{Input(integer,l);});
  std::vector<Ort::Value> items; items.push_back(tensor<float>({1},0));
  auto sequence=Ort::Value::CreateSequence(items);reject([&]{Input(sequence,l);});
  auto expected=Input(f,l);
  for (auto n:{-1,0,25,27,1000}) {auto v=output(26,n);reject([&]{Output(v,1,101,expected);});}
  for (auto shape:std::vector<std::vector<int64_t>>{{26,71},{2,26,71},{1,25,71},{1,26,70}}) {
    auto v=output(26,26);v[0]=tensor<float>(shape,-1);reject([&]{Output(v,1,101,expected);});
  }
  auto v=output(26,26);v[1]=tensor<int64_t>({1},26);reject([&]{Output(v,1,101,expected);});
  v=output(26,26);v[1]=tensor<int32_t>({1,1},26);reject([&]{Output(v,1,101,expected);});
  v=output(26,26);v[0]=tensor<int32_t>({1,26,71},-1);reject([&]{Output(v,1,101,expected);});
  for (float bad:{std::numeric_limits<float>::infinity(),-std::numeric_limits<float>::infinity(),std::numeric_limits<float>::quiet_NaN()}) {
    v=output(26,26);v[0].GetTensorMutableData<float>()[100]=bad;
    reject([&]{Output(v,1,101,expected);});
  }
  v.clear();reject([&]{Output(v,1,101,expected);});
  // Different per-item lengths in one padded batch must not leak/reset incorrectly.
  auto bf=tensor<float>({2,101,64},0);auto bl=tensor<int64_t>({2},1);
  bl.GetTensorMutableData<int64_t>()[1]=101;auto be=Input(bf,bl);
  v.push_back(tensor<float>({2,26,71},-1));v.push_back(tensor<int32_t>({2},1));
  v[1].GetTensorMutableData<int32_t>()[1]=26;
  require(Output(v,2,101,be)==std::vector<int64_t>({1,26}));++cases;
  std::cout << "synthetic validation cases: " << cases << "; no model/session\n";
 } catch (const std::exception &e) {std::cerr << e.what() << '\n';return 1;}
}
