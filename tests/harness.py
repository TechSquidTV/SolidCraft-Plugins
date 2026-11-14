"""What the CLI tests share: running a script through solidcraft-cli and reporting checks."""
import atexit, json, os, shutil, signal, subprocess, sys, tempfile

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
CLI = os.environ.get("SOLIDCRAFT_CLI", "solidcraft-cli")
TMP = tempfile.mkdtemp(prefix="threadcraft-test-")
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


def package(plugin_id):
    return os.path.join(ROOT, "dist", plugin_id)


def inspect(plugin_id):
    """plugin.inspect of a built package: (exit code, output text, its reply or None)."""
    code, text, results = run([{"command": "plugin.inspect", "params": {"path": package(plugin_id)}}])
    return code, text, results[0] if results else None
