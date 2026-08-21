import struct
import tempfile
import unittest
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


if __name__ == "__main__":
    unittest.main()
