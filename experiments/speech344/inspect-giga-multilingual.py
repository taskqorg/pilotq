#!/usr/bin/env python3
"""Static ONNX inspection only. Run reviewed private tooling under PM isolation."""
import argparse
from collections import Counter
import hashlib
import importlib.metadata
import importlib.util
import json
import os
import signal
from pathlib import Path
import time

REPO = Path(__file__).resolve().parents[2]
EXPECTED = {
    'model.int8.onnx': (224762524, '2d94f93ffd4ef58e7899c9de885c25bbbc8c9f1073618868d118a674450ba5f7'),
    'tokens.txt': (391, '9b5df7987cb4ca52c1a468649ce897fab1cd182067416e29fef49dfaa7a856c2'),
    'multilingual_ctc.yaml': (1155, '171bf1d9a678d7085ea336079d3f4ce5cf7cd048e099e4ae9b9c1445d26fc247'),
    'export-onnx-ctc-multilingual.py': (3818, '50c90e200395fad38a6123fd89a79830322991593b43b91f4f6c0f3918944d1b'),
    'LICENSE': (1070, 'f00de6715714c7a63d08639cdbfaa40224eefc407302614bd19f1a8b98c875aa'),
    'README.md': (4170, 'f01c7a763c528ba9caa4e995bc26b5fe01b68adb7c4c5b3e681058b8522db907'),
}
VERSIONS = {'onnx': '1.20.1', 'numpy': '2.4.2', 'protobuf': '6.33.5',
            'ml_dtypes': '0.5.4', 'typing_extensions': '4.15.0'}


def sha(path):
    with path.open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest()


def reject_external(message, path='model'):
    # Standard protobuf reflection traverses graphs, subgraphs, sparse tensors,
    # attributes/functions/training graphs, without a handwritten wire parser.
    if message.DESCRIPTOR.full_name == 'onnx.TensorProto':
        if message.external_data or message.data_location == 1:
            raise ValueError('external tensor data refused: ' + path)
    for field, value in message.ListFields():
        if field.type != field.TYPE_MESSAGE:
            continue
        if field.is_repeated:
            for index, item in enumerate(value):
                reject_external(item, f'{path}.{field.name}[{index}]')
        else:
            reject_external(value, path + '.' + field.name)


def describe_value(value, onnx):
    tensor = value.type.tensor_type
    if not value.type.HasField('tensor_type'):
        return {'name': value.name, 'type': str(value.type)}
    return {'name': value.name, 'dtype': onnx.TensorProto.DataType.Name(tensor.elem_type),
            'shape': [d.dim_value if d.HasField('dim_value') else d.dim_param or None
                      for d in tensor.shape.dim]}


def node_summary(node, onnx):
    attrs = {}
    for attr in node.attribute:
        # Report structural integer/string attributes, never weight payloads.
        if attr.type == onnx.AttributeProto.INT:
            attrs[attr.name] = attr.i
        elif attr.type == onnx.AttributeProto.INTS:
            attrs[attr.name] = list(attr.ints)
        elif attr.type == onnx.AttributeProto.STRING:
            attrs[attr.name] = attr.s.decode('utf-8', errors='strict')
        elif attr.type == onnx.AttributeProto.TENSOR:
            tensor = attr.t
            count = 1
            for dim in tensor.dims:
                count *= dim
            if count <= 16 and tensor.data_type in (onnx.TensorProto.INT32, onnx.TensorProto.INT64):
                attrs[attr.name] = onnx.numpy_helper.to_array(tensor).tolist()
    return {'name': node.name, 'domain': node.domain, 'op': node.op_type,
            'inputs': list(node.input), 'outputs': list(node.output), 'structural_attributes': attrs}


def output_lineage(graph, output, onnx):
    producers = {name: node for node in graph.node for name in node.output}
    initializers = {tensor.name: tensor for tensor in graph.initializer}
    pending = [output]; seen = set(); nodes = []; leaves = []
    while pending and len(seen) < 256:
        name = pending.pop()
        if not name or name in seen:
            continue
        seen.add(name)
        node = producers.get(name)
        if node is None:
            tensor = initializers.get(name)
            leaf = {'name': name, 'kind': 'initializer' if tensor is not None else 'graph input/unresolved'}
            if tensor is not None and tensor.data_type in (onnx.TensorProto.INT32, onnx.TensorProto.INT64):
                count = 1
                for dim in tensor.dims:
                    count *= dim
                if count <= 16:
                    leaf['integer_value'] = onnx.numpy_helper.to_array(tensor).tolist()
            leaves.append(leaf)
        else:
            nodes.append(node_summary(node, onnx)); pending.extend(node.input)
    return {'output': output, 'nodes': nodes, 'leaves': leaves, 'truncated': bool(pending),
            'formula_status': 'static producer evidence only; manually verify padding/rounding and returned valid length, no evaluated formula'}


def integer_expression(graph, output, onnx):
    """Render supported integer producer operations symbolically, never evaluate graph."""
    producers = {name: node for node in graph.node for name in node.output}
    initializers = {tensor.name: tensor for tensor in graph.initializer}
    allowed = {'Constant', 'Identity', 'Cast', 'Add', 'Sub', 'Mul', 'Div', 'Floor',
               'Ceil', 'Gather', 'Shape', 'Squeeze', 'Unsqueeze', 'Concat', 'Slice'}
    budget = [256]
    def expression(name, depth=0):
        budget[0] -= 1
        if depth >= 32 or budget[0] < 0:
            return {'unknown': name, 'reason': 'symbolic depth/node cap'}
        tensor = initializers.get(name)
        if tensor is not None:
            count = 1
            for dim in tensor.dims:
                count *= dim
            if tensor.data_type in (onnx.TensorProto.INT32, onnx.TensorProto.INT64) and count <= 16:
                return {'integer_constant': onnx.numpy_helper.to_array(tensor).tolist()}
            return {'tensor': name, 'reason': 'not a small integer constant'}
        node = producers.get(name)
        if node is None:
            return {'input_or_unresolved': name}
        if node.domain not in ('', 'ai.onnx') or node.op_type not in allowed:
            return {'unknown': name, 'producer': node.op_type, 'reason': 'unsupported symbolic operation'}
        row = node_summary(node, onnx)
        return {'op': row['op'], 'attributes': row['structural_attributes'],
                'inputs': [expression(x, depth + 1) for x in node.input]}
    return expression(output)


def selftest(onnx):
    # Failure protection: external data may hide below attributes/sparse tensors.
    from onnx import helper, TensorProto
    tensor = helper.make_tensor('external', TensorProto.FLOAT, [1], [0.0])
    tensor.data_location = TensorProto.EXTERNAL
    entry = tensor.external_data.add(); entry.key = 'location'; entry.value = 'invented-forbidden.bin'
    graphs = [helper.make_graph([], 'top', [], [], [tensor]),
              helper.make_graph([helper.make_node('Constant', [], ['x'], value=tensor)], 'attribute', [], [])]
    nested = helper.make_graph([helper.make_node('If', ['condition'], ['y'], then_branch=graphs[0], else_branch=graphs[0])], 'nested', [], [])
    graphs.append(nested)
    sparse = onnx.SparseTensorProto(); sparse.values.CopyFrom(tensor)
    graph = helper.make_graph([], 'sparse', [], []); graph.sparse_initializer.append(sparse); graphs.append(graph)
    for graph in graphs:
        try:
            reject_external(helper.make_model(graph))
        except ValueError:
            pass
        else:
            raise AssertionError('external data accepted')
    clean = helper.make_model(helper.make_graph([], 'clean', [], []))
    reject_external(clean)
    print('PASS: top/attribute/nested/sparse external data rejected; embedded graph accepted; no graph execution')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--assets', type=Path)
    parser.add_argument('--output', type=Path)
    parser.add_argument('--selftest', action='store_true')
    args = parser.parse_args()
    spec = importlib.util.spec_from_file_location('guard344', REPO / 'scripts/probe-speech-344.py')
    guard = importlib.util.module_from_spec(spec); spec.loader.exec_module(guard)
    isolation = guard.limits()  # Actual effective caps before importing parser or loading graph.
    started = time.monotonic(); deadline = started + 600
    def bounded():
        if time.monotonic() >= deadline:
            raise TimeoutError('static inspection shared600s')
    for package, version in VERSIONS.items():
        if importlib.metadata.version(package) != version:
            raise ValueError('private inspector version mismatch: ' + package)
    if not args.selftest:
        if args.assets is None or args.output is None:
            parser.error('--assets and --output required')
        for name, (size, digest) in EXPECTED.items():
            path = args.assets / name
            if path.stat().st_size != size or sha(path) != digest:
                raise ValueError('asset identity mismatch: ' + name)
            bounded()
    for variable in ('OMP_NUM_THREADS', 'OPENBLAS_NUM_THREADS', 'MKL_NUM_THREADS', 'NUMEXPR_NUM_THREADS'):
        os.environ[variable] = '4'  # Private inspector process only; no host setting changes.
    import onnx
    bounded()
    selftest(onnx)  # Prepared guard runs before candidate parse under the same deadline.
    bounded()
    if args.selftest:
        return
    os.umask(0o077)
    args.output.mkdir(mode=0o700, parents=True, exist_ok=False)
    evidence = {'status': 'incomplete', 'isolation': isolation, 'versions': VERSIONS,
                'model_executed': False, 'script_sha256': sha(Path(__file__)), 'assets': EXPECTED}
    try:
        model = onnx.load_model(str(args.assets / 'model.int8.onnx'), load_external_data=False)
        bounded(); reject_external(model); bounded()
        # Static schema/graph checker only; no ReferenceEvaluator/Runtime or shape execution.
        onnx.checker.check_model(model, full_check=False)
        bounded()
        evidence.update({'ir_version': model.ir_version,
                         'producer': {'name': model.producer_name, 'version': model.producer_version},
                         'opsets': [{'domain': x.domain, 'version': x.version} for x in model.opset_import],
                         'metadata': [{'key': x.key, 'value': x.value} for x in model.metadata_props],
                         'inputs': [describe_value(x, onnx) for x in model.graph.input],
                         'outputs': [describe_value(x, onnx) for x in model.graph.output],
                         'operators': dict(Counter((x.domain + ':' + x.op_type) for x in model.graph.node)),
                         'convolutions': [node_summary(x, onnx) for x in model.graph.node if x.op_type == 'Conv'],
                         'normalization_nodes': [node_summary(x, onnx) for x in model.graph.node if 'Norm' in x.op_type],
                         'normalization_status': 'metadata/operator evidence only; no frontend numerical parity proof',
                         'output_lineage': [output_lineage(model.graph, x.name, onnx) for x in model.graph.output],
                         'integer_output_expressions': {x.name: integer_expression(model.graph, x.name, onnx)
                             for x in model.graph.output if x.type.tensor_type.elem_type in (onnx.TensorProto.INT32, onnx.TensorProto.INT64)},
                         'official_checkpoint_equivalence': 'unknown', 'frontend_reference_parity': 'unknown',
                         'host_cleanup': 'independent PM runner readback required'})
        bounded()
        evidence['status'] = 'static inspection completed; no compatibility/quality acceptance'
    except BaseException as error:
        evidence['failure'] = str(error)
        raise
    finally:
        evidence['seconds'] = time.monotonic() - started
        (args.output / 'inspection.json').write_text(json.dumps(evidence, ensure_ascii=False, indent=2))


if __name__ == '__main__':
    def terminate(signum, frame):
        raise SystemExit('inspection terminated; PM must verify unit cleanup')
    signal.signal(signal.SIGTERM, terminate)
    main()
