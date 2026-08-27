# Gates: experimental Rust dongle firmware

OWNS: Cargo.toml, src/**, tools/**, firmware/experimental/**, README.md, README_ko.md, README_ja.md, ROADMAP.md, ROADMAP_ko.md, ROADMAP_ja.md, RECOVERY.md, RECOVERY_ko.md, RECOVERY_ja.md, GATES.md

Scope: add a secure BLE keyboard-to-dongle transport, build a layout-matched nRF52833 dongle UF2, and leave an exact recovery and hardware-test handoff.

- [x] G1: dongle framing, advertisement matching, sequence handling, output routing, and dedicated bond records pass host tests
  CHECK: cargo test --target x86_64-pc-windows-msvc --lib --no-default-features --features layout-kr && echo Dongle host verification passed
  EXPECT: Dongle host verification passed
  EVIDENCE: exit=0; shell=C:\Windows\system32\cmd.exe; cwd=D:\etc\Nocfree-and-ZMK-rust; path=da2fab1ecc4e/39 entries; output=Finished `test` profile [optimized + debuginfo] target(s) in 8.00s | Running unittests src\lib.rs (target\x86_64-pc-windows-msvc\debug\deps\nocfree_and_rust-d343baa66fb7bba1.exe)

- [x] G2: KR left, right, and dongle ARM applications compile without warnings
  CHECK: cargo clippy --release --target thumbv7em-none-eabihf --no-default-features --features layout-kr --bin central --bin right --bin dongle -- -D warnings && cargo build --release --target thumbv7em-none-eabihf --no-default-features --features layout-kr --bin central --bin right --bin dongle && echo Dongle ARM verification passed
  EXPECT: Dongle ARM verification passed
  EVIDENCE: exit=0; shell=C:\Windows\system32\cmd.exe; cwd=D:\etc\Nocfree-and-ZMK-rust; path=da2fab1ecc4e/39 entries; output=warning: `nrf-softdevice-macro` (lib) generated 1 warning | Finished `release` profile [optimized] target(s) in 12.59s

- [x] G3: the KR dongle UF2 is regenerated from the ARM application and stays inside the application partition
  CHECK: python -B -c "import sys; sys.path.insert(0, 'tools'); import build_release as b; b.run('cargo', 'fmt', '--package', 'nocfree-and-rust', '--', '--check'); b.build_dongle('KR', b.llvm_objcopy())" && python -B tools/test_dongle_firmware.py && echo Dongle UF2 verification passed
  EXPECT: Dongle UF2 verification passed
  EVIDENCE: exit=0; shell=C:\Windows\system32\cmd.exe; cwd=D:\etc\Nocfree-and-ZMK-rust; path=da2fab1ecc4e/39 entries; output=Ran 3 tests in 1.125s | OK

- [x] G4: repository artifact, documentation, and recovery contracts pass
  CHECK: python -B -m unittest discover -s tools -p test_*.py && echo Dongle repository verification passed
  EXPECT: Dongle repository verification passed
  EVIDENCE: exit=0; shell=C:\Windows\system32\cmd.exe; cwd=D:\etc\Nocfree-and-ZMK-rust; path=da2fab1ecc4e/39 entries; output=Ran 34 tests in 1.535s | OK

- [x] G5: the official v2.3.21 dongle UF2 remains untracked and unchanged while its hardware boundaries are recorded
  CHECK: python -B tools/test_dongle_firmware.py && echo Official dongle evidence verification passed
  EXPECT: Official dongle evidence verification passed
  EVIDENCE: exit=0; shell=C:\Windows\system32\cmd.exe; cwd=D:\etc\Nocfree-and-ZMK-rust; path=da2fab1ecc4e/39 entries; output=Ran 3 tests in 1.157s | OK

- [ ] G6: matching experimental KR left, right, and dongle images pass pairing, reconnect, input, latency, coexistence, and recovery on physical hardware
  EVIDENCE: pending; no matching keyboard is currently available, so no firmware will be flashed and no hardware-success claim will be made.
