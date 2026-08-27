import hashlib
import struct
import subprocess
import unittest
from pathlib import Path

from nocfree_uf2 import APP_BASE, APP_END, NRF52833_FAMILY_ID, UF2Image


ROOT = Path(__file__).resolve().parents[1]
OFFICIAL = (
    ROOT
    / "NocFree_V2.3.21_All_UF2"
    / "NocFree_and_V2.3.21_Dongle.uf2"
)
RUST_DONGLE = (
    ROOT
    / "firmware"
    / "experimental"
    / "NocFree_And_Rust_ZMK_Based_KR_Experimental_Dongle.uf2"
)
OFFICIAL_SHA256 = "11D81AEFD52554BCC7E31634518BBAF858B55C718C9C8705E76C9F389031939A"


class OfficialDongleEvidenceTests(unittest.TestCase):
    def test_official_image_is_preserved_outside_git(self) -> None:
        self.assertTrue(OFFICIAL.is_file())
        self.assertEqual(hashlib.sha256(OFFICIAL.read_bytes()).hexdigest().upper(), OFFICIAL_SHA256)
        tracked = subprocess.check_output(
            ("git", "ls-files"), cwd=ROOT, text=True
        ).splitlines()
        self.assertNotIn(OFFICIAL.relative_to(ROOT).as_posix(), tracked)

    def test_official_image_proves_the_dongle_application_boundaries(self) -> None:
        image = UF2Image.load(OFFICIAL)
        addresses = sorted(image.blocks)
        self.assertEqual(image.family_id, NRF52833_FAMILY_ID)
        self.assertEqual((len(addresses), addresses[0], addresses[-1] + 256), (286, APP_BASE, 0x38E00))
        stack, reset = struct.unpack("<2I", image.read(APP_BASE, 8))
        self.assertEqual(stack, 0x20020000)
        self.assertEqual(reset, 0x00034DDD)
        self.assertLess(addresses[-1] + 256, APP_END)
        payload = b"".join(image.blocks[address] for address in addresses)
        self.assertIn(b"NocFree_Dongle", payload)
        self.assertIn(bytes.fromhex("12 01 00 02 00 00 00 40 9A 23 D8 80"), payload)

    def test_rust_dongle_stays_in_the_same_application_partition(self) -> None:
        self.assertTrue(RUST_DONGLE.is_file())
        image = UF2Image.load(RUST_DONGLE)
        addresses = sorted(image.blocks)
        self.assertEqual(image.family_id, NRF52833_FAMILY_ID)
        self.assertEqual(addresses[0], APP_BASE)
        self.assertLessEqual(addresses[-1] + 256, APP_END)


if __name__ == "__main__":
    unittest.main()
