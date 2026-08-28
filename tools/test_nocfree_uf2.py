import json
import struct
import tempfile
import unittest
import zipfile
from pathlib import Path

from nocfree_uf2 import (
    APP_BASE,
    APP_END,
    NRF52833_FAMILY_ID,
    RAM_END,
    UF2Image,
    pack_application,
    validate_application,
)


ROOT = Path(__file__).resolve().parents[1]


def valid_application(payload: bytes = b"") -> bytes:
    return struct.pack("<2I", RAM_END, APP_BASE + 0x101) + payload


class Uf2PackingTests(unittest.TestCase):
    def test_pack_round_trips_inside_application_partition(self) -> None:
        application = valid_application(bytes(range(256)) * 2)
        with tempfile.TemporaryDirectory() as directory:
            input_path = Path(directory) / "app.bin"
            output_path = Path(directory) / "app.uf2"
            input_path.write_bytes(application)

            pack_application(input_path, output_path)
            image = UF2Image.load(output_path)

        self.assertEqual(image.family_id, NRF52833_FAMILY_ID)
        self.assertEqual(min(image.blocks), APP_BASE)
        self.assertLessEqual(max(image.blocks) + 256, APP_END)
        self.assertEqual(image.read(APP_BASE, len(application)), application)

    def test_rejects_application_that_crosses_storage_boundary(self) -> None:
        application = valid_application(b"\0" * (APP_END - APP_BASE - 7))
        with self.assertRaisesRegex(ValueError, "beyond"):
            validate_application(application)

    def test_rejects_invalid_stack_and_reset_vectors(self) -> None:
        with self.assertRaisesRegex(ValueError, "stack"):
            validate_application(struct.pack("<2I", 0, APP_BASE + 0x101))
        with self.assertRaisesRegex(ValueError, "reset"):
            validate_application(struct.pack("<2I", RAM_END, APP_BASE + 0x100))


class DfuPackageTests(unittest.TestCase):
    def test_serial_dfu_packages_contain_the_current_applications(self) -> None:
        for half in ("Left", "Right"):
            binary = ROOT / "firmware" / f"NocFree_Rust_{half}.bin"
            package = ROOT / "firmware" / f"NocFree_Rust_{half}_DFU.zip"
            with self.subTest(half=half), zipfile.ZipFile(package) as archive:
                self.assertEqual(
                    archive.read(f"NocFree_Rust_{half}.bin"), binary.read_bytes()
                )
                manifest = json.loads(archive.read("manifest.json"))["manifest"]
                application = manifest["application"]["init_packet_data"]
                self.assertEqual(manifest["dfu_version"], 0.5)
                self.assertEqual(application["device_type"], 82)
                self.assertEqual(application["softdevice_req"], [0xFFFE])


class ExperimentalArtifactTests(unittest.TestCase):
    def test_layout_uf2_pairs_stay_inside_the_application_partition(self) -> None:
        directory = ROOT / "firmware" / "experimental"
        paths = sorted(directory.glob("*.uf2"))
        self.assertEqual(len(paths), 12)
        for layout in ("ISO", "JIS", "KR"):
            for half in ("Left", "Right"):
                expected = (
                    directory
                    / f"NocFree_And_Rust_ZMK_Based_{layout}_Experimental_{half}.uf2"
                )
                self.assertIn(expected, paths)
        for half in ("Left", "Right"):
            self.assertIn(
                directory
                / f"NocFree_And_Rust_ZMK_Based_ANSI_Linear_Backlight_Experimental_{half}.uf2",
                paths,
            )
        for layout in ("ANSI", "ISO", "JIS", "KR"):
            self.assertIn(
                directory / f"NocFree_And_Rust_ZMK_Based_{layout}_Experimental_Dongle.uf2",
                paths,
            )
        for path in paths:
            with self.subTest(path=path.name):
                image = UF2Image.load(path)
                addresses = sorted(image.blocks)
                self.assertEqual(image.family_id, NRF52833_FAMILY_ID)
                self.assertEqual(addresses[0], APP_BASE)
                self.assertLessEqual(addresses[-1] + 256, APP_END)


if __name__ == "__main__":
    unittest.main()
