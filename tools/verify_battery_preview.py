"""Reject stale preview manifests, altered images, or images outside application flash."""
import hashlib
import json

from build_release import ROOT
from nocfree_uf2 import APP_BASE, APP_END, NRF52833_FAMILY_ID, UF2Image, validate_application


def main():
    directory = ROOT / "target/battery-preview"
    manifest = json.loads((directory / "manifest.json").read_text(encoding="utf-8"))
    for relative, digest in manifest["source_sha256"].items():
        assert hashlib.sha256((ROOT / relative).read_bytes()).hexdigest() == digest, f"Stale build: {relative}"
    expected = {(layout, role) for layout in ("ANSI", "ISO", "JIS", "KR") for role in ("Left", "Right")}
    expected.add(("Universal", "Dongle"))
    observed = [(entry["layout"], entry["role"]) for entry in manifest["images"]]
    assert len(observed) == len(expected) and set(observed) == expected, "Missing or duplicate hardware role"
    for entry in manifest["images"]:
        uf2 = directory / entry["file"]
        assert uf2.name == f'NocFree_Battery_Experimental_{entry["layout"]}_{entry["role"]}.uf2'
        assert hashlib.sha256(uf2.read_bytes()).hexdigest() == entry["sha256"], uf2
        application = uf2.with_suffix(".bin").read_bytes()
        validate_application(application)
        image = UF2Image.load(uf2)
        assert image.family_id == NRF52833_FAMILY_ID
        assert min(image.blocks) == APP_BASE and max(image.blocks) + 256 <= APP_END
        assert image.read(APP_BASE, len(application)) == application
    print("BATTERY_PREVIEW_ARTIFACTS_PASS: 9 current source-matched bounded images")


if __name__ == "__main__":
    main()
