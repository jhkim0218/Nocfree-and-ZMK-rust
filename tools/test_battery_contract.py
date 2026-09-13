"""Run the same golden wire vectors against the actual no_std Rust codec."""
import json
import os
import subprocess
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


class BatteryWireTests(unittest.TestCase):
    def test_shared_vectors_against_firmware_decoder(self):
        vectors = json.loads((ROOT / "desktop/protocol/vectors.json").read_text(encoding="utf-8"))
        self.assertTrue(any(v["valid"] for v in vectors))
        self.assertTrue(any(not v["valid"] for v in vectors))
        output = ROOT / "target/battery-contract"
        output.mkdir(parents=True, exist_ok=True)
        codec = (ROOT / "src/battery_telemetry.rs").as_posix()
        lines = [f'#[path = "{codec}"] mod battery_telemetry;', '#[test] fn shared_vectors() {']
        for vector in vectors:
            data = list(bytes.fromhex(vector["hex"]))
            expected = str(vector["valid"]).lower()
            lines.append(f'assert_eq!(battery_telemetry::BatterySnapshot::decode(&{data}).is_some(), {expected}, {json.dumps(vector["name"])});')
        lines.append('}')
        source = output / "vectors.rs"
        source.write_text("\n".join(lines), encoding="utf-8")
        binary = output / ("vectors.exe" if os.name == "nt" else "vectors")
        subprocess.run(["rustc", "--edition=2024", "--test", str(source), "-o", str(binary)], cwd=ROOT, check=True)
        subprocess.run([str(binary)], cwd=ROOT, check=True)


if __name__ == "__main__":
    unittest.main()
