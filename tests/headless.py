"""An isolated, persistent native host for undo and failure/rollback tests.

Unlike the live app connector, this starts its own empty headless SolidCraft process.
"""
import json
import os
import selectors
import subprocess
import time

from harness import CLI, ROOT, TMP


class Headless:
    def __init__(self, *, plugins_dir=None):
        self.plugins_dir = plugins_dir

    def __enter__(self):
        self.log = open(os.path.join(TMP, 'headless.log'), 'w+')
        cmd = [CLI]
        if self.plugins_dir is not None:
            cmd.extend(['--plugins-dir', self.plugins_dir])
        cmd.extend(['mcp', '--headless', '--allow-path', ROOT, '--allow-path', TMP])
        self.process = subprocess.Popen(cmd,
                                        stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=self.log, text=True, bufsize=1)
        self.selector = selectors.DefaultSelector()
        self.selector.register(self.process.stdout, selectors.EVENT_READ)
        self.next_id = 0
        self.rpc('initialize', {'protocolVersion':'2024-11-05','capabilities':{},
                               'clientInfo':{'name':'parts-library-tests','version':'1'}})
        self.process.stdin.write(json.dumps({'jsonrpc':'2.0','method':'notifications/initialized'})+'\n')
        self.process.stdin.flush()
        return self

    def __exit__(self, *_):
        self.process.stdin.close()
        try:
            self.process.wait(timeout=5)
        except subprocess.TimeoutExpired:
            self.process.terminate()
            self.process.wait(timeout=5)
        self.process.stdout.close()
        self.selector.close()
        self.log.close()

    def rpc(self, method, params):
        self.next_id += 1
        request_id = self.next_id
        self.process.stdin.write(json.dumps({'jsonrpc':'2.0','id':request_id,'method':method,'params':params})+'\n')
        self.process.stdin.flush()
        deadline = time.monotonic()+50
        while time.monotonic() < deadline:
            if not self.selector.select(max(0,deadline-time.monotonic())):
                break
            line = self.process.stdout.readline()
            if not line:
                self.log.flush()
                self.log.seek(0)
                raise RuntimeError('headless host exited: '+self.log.read()[-1500:])
            reply = json.loads(line)
            if reply.get('id') == request_id:
                if 'error' in reply:
                    raise RuntimeError(str(reply['error']))
                return reply['result']
        raise TimeoutError('headless host did not reply within 50 seconds')

    def command(self, command, allow_error=False, **params):
        return self.tool('execute', {'command':command,'params':params}, allow_error=allow_error)

    def tool(self, name, arguments, allow_error=False):
        reply = self.rpc('tools/call', {'name':name,'arguments':arguments})
        texts = [b['text'] for b in reply.get('content',[]) if b.get('type') == 'text']
        text = '\n'.join(texts)
        if reply.get('isError'):
            if allow_error:
                return {'error':text}
            raise RuntimeError(f'{arguments.get("command", name)}: {text}')
        return json.loads(text)
