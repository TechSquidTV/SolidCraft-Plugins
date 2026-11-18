"""What the CLI tests share: running a script through solidcraft-cli and reporting checks."""
import atexit, json, os, shutil, signal, subprocess, sys, tempfile
from copy import deepcopy

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
CLI = os.environ.get("SOLIDCRAFT_CLI", "solidcraft-cli")
TMP = tempfile.mkdtemp(prefix="techsquid-plugins-test-")
# The temporary folder goes on every way out: a normal end, sys.exit, or an exception.
atexit.register(shutil.rmtree, TMP, ignore_errors=True)


def run(steps, save=None, inp=None):
    """Run the steps. Returns (exit code, output text, the steps' results or None)."""
    script = os.path.join(TMP, "s.json")
    out = os.path.join(TMP, "o.json")
    if os.path.exists(out):
        os.remove(out)
    with open(script, "w") as f:
        json.dump(steps, f)
    cmd = [CLI, "run", script, "--results", "--quiet", "--json-out", out]
    if inp:
        cmd += ["--in", inp]
    if save:
        cmd += ["--save", save]
    p = subprocess.run(cmd, capture_output=True, text=True)
    text = p.stdout + p.stderr
    if p.returncode != 0 or not os.path.exists(out):
        return p.returncode, text, None
    with open(out) as f:
        results = [r.get("result", r) if isinstance(r, dict) else r for r in json.load(f)]
    return 0, text, results


def killed(code):
    """Whether the CLI was killed with SIGKILL, which is what the signed macOS build of
    SolidCraft 0.41.0 does when it loads a component (hardened runtime, no JIT
    entitlement). -9 is the signal as subprocess reports it, 137 is 128 + 9 as a shell
    reports it. Any other signal, such as an abort or a segfault, is a real failure."""
    return code == -signal.SIGKILL or code == 128 + signal.SIGKILL


def last_line(text):
    """The last line of the CLI's output, or a note that it printed nothing."""
    lines = text.strip().splitlines()
    return lines[-1] if lines else "(solidcraft-cli printed nothing)"


class Tally:
    passed = 0
    failed = 0
    skipped = 0


def check(name, ok, detail=""):
    print(("PASS " if ok else "FAIL ") + name + ((" " + detail) if detail else ""))
    if ok:
        Tally.passed += 1
    else:
        Tally.failed += 1
        check.failed = True


check.failed = False


def skip(name, reason):
    print(f"SKIP {name}: {reason}")
    Tally.skipped += 1


def finish():
    """Print the counts and exit: 1 when any check failed, 0 otherwise. A run where
    every case was skipped says so, because it measured nothing."""
    print(f"{Tally.passed} passed, {Tally.failed} failed, {Tally.skipped} skipped")
    if Tally.skipped and not Tally.passed and not Tally.failed:
        print("SKIP: this solidcraft-cli cannot run a plugin, so nothing was checked")
    elif Tally.skipped:
        print("NOTE: some cases were skipped because solidcraft-cli was killed")
    sys.exit(1 if Tally.failed else 0)


def plugin_ids():
    """Every plugin in the repository: each folder under plugins/ with a plugin.json.
    The folder name is the plugin id and the package lands in dist/<id>/."""
    plugins = os.path.join(ROOT, "plugins")
    return sorted(d for d in os.listdir(plugins)
                  if os.path.isfile(os.path.join(plugins, d, "plugin.json")))


def picks(sketch, ids, curve=False):
    """The Placement field's value, as SolidCraft's pick field sends it: one reference per
    id in `ids` (a list, or text with commas). `sketch` is the sketch's feature id, or its
    name as a program may give it. `curve` picks circles or lines instead of points."""
    if isinstance(ids, str):
        ids = [i.strip() for i in ids.split(",") if i.strip()]
    kind = "external_sketch_curve" if curve else "external_sketch_point"
    return [{"type": kind, "sketch": sketch, "id": i} for i in ids]


def package(plugin_id):
    return os.path.join(ROOT, "dist", plugin_id)


def inspect(plugin_id):
    """plugin.inspect of a built package: (exit code, output text, its reply or None)."""
    code, text, results = run([{"command": "plugin.inspect", "params": {"path": package(plugin_id)}}])
    return code, text, results[0] if results else None


_approvals = {}


def recipe_commands(recipe):
    """Resolve repository package paths and approvals in an example's steps."""
    steps = []
    for step in recipe["steps"]:
        params = deepcopy(step["params"])
        if step["command"] == "plugin.run":
            plugin_id = os.path.basename(params["path"])
            if plugin_id not in _approvals:
                code, text, info = inspect(plugin_id)
                if code != 0 or not info:
                    sys.exit(f"plugin.inspect of dist/{plugin_id} failed, run ./build.sh first: {text[-500:]}")
                _approvals[plugin_id] = info["approval"]
            params["path"] = package(plugin_id)
            params["approval"] = _approvals[plugin_id]
        steps.append({"command": step["command"], "params": params})
    return steps
