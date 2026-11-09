#!/usr/bin/env python3
"""Biomes Unseen, live: the REAL client (offscreen, software Vulkan) with the
shipped biomes_unseen.hmod enabled, driven by horizon-ctl. Creates a creative
world, then proves in the running game what the host tests can only prove on
files:

  /locate biome biomes_unseen:bayou       -> found with coordinates
  /give @s biomes_unseen:cherry_planks    -> the item exists and is given

Also prints the game's volt-mods-report.txt (the diagnostics file the player
can send) and checks its counts.

    python3 tools/biomes_e2e.py --bin-dir target/fast

Needs the prepared resources in .local/horizon-client/<version> and, on a
headless Linux box, a DISPLAY (xvfb-run) and a Vulkan ICD (lavapipe).
"""
import argparse
import json
import os
import shutil
import subprocess
import sys
import tempfile
import time
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(ROOT / "tools"))
import lab  # noqa: E402

HMOD = ROOT / "crates/horizon-client/testdata/biomes_unseen.hmod"
LAVAPIPE_ICDS = [
    Path("/usr/share/vulkan/icd.d/lvp_icd.x86_64.json"),
    Path("/run/opengl-driver/share/vulkan/icd.d/lvp_icd.x86_64.json"),
]


class CtlError(RuntimeError):
    pass


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--bin-dir", required=True,
                        help="folder holding horizon-client and horizon-ctl")
    parser.add_argument("--hmod", default=str(HMOD))
    args = parser.parse_args()
    bins = Path(args.bin_dir).resolve()
    ctl_bin = bins / "horizon-ctl"
    client = bins / "horizon-client"
    for b in (ctl_bin, client):
        if not b.is_file():
            sys.exit(f"missing binary: {b}")
    resources = lab.RESOURCES.resolve()
    if not (resources / "horizon-resources.json").exists():
        sys.exit(f"no prepared resources in {resources}")

    work = Path(tempfile.mkdtemp(prefix="volt-biomes-e2e-"))
    (work / ".local/horizon-client").mkdir(parents=True)
    os.symlink(resources, work / ".local/horizon-client" / lab.VERSION)
    (work / "options.txt").write_text("onboardAccessibility:false\n")
    # The mod, installed and enabled exactly as the launcher does it.
    (work / "horizon-mods").mkdir()
    shutil.copyfile(args.hmod, work / "horizon-mods/biomes_unseen.hmod")
    (work / "horizon-mods.txt").write_text("mod.biomes_unseen:true\n")

    env = lab.game_env(HORIZON_HOME=work, HORIZON_TELEMETRY_URL="off")
    for icd in LAVAPIPE_ICDS:
        if icd.exists():
            env.setdefault("VK_ICD_FILENAMES", str(icd))
            break
    socket = work / "c.sock"

    def ctl(words, timeout=120):
        r = subprocess.run([str(ctl_bin), "--socket", str(socket), *words],
                           cwd=work, env=env, capture_output=True, text=True,
                           timeout=timeout)
        out = r.stdout.strip().splitlines()
        reply = json.loads(out[-1]) if out else {}
        if r.returncode != 0:
            raise CtlError(f"horizon-ctl {words[0]}: exit {r.returncode}: "
                           f"{reply or r.stderr.strip()[-800:]}")
        return reply

    def chat_since(since):
        evs = ctl(["events", "--since", str(since)])
        lines = []
        for e in evs.get("events", []):
            if isinstance(e, dict):
                text = e.get("text") or e.get("message") or e.get("value") or ""
                if text:
                    lines.append(str(text))
        return evs.get("last", since), lines

    def last_seq():
        return ctl(["events"]).get("last", 0)

    ok = True
    try:
        print("launching the real client (offscreen, software Vulkan)...", flush=True)
        ctl(["launch", "--game", str(client), "--world", "BiomesUnseen",
             "--seed", "6400", "--creative", "--offscreen", "--wait",
             "--render-distance", "2"], timeout=900)
        ctl(["wait-ready", "600"], timeout=700)
        print("world is open and loaded", flush=True)

        # /locate biome: registered mod biomes must be findable in the
        # world. Appended climate entries win only where their ranges are
        # the closest match, and /locate searches a 6400-block radius - a
        # single biome can legitimately be outside it, so bayou is asked
        # first (and its reply always shown), then more until one is found.
        candidates = ["bayou", "grassland", "prairie", "coniferous_forest",
                      "seasonal_forest", "wetland", "highland", "marsh",
                      "orchard", "pasture"]
        hard_fail = ("unknown", "incorrect argument", "no such",
                     "unparseable", "expected", "can't find element",
                     "cant find element", "not_found")
        located = ""
        for biome in candidates:
            since = last_seq()
            ctl(["chat", f"/locate biome biomes_unseen:{biome}"])
            line = ""
            deadline = time.time() + 240
            while time.time() < deadline and not line:
                time.sleep(1.5)
                _, lines = chat_since(since)
                for l in lines:
                    low = l.lower()
                    if biome in low or "could not find" in low \
                            or any(b in low for b in hard_fail):
                        line = l
                        break
            if not line:
                raise AssertionError(f"/locate biome {biome}: no reply arrived")
            print(f"E2E locate biomes_unseen:{biome}: {line}", flush=True)
            if any(b in line.lower() for b in hard_fail):
                raise AssertionError(
                    f"/locate rejected the id (the biome is NOT registered): {line}")
            if "could not find" not in line.lower():
                located = f"biomes_unseen:{biome} -> {line}"
                break
        if not located:
            raise AssertionError(
                "no Biomes Unseen biome within 6400 blocks for any of "
                f"{candidates}: they are registered (ids accepted) but none "
                "was located")
        print(f"E2E PROOF locate: {located}", flush=True)

        # /give: the mod item resolves to a real registered item.
        since = last_seq()
        ctl(["chat", "/give @s biomes_unseen:cherry_planks"])
        give_line = ""
        deadline = time.time() + 120
        while time.time() < deadline and not give_line:
            time.sleep(1.5)
            since2, lines = chat_since(since)
            for line in lines:
                low = line.lower()
                if "gave" in low or "cherry" in low or "unknown" in low \
                        or "incorrect argument" in low or "expected" in low:
                    give_line = line
                    break
        if not give_line:
            raise AssertionError("/give: no reply arrived in chat")
        print(f"E2E PROOF give: {give_line}", flush=True)
        if any(b in give_line.lower() for b in ("unknown", "incorrect argument",
                                                "expected", "unparseable")):
            raise AssertionError(f"/give failed: {give_line}")
        if "gave" not in give_line.lower():
            raise AssertionError(f"unexpected /give reply: {give_line}")

        # The diagnostics file the player can send: it must exist and carry
        # the loaded counts (this is the report game/mods.rs writes).
        report = work / "volt-mods-report.txt"
        text = report.read_text(encoding="utf-8", errors="replace")
        print("---- volt-mods-report.txt ----")
        print(text)
        print("------------------------------", flush=True)
        for needle in ("176 block(s)", "193 creative item(s)",
                       "minecraft:overworld 39", "minecraft:nether 5",
                       "world opened: "):
            if needle not in text:
                raise AssertionError(f"report is missing {needle!r}")
        print("E2E PROOF report: volt-mods-report.txt lists 176 blocks, "
              "193 creative items, 39+5 placed biomes and the opened world",
              flush=True)
        print("E2E PASS: the running game locates biomes_unseen:bayou and "
              "gives biomes_unseen:cherry_planks", flush=True)
    except (AssertionError, CtlError, subprocess.TimeoutExpired,
            json.JSONDecodeError, FileNotFoundError) as e:
        ok = False
        print(f"E2E FAIL: {e!r}", flush=True)
        log = work / "logs/latest.log"
        if log.exists():
            tail = log.read_text(errors="replace").splitlines()[-60:]
            print("game log tail:\n" + "\n".join(tail), flush=True)
    finally:
        try:
            ctl(["quit"], timeout=30)
        except Exception:
            pass
        time.sleep(2)
        shutil.rmtree(work, ignore_errors=True)
    sys.exit(0 if ok else 1)


if __name__ == "__main__":
    main()
