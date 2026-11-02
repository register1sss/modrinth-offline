#!/usr/bin/env python3
"""The prebuilt U3 Fixes engine shipped with the Windows installer (CI only).

Runs the pinned Horizon launcher's own `native-build` on the embedded U3 Fixes, as U3drinth runs
it on a player's computer, against a stand-in base version: a native build reads nothing from its
base but the prepared-resources marker, and copies the base's game files only when it installs.
Horizon names the build after its inputs (native_mods::version_for); U3drinth installs this engine
under that name before its own native-build, which then reuses it instead of compiling for 20-30
minutes. The name is checked here and again by the launcher's tests (U3_ENGINE_DIR).

  build.py --key    print the cache key: the build's version, the runtime pin and this recipe
  build.py OUT      build into OUT: horizon-client.exe and engine.json
"""
import hashlib
import io
import json
import os
import pathlib
import re
import shutil
import subprocess
import sys
import tempfile
import urllib.request
import zipfile

RECIPE = 1  # bump when this procedure changes what it produces
ROOT = pathlib.Path(__file__).resolve().parents[2]
SOURCES = ROOT / "packages/app-lib/src/minecraft_rust"
BASE = "u3-ci-base"


def rust_const(path, name):
    text = path.read_text(encoding="utf-8")
    found = re.search(rf'const {name}: &str =\s*"([^"]+)"', text)
    if not found:
        sys.exit(f"{path.name}: no {name}")
    return found.group(1)


def runtime(name):
    return rust_const(ROOT / "packages/app-lib/src/minecraft_rust.rs", name)


def key_input(name):
    return rust_const(SOURCES / "engine.rs", name)


def entries():
    """mods.rs FIXES: the embedded bundle's archive entries, in their order, with their bytes."""
    text = (SOURCES / "mods.rs").read_text(encoding="utf-8")
    block = text[text.index("pub(super) const FIXES:") : text.index("pub(super) const FIXES_VERSION")]
    found = re.findall(r'\(\s*"([^"]+)",\s*include_str!\("(u3_fixes/[^"]+)"\),?\s*\)', block)
    if not found or found[0][0] != "horizon-native.json":
        sys.exit("mods.rs: FIXES not found")
    return [(name, (SOURCES / path).read_bytes()) for name, path in found]


def expected(files):
    """Horizon's version_for over the bundle, as engine.rs computes it."""
    manifest = json.loads(files[0][1])
    digest = hashlib.sha256()
    for part in (
        manifest["mixins"],
        key_input("PROFILE"),
        key_input("FEATURES"),
        key_input("RUST"),
        manifest["horizon"],
    ):
        digest.update(part.encode() + b"\0")
    digest.update(manifest["id"].encode() + b"\0")
    prefix = f"mixins/{manifest['id']}/"
    for name, data in sorted((n[len(prefix) :], d) for n, d in files if n.startswith(prefix)):
        digest.update(name.encode())
        digest.update(len(data).to_bytes(8, "little"))
        digest.update(data)
    return manifest, f"native-{manifest['horizon'][:7]}-{digest.digest()[:6].hex()}"


def download(url, sha256):
    with urllib.request.urlopen(url, timeout=900) as response:
        data = response.read()
    if hashlib.sha256(data).hexdigest() != sha256:
        sys.exit(f"{url}: checksum mismatch")
    return data


def build(out):
    files = entries()
    manifest, version = expected(files)
    print(f"Building {version} (U3 Fixes {manifest['version']} on Horizon {manifest['horizon'][:7]})")
    work = pathlib.Path(os.environ.get("RUNNER_TEMP") or tempfile.gettempdir()) / "u3-engine"
    shutil.rmtree(work, ignore_errors=True)
    launcher, game = work / "launcher", work / "game"
    archive = download(runtime("ARCHIVE_URL"), runtime("ARCHIVE_HASH"))
    exe = zipfile.ZipFile(io.BytesIO(archive)).read("horizon-launcher.exe")
    if hashlib.sha256(exe).hexdigest() != runtime("EXE_HASH"):
        sys.exit("horizon-launcher.exe: checksum mismatch")
    program = work / "runtime" / "horizon-launcher.exe"
    program.parent.mkdir(parents=True)
    program.write_bytes(exe)
    # The stand-in base: installed and prepared for the bundle's Minecraft version, no files.
    base = launcher / "versions" / BASE
    resources = base / ".local" / "horizon-client" / manifest["game"]
    resources.mkdir(parents=True)
    (resources / "horizon-resources.json").write_text("{}")
    (base / "research-lock.json").write_text(json.dumps({"minecraft": {"version": manifest["game"]}}))
    (base / ".installed").write_text("")
    # The bundle installed and on in Horizon's own home, as the launcher installs it.
    mods = game / "horizon-native-mods"
    mods.mkdir(parents=True)
    with zipfile.ZipFile(mods / f"{manifest['id']}.hnmod", "w", zipfile.ZIP_STORED) as bundle:
        for name, data in files:
            info = zipfile.ZipInfo(name, (1980, 1, 1, 0, 0, 0))
            info.create_system = 3
            info.external_attr = 0o100644 << 16
            bundle.writestr(info, data)
    (game / "horizon-mods.txt").write_text(f"mod.{manifest['id']}:true\n")
    env = dict(os.environ, HORIZON_LAUNCHER_HOME=str(launcher), HORIZON_HOME=str(game))
    for name in ("HORIZON_MSA_CLIENT_ID", "HORIZON_MSA_MICROSOFT_ENDPOINT", "HORIZON_LAUNCHER_PAGE", "HORIZON_MOD_INDEX", "HORIZON_MOD_RATINGS", "CARGO_BUILD_JOBS", "RUSTFLAGS"):
        env.pop(name, None)
    command = [str(program), "native-build", "--instance", "horizon", "--version", BASE, "--trust-native-code"]
    if subprocess.run(command, env=env, cwd=work, stdin=subprocess.DEVNULL).returncode != 0:
        log = launcher / "native" / "build.log"
        if log.is_file():
            print("".join(log.read_text(encoding="utf-8", errors="replace").splitlines(True)[-200:]))
        sys.exit("Horizon's native-build failed")
    built = launcher / "versions" / version
    if not (built / ".installed").is_file():
        found = sorted(p.name for p in (launcher / "versions").glob("native-*"))
        sys.exit(f"Horizon built {found}, not {version}: engine.rs no longer matches its version_for")
    about = json.loads((built / "native-mods.json").read_text(encoding="utf-8"))
    mods_built = [{"id": manifest["id"], "version": manifest["version"]}]
    if (
        about.get("horizon") != manifest["horizon"]
        or about.get("horizon_mixins") != manifest["mixins"]
        or about.get("rust") != key_input("RUST")
        or about.get("mods") != mods_built
    ):
        sys.exit(f"Unexpected build: {about}")
    engine = (built / "horizon-client.exe").read_bytes()
    if engine[:2] != b"MZ":
        sys.exit("The built engine is not a Windows program")
    out.mkdir(parents=True, exist_ok=True)
    (out / "horizon-client.exe").write_bytes(engine)
    started = subprocess.run(
        [str(out / "horizon-client.exe"), "--help"], capture_output=True, timeout=300, stdin=subprocess.DEVNULL
    )
    if b"--resources" not in started.stdout + started.stderr:
        sys.exit(f"The built engine does not start: {started.returncode} {started.stderr[-2000:]!r}")
    facts = {
        "version": version,
        "sha256": hashlib.sha256(engine).hexdigest(),
        "size": len(engine),
        "horizon": manifest["horizon"],
        "horizon_mixins": manifest["mixins"],
        "rust": key_input("RUST"),
        "mods": mods_built,
        "runtime": runtime("RELEASE"),
        "recipe": RECIPE,
    }
    (out / "engine.json").write_text(json.dumps(facts, indent=2) + "\n")
    print(json.dumps(facts, indent=2))


def main():
    if sys.argv[1:] == ["--key"]:
        _, version = expected(entries())
        print(f"{version}-{runtime('RELEASE').replace('@', '-')}-r{RECIPE}")
    elif len(sys.argv) == 2:
        build(pathlib.Path(sys.argv[1]).resolve())
    else:
        sys.exit(__doc__)


if __name__ == "__main__":
    main()
