from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path
import struct
import zipfile

from nocfree_uf2 import APP_BASE, APP_END, NRF52833_FAMILY_ID, UF2_PAYLOAD_SIZE, UF2Image


ROOT = Path(__file__).resolve().parents[1]
DEFAULT_IMAGE = ROOT.parent / "NocFree_and_V2.3.0_Dongle.uf2"
DEFAULT_PACKAGE = ROOT.parent / "official_v2_3_1" / "dongle.zip"
EXPECTED_SHA256 = "98097252b4752888d4cd959d98b019152b95e743d1af4039db841d670002a93c"
EXPECTED_PACKAGE_SHA256 = "02c1ee2bb420e374e51ac6b0c0ee7a422796dfdff0ac2707ca28096a564c0567"
EXPECTED_BINARY_SHA256 = "5e3ad6ce64f41db164a83a5fab88c28f7c92e0edfbc7dcf99b9d23ec3c9010cd"
EXPECTED_SIZE = 143_872
EXPECTED_END = 0x38900


def verify(path: Path, package_path: Path) -> None:
    raw = path.read_bytes()
    digest = hashlib.sha256(raw).hexdigest()
    if digest != EXPECTED_SHA256:
        raise ValueError(f"Unexpected SHA-256: {digest}")
    if len(raw) != EXPECTED_SIZE:
        raise ValueError(f"Unexpected UF2 size: {len(raw)}")

    image = UF2Image.load(path)
    addresses = sorted(image.blocks)
    expected_addresses = list(range(APP_BASE, EXPECTED_END, UF2_PAYLOAD_SIZE))
    if image.family_id != NRF52833_FAMILY_ID:
        raise ValueError(f"Unexpected UF2 family: 0x{image.family_id:08X}")
    if addresses != expected_addresses:
        raise ValueError("Stock UF2 application blocks are missing or outside the expected range")
    if EXPECTED_END > APP_END:
        raise ValueError("Stock UF2 overlaps protected flash")

    package_raw = package_path.read_bytes()
    package_digest = hashlib.sha256(package_raw).hexdigest()
    if package_digest != EXPECTED_PACKAGE_SHA256:
        raise ValueError(f"Unexpected DFU package SHA-256: {package_digest}")
    with zipfile.ZipFile(package_path) as package:
        expected_files = {"Reciever_test.ino.bin", "Reciever_test.ino.dat", "manifest.json"}
        if set(package.namelist()) != expected_files:
            raise ValueError("Unexpected files in stock DFU package")
        manifest = json.loads(package.read("manifest.json"))["manifest"]
        if set(manifest) != {"application", "dfu_version"}:
            raise ValueError("Stock DFU package is not application-only")
        application = manifest["application"]
        init_data = application["init_packet_data"]
        if manifest["dfu_version"] != 0.5 or init_data["device_type"] != 82:
            raise ValueError("Unexpected stock DFU manifest version or device type")
        if init_data["softdevice_req"] != [0x123]:
            raise ValueError("Unexpected stock DFU SoftDevice requirement")
        binary = package.read(application["bin_file"])

    binary_digest = hashlib.sha256(binary).hexdigest()
    if binary_digest != EXPECTED_BINARY_SHA256:
        raise ValueError(f"Unexpected application binary SHA-256: {binary_digest}")
    initial_stack, reset_vector = struct.unpack_from("<2I", binary)
    if initial_stack != 0x20020000 or not APP_BASE <= (reset_vector & ~1) < EXPECTED_END:
        raise ValueError("Unexpected stock application vector table")
    if APP_BASE + len(binary) > EXPECTED_END:
        raise ValueError("Stock DFU application exceeds the verified UF2 range")

    print(f"SHA-256: {digest.upper()}")
    print(f"DFU package SHA-256: {package_digest.upper()}")
    print(f"UF2 range: 0x{addresses[0]:05X}..0x{EXPECTED_END - 1:05X}")
    print("Stock dongle image verification passed")


def main() -> None:
    parser = argparse.ArgumentParser(description="Verify the preserved stock dongle UF2")
    parser.add_argument("path", nargs="?", type=Path, default=DEFAULT_IMAGE)
    parser.add_argument("--package", type=Path, default=DEFAULT_PACKAGE)
    args = parser.parse_args()
    verify(args.path, args.package)


if __name__ == "__main__":
    main()
