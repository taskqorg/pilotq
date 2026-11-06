#!/usr/bin/env python3
# Invented model events; the MCP bridge and PilotQ dispatch are real.
import hashlib, json, os, subprocess, sys, time
if sys.argv[1:] == ["--version"]: print("2.1.0"); sys.exit(0)
args = sys.argv[1:]
value = lambda k: args[args.index(k) + 1]
sid = value('--resume') if '--resume' in args else value('--session-id')
native = '--permission-mode' in args and value('--permission-mode') == 'bypassPermissions'
assert native or value('--tools') == ''
assert not os.environ.get('ANTHROPIC_API_KEY') and not os.environ.get('ANTHROPIC_AUTH_TOKEN')
server = json.load(open(value('--mcp-config')))['mcpServers']['pilotq']
message = json.loads(sys.stdin.readline())
def send(v):
    print(json.dumps(v), flush=True)
with open(os.path.join(os.path.dirname(__file__), 'relay.jsonl'), 'a') as log:
    log.write(json.dumps({'part_types': [p['type'] for p in message['message']['content']], 'session': sid, 'native': native, 'relay_token_hash': hashlib.sha256(server['env']['PILOTQ_MCP_TOKEN'].encode()).hexdigest()}) + '\n')
for part in message['message']['content']:
    if part['type'] == 'image': assert part['source']['type'] == 'base64' and part['source']['data']
send({'type': 'system', 'subtype': 'init', 'session_id': sid})
send({'type': 'assistant', 'message': {'id': 'tool-message', 'model': 'claude-invented', 'content': [{'type': 'tool_use', 'id': 't1', 'name': 'mcp__pilotq__project_list', 'input': {}}]}})
bridge = subprocess.Popen([server['command']] + server['args'], env={**os.environ, **server['env']}, stdin=subprocess.PIPE, stdout=subprocess.PIPE, text=True)
requests = [{'id': 1, 'method': 'initialize'}, {'id': 2, 'method': 'tools/list'}, {'id': 3, 'method': 'tools/call', 'params': {'name': 'project_list', 'arguments': {}}}]
output, _ = bridge.communicate('\n'.join(json.dumps(v) for v in requests) + '\n', timeout=15)
replies = [json.loads(line) for line in output.splitlines()]
assert any(t['name'] == 'project_list' for t in replies[1]['result']['tools'])
assert not replies[2]['result']['isError']
send({'type': 'user', 'message': {'content': [{'type': 'tool_result', 'tool_use_id': 't1', 'content': 'granted result'}]}})
send({'type': 'stream_event', 'event': {'type': 'content_block_delta', 'delta': {'type': 'text_delta', 'text': 'preview'}}})
time.sleep(1.3)
send({'type':'assistant','message':{'id':'earlier','content':[{'type':'text','text':'earlier result'}]}})
send({'type': 'assistant', 'message': {'id': 'answer', 'model': 'claude-invented', 'content': [{'type': 'text', 'text': 'relay result'}], 'usage': {'input_tokens': 10, 'cache_read_input_tokens': 0, 'cache_creation_input_tokens': 0}}})
reset = int(time.time()) + 10000
send({'type': 'rate_limit_event', 'rate_limit_info': {'unifiedWindows': {'five_hour': {'utilization': .25, 'resetsAt': reset}, 'seven_day': {'utilization': .5, 'resetsAt': reset}}}})
send({'type': 'result', 'subtype': 'success', 'is_error': False, 'result': 'relay result', 'session_id': sid, 'modelUsage': {'claude-invented': {'contextWindow': 100}}})
