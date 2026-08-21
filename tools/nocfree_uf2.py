from __future__ import annotations

import argparse
import struct
from dataclasses import dataclass
from pathlib import Path


UF2_MAGIC_START0 = 0x0A324655
UF2_MAGIC_START1 = 0x9E5D5157
UF2_MAGIC_END = 0x0AB16F30
UF2_FLAG_FAMILY_ID_PRESENT = 0x00002000
NRF52833_FAMILY_ID = 0x621E937A
UF2_BLOCK_SIZE = 512
UF2_PAYLOAD_SIZE = 256
APP_BASE = 0x27000
APP_END = 0x65000
RAM_BASE = 0x20010000
RAM_END = 0x20020000


@dataclass
class UF2Image:
    blocks: dict[int, bytes]
    family_id: int = NRF52833_FAMILY_ID

    @classmethod
    def from_application(cls, data: bytes, base: int = APP_BASE) -> "UF2Image":
        blocks: dict[int, bytes] = {}
        for offset in range(0, len(data), UF2_PAYLOAD_SIZE):
            payload = data[offset : offset + UF2_PAYLOAD_SIZE]
            blocks[base + offset] = payload.ljust(UF2_PAYLOAD_SIZE, b"\xFF")
        return cls(blocks)

    @classmethod
    def load(cls, path: Path) -> "UF2Image":
        raw = path.read_bytes()
        if len(raw) % UF2_BLOCK_SIZE:
            raise ValueError(f"UF2 size is not a multiple of 512: {path}")
        blocks: dict[int, bytes] = {}
        family_id: int | None = None
        for offset in range(0, len(raw), UF2_BLOCK_SIZE):
            block = raw[offset : offset + UF2_BLOCK_SIZE]
            magic0, magic1, flags, address, size, _, _, family = struct.unpack_from(
                "<8I", block
            )
            if magic0 != UF2_MAGIC_START0 or magic1 != UF2_MAGIC_START1:
                raise ValueError(f"Invalid UF2 start magic at block {offset // 512}")
            if struct.unpack_from("<I", block, 508)[0] != UF2_MAGIC_END:
                raise ValueError(f"Invalid UF2 end magic at block {offset // 512}")
            if flags != UF2_FLAG_FAMILY_ID_PRESENT:
                raise ValueError(f"Unexpected UF2 flags: 0x{flags:08X}")
            if size != UF2_PAYLOAD_SIZE:
                raise ValueError(f"Unexpected payload size: {size}")
            if family_id is None:
                family_id = family
            elif family != family_id:
                raise ValueError("Mixed UF2 family IDs")
            if address in blocks:
                raise ValueError(f"Duplicate UF2 address: 0x{address:08X}")
            blocks[address] = block[32 : 32 + size]
        return cls(blocks, family_id or 0)

    def read(self, address: int, size: int) -> bytes:
        result = bytearray()
        while size:
            block_address = address & ~(UF2_PAYLOAD_SIZE - 1)
            block = self.blocks.get(block_address)
            if block is None:
                raise KeyError(f"Missing UF2 block at 0x{block_address:08X}")
            within = address - block_address
            take = min(size, UF2_PAYLOAD_SIZE - within)
            result.extend(block[within : within + take])
            address += take
            size -= take
        return bytes(result)

    def write(self, address: int, data: bytes) -> None:
        position = 0
        while position < len(data):
            block_address = address & ~(UF2_PAYLOAD_SIZE - 1)
            within = address - block_address
            block = bytearray(self.blocks.get(block_address, b"\xFF" * UF2_PAYLOAD_SIZE))
            take = min(len(data) - position, UF2_PAYLOAD_SIZE - within)
            block[within : within + take] = data[position : position + take]
            self.blocks[block_address] = bytes(block)
            address += take
            position += take

    def to_bytes(self) -> bytes:
        addresses = sorted(self.blocks)
        total = len(addresses)
        result = bytearray()
        for number, address in enumerate(addresses):
            payload = self.blocks[address]
            if len(payload) != UF2_PAYLOAD_SIZE:
                raise ValueError(f"Bad payload length at 0x{address:08X}")
            block = bytearray(UF2_BLOCK_SIZE)
            struct.pack_into(
                "<8I",
                block,
                0,
                UF2_MAGIC_START0,
                UF2_MAGIC_START1,
                UF2_FLAG_FAMILY_ID_PRESENT,
                address,
                UF2_PAYLOAD_SIZE,
                number,
                total,
                self.family_id,
            )
            block[32 : 32 + UF2_PAYLOAD_SIZE] = payload
            struct.pack_into("<I", block, 508, UF2_MAGIC_END)
            result.extend(block)
        return bytes(result)

    def save(self, path: Path) -> None:
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(self.to_bytes())


def validate_application(data: bytes) -> None:
    if len(data) < 8:
        raise ValueError("Application binary is too short for a vector table")
    if len(data) > APP_END - APP_BASE:
        raise ValueError(
            f"Application ends at 0x{APP_BASE + len(data):X}, beyond 0x{APP_END:X}"
        )

    initial_stack, reset_vector = struct.unpack_from("<2I", data)
    if not RAM_BASE <= initial_stack <= RAM_END:
        raise ValueError(f"Initial stack pointer is outside app RAM: 0x{initial_stack:X}")
    reset_address = reset_vector & ~1
    if reset_vector & 1 == 0 or not APP_BASE <= reset_address < APP_END:
        raise ValueError(f"Invalid Thumb reset vector: 0x{reset_vector:X}")


def pack_application(input_path: Path, output_path: Path) -> None:
    data = input_path.read_bytes()
    validate_application(data)
    UF2Image.from_application(data).save(output_path)

    written = UF2Image.load(output_path)
    if written.family_id != NRF52833_FAMILY_ID:
        raise ValueError(f"Wrong UF2 family ID: 0x{written.family_id:08X}")
    addresses = sorted(written.blocks)
    if not addresses or addresses[0] != APP_BASE:
        raise ValueError("UF2 does not start at the application boundary")
    if addresses[-1] + UF2_PAYLOAD_SIZE > APP_END:
        raise ValueError("UF2 overlaps protected flash")
    if written.read(APP_BASE, len(data)) != data:
        raise ValueError("UF2 round-trip verification failed")


def main() -> None:
    parser = argparse.ArgumentParser(
        description="Pack a NocFree nRF52833 application binary as a bounded UF2"
    )
    parser.add_argument("input", type=Path)
    parser.add_argument("output", type=Path)
    args = parser.parse_args()
    pack_application(args.input, args.output)


if __name__ == "__main__":
    main()
