import hashlib
import struct
import unittest
from pathlib import Path

from nocfree_uf2 import APP_BASE, APP_END, NRF52833_FAMILY_ID, UF2Image


ROOT = Path(__file__).resolve().parents[1]
OFFICIAL = (
    ROOT.parent
    / "NocFree_and_V2.3.0_Dongle.uf2"
)
RUST_DONGLE = (
    ROOT
    / "firmware"
    / "NocFree_And_Rust_ZMK_Based_Dongle.uf2"
)
OFFICIAL_SHA256 = "98097252B4752888D4CD959D98B019152B95E743D1AF4039DB841D670002A93C"


class OfficialDongleEvidenceTests(unittest.TestCase):
    def test_official_image_is_preserved_outside_git(self) -> None:
        self.assertTrue(OFFICIAL.is_file())
        self.assertEqual(hashlib.sha256(OFFICIAL.read_bytes()).hexdigest().upper(), OFFICIAL_SHA256)
        self.assertFalse(OFFICIAL.is_relative_to(ROOT))

    def test_official_image_proves_the_dongle_application_boundaries(self) -> None:
        image = UF2Image.load(OFFICIAL)
        addresses = sorted(image.blocks)
        self.assertEqual(image.family_id, NRF52833_FAMILY_ID)
        self.assertEqual((len(addresses), addresses[0], addresses[-1] + 256), (281, APP_BASE, 0x38900))
        stack, reset = struct.unpack("<2I", image.read(APP_BASE, 8))
        self.assertEqual(stack, 0x20020000)
        self.assertEqual(reset, 0x000348C1)
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
