#!/usr/bin/env python3
"""Targeted static supplement; PM review/isolation required, never executes ONNX."""
import argparse
import importlib.metadata
import importlib.util
import json
import math
import os
from pathlib import Path
import signal
import time

HERE = Path(__file__).resolve().parent


def load(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def small_value(tensor, onnx):
    count = math.prod(tensor.dims)
    allowed = (onnx.TensorProto.FLOAT, onnx.TensorProto.DOUBLE,
               onnx.TensorProto.INT32, onnx.TensorProto.INT64)
    if tensor.data_type not in allowed or count > 16:
        raise ValueError('length constant is not approved small numeric tensor')
    value = onnx.numpy_helper.to_array(tensor)
    if not bool(__import__('numpy').isfinite(value).all()):
        raise ValueError('nonfinite length constant')
    return {'dtype': onnx.TensorProto.DataType.Name(tensor.data_type),
            'dims': list(tensor.dims), 'value': value.tolist()}


def length_lineage(graph, onnx, base):
    producers = {out: node for node in graph.node for out in node.output}
    tensors = {x.name: x for x in graph.initializer}
    pending = ['encoded_lengths']; seen = set(); nodes = []; leaves = []
    while pending:
        name = pending.pop()
        if not name or name in seen:
            continue
        seen.add(name)
        if len(seen) > 256:
            raise ValueError('length lineage exceeds cap')
        node = producers.get(name)
        if node is None:
            row = {'name': name, 'kind': 'input/unresolved'}
            if name in tensors:
                row.update(kind='initializer', numeric=small_value(tensors[name], onnx))
            leaves.append(row)
            continue
        row = base.node_summary(node, onnx)
        numeric = {}
        for attr in node.attribute:
            if attr.type == onnx.AttributeProto.TENSOR:
                numeric[attr.name] = small_value(attr.t, onnx)
            elif attr.type in (onnx.AttributeProto.FLOAT, onnx.AttributeProto.FLOATS):
                values = [attr.f] if attr.type == onnx.AttributeProto.FLOAT else list(attr.floats)
                if len(values) > 16 or not all(math.isfinite(x) for x in values):
                    raise ValueError('nonfinite/oversize length float attribute')
                numeric[attr.name] = values
        row['numeric_attributes'] = numeric
        nodes.append(row); pending.extend(node.input)
    return {'nodes': nodes, 'leaves': leaves, 'truncated': False}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--assets', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    started = time.monotonic()
    def expired(signum, frame):
        raise TimeoutError('supplement shared600s or termination')
    signal.signal(signal.SIGALRM, expired)
    signal.signal(signal.SIGTERM, expired)
    signal.setitimer(signal.ITIMER_REAL, 600)
    base = load('static344', HERE / 'inspect-giga-multilingual.py')
    guard = load('guard344', base.REPO / 'scripts/probe-speech-344.py')
    isolation = guard.limits()
    for package, version in base.VERSIONS.items():
        if importlib.metadata.version(package) != version:
            raise ValueError('private version mismatch: ' + package)
    for name, (size, digest) in base.EXPECTED.items():
        path = args.assets / name
        if path.stat().st_size != size or base.sha(path) != digest:
            raise ValueError('asset mismatch: ' + name)
    for name in ('OMP_NUM_THREADS', 'OPENBLAS_NUM_THREADS', 'MKL_NUM_THREADS', 'NUMEXPR_NUM_THREADS'):
        os.environ[name] = '4'
    os.umask(0o077)
    args.output.mkdir(mode=0o700, parents=True, exist_ok=False)
    evidence = {'status': 'incomplete', 'model_executed': False,
                'isolation': isolation, 'versions': base.VERSIONS,
                'script_sha256': base.sha(Path(__file__)),
                'base_script_sha256': base.sha(HERE / 'inspect-giga-multilingual.py'),
                'model_sha256': base.EXPECTED['model.int8.onnx'][1]}
    try:
        import onnx
        model = onnx.load_model(str(args.assets / 'model.int8.onnx'), load_external_data=False)
        base.reject_external(model)
        # No checker repeat, shape inference, evaluator or graph operators.
        evidence['encoded_lengths'] = length_lineage(model.graph, onnx, base)
        initializers = {x.name: x for x in model.graph.initializer}
        producers = {out: x for x in model.graph.node for out in x.output}
        convs = []
        for node in model.graph.node:
            if node.op_type != 'ConvInteger' or 'pre_encode' not in node.name:
                continue
            row = base.node_summary(node, onnx)
            weight = node.input[1]
            tensor = initializers.get(weight)
            row['weight'] = {'name': weight,
                'dims': list(tensor.dims) if tensor is not None else None,
                'dtype': onnx.TensorProto.DataType.Name(tensor.data_type) if tensor is not None else None,
                'producer': base.node_summary(producers[weight], onnx) if weight in producers else None}
            convs.append(row)
        if not convs:
            raise ValueError('no actual pre_encode ConvInteger found')
        evidence.update(subsampling_convinteger=convs,
            functions=[{'name': x.name, 'domain': x.domain, 'nodes': len(x.node)} for x in model.functions],
            training_info_count=len(model.training_info),
            status='static supplement complete; formula/frontend/quality not qualified')
    except BaseException as error:
        evidence['failure'] = str(error)
        raise
    finally:
        evidence['seconds'] = time.monotonic() - started
        (args.output / 'supplement.json').write_text(json.dumps(evidence, ensure_ascii=False, indent=2, allow_nan=False))


if __name__ == '__main__':
    main()
