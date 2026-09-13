"""Build checked experimental battery images without overwriting release firmware."""
from __future__ import annotations

import hashlib
import json
import subprocess
import sys
from pathlib import Path

from build_release import ARM_TARGET, LAYOUTS, ROOT, llvm_objcopy, run, rust_host
from nocfree_uf2 import pack_application


def main() -> None:
    host = rust_host()
    objcopy = llvm_objcopy()
    destination = ROOT / "target/battery-preview"
    destination.mkdir(parents=True, exist_ok=True)
    manifest = destination / "manifest.json"
    # A failed rebuild must not leave an old success manifest for new partial artifacts.
    manifest.unlink(missing_ok=True)
    images = []
    run("cargo", "fmt", "--package", "nocfree-and-rust", "--", "--check")
    for layout in LAYOUTS:
        features = ["--no-default-features", "--features", f"layout-{layout.lower()},backlight-perceptual"]
        run("cargo", "test", "--target", host, "--lib", *features)
        run("cargo", "clippy", "--target", host, "--lib", *features, "--", "-D", "warnings")
        roles = [("central", "Left"), ("right", "Right")]
        if layout == "JIS":
            roles.append(("dongle", "Dongle"))
        for binary, role in roles:
            run("cargo", "clippy", "--release", "--bin", binary, *features, "--", "-D", "warnings")
            run("cargo", "build", "--release", "--bin", binary, *features)
            label = "Universal" if role == "Dongle" else layout
            stem = f"NocFree_Battery_Experimental_{label}_{role}"
            raw = destination / f"{stem}.bin"
            uf2 = destination / f"{stem}.uf2"
            run(str(objcopy), "-O", "binary", str(ROOT / "target" / ARM_TARGET / "release" / binary), str(raw))
            pack_application(raw, uf2)
            images.append({"layout": label, "role": role, "file": uf2.name,
                           "sha256": hashlib.sha256(uf2.read_bytes()).hexdigest(),
                           "application_bytes": raw.stat().st_size})
    run(sys.executable, "-B", "-m", "unittest", "discover", "-s", "tools", "-p", "test_*.py")
    # Include the complete source inputs, so an uncommitted preview is still identifiable.
    sources = {}
    for path in sorted([*ROOT.glob("src/**/*.rs"), ROOT / "Cargo.toml", ROOT / "Cargo.lock",
                        ROOT / "build.rs", ROOT / "memory.x", ROOT / ".cargo/config.toml"]):
        sources[path.relative_to(ROOT).as_posix()] = hashlib.sha256(path.read_bytes()).hexdigest()
    manifest.write_text(json.dumps({
        "status": "experimental; hardware qualification pending; not flashed",
        "base_commit": subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip(),
        "source_sha256": sources, "images": images,
    }, indent=2) + "\n", encoding="utf-8")
    print(f"BATTERY_PREVIEW_CHECKS_PASS: {len(images)} bounded UF2 images; {manifest}")


if __name__ == "__main__":
    main()
