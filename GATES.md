# Gates: universal Rust dongle firmware

OWNS: Cargo.toml, src/**, tools/**, firmware/**, README.md, README_ko.md, README_ja.md, ROADMAP.md, ROADMAP_ko.md, ROADMAP_ja.md, RECOVERY.md, RECOVERY_ko.md, RECOVERY_ja.md, GATES.md

Scope: build ANSI, ISO, JIS, and KR keyboard firmware with one universal dongle, verify their software contracts, and record exact hardware-test handoffs.

- [x] G1: dongle framing, advertisement matching, sequence handling, output routing, and dedicated bond records pass for every layout
  CHECK: cargo test --target x86_64-pc-windows-msvc --lib --no-default-features --features layout-ansi && cargo test --target x86_64-pc-windows-msvc --lib --no-default-features --features layout-iso && cargo test --target x86_64-pc-windows-msvc --lib --no-default-features --features layout-jis && cargo test --target x86_64-pc-windows-msvc --lib --no-default-features --features layout-kr && echo Dongle host verification passed
  EXPECT: Dongle host verification passed
  EVIDENCE: exit=0; shell=C:\WINDOWS\system32\cmd.exe; cwd=D:\study\nocfree\NocFree-and-rust; path=186adcb2655b/36 entries; output=Finished `test` profile [optimized + debuginfo] target(s) in 0.30s | Running unittests src\lib.rs (target\x86_64-pc-windows-msvc\debug\deps\nocfree_and_rust-895f109436db97e1.exe)

- [x] G2: every layout's left and right plus the maximum-size universal dongle ARM applications compile without warnings
  CHECK: for %L in (ansi iso jis kr) do @cargo clippy --release --target thumbv7em-none-eabihf --no-default-features --features layout-%L --bin central --bin right -- -D warnings || exit /b 1 & cargo clippy --release --target thumbv7em-none-eabihf --no-default-features --features layout-jis --bin dongle -- -D warnings && echo Dongle ARM verification passed
  EXPECT: Dongle ARM verification passed
  EVIDENCE: exit=0; shell=C:\WINDOWS\system32\cmd.exe; cwd=D:\study\nocfree\NocFree-and-rust; path=186adcb2655b/36 entries; output=Finished `release` profile [optimized] target(s) in 0.09s | Finished `release` profile [optimized] target(s) in 0.09s

- [x] G3: the one universal dongle UF2 is regenerated from the maximum-size ARM application and stays inside the application partition
  CHECK: python -B tools/test_nocfree_uf2.py && echo Dongle UF2 verification passed
  EXPECT: Dongle UF2 verification passed
  EVIDENCE: exit=0; shell=C:\WINDOWS\system32\cmd.exe; cwd=D:\study\nocfree\NocFree-and-rust; path=186adcb2655b/36 entries; output=Ran 6 tests in 0.014s | OK

- [x] G4: repository artifact, documentation, and recovery contracts pass
  CHECK: python -B -m unittest discover -s tools -p test_*.py && echo Dongle repository verification passed
  EXPECT: Dongle repository verification passed
  EVIDENCE: exit=0; shell=C:\WINDOWS\system32\cmd.exe; cwd=D:\study\nocfree\NocFree-and-rust; path=186adcb2655b/36 entries; output=Ran 35 tests in 0.018s | OK

- [x] G5: the official v2.3.0 dongle UF2 remains untracked and unchanged while its hardware boundaries are recorded
  CHECK: python -B tools/test_dongle_firmware.py && echo Official dongle evidence verification passed
  EXPECT: Official dongle evidence verification passed
  EVIDENCE: exit=0; shell=C:\WINDOWS\system32\cmd.exe; cwd=D:\study\nocfree\NocFree-and-rust; path=186adcb2655b/36 entries; output=Ran 3 tests in 0.001s | OK

- [x] G6: universal ANSI left, right, and dongle images pass pairing, reconnect, input, latency, coexistence, and recovery on physical hardware
  EVIDENCE: 2026-08-28; flashed the regenerated ANSI Left UF2 and universal Dongle UF2 through their verified 1200-baud UF2 bootloaders. Windows enumerated `RUST-LEFT` and `RUST-DONGLE` keyboard/consumer HID interfaces without errors; hardware pairing, both-half input, rapid input, dongle reconnect, and recovery were operator-confirmed. The corrected ANSI Left remained connected and accepted input after more than six idle minutes in 2.4G mode without replugging the dongle. ANSI Right UF2 was unchanged from the prior verified image. ISO/JIS/KR remain unverified on matching hardware.

- [x] G7: ISO, JIS, and KR tester handoff specifies matching keyboard images, the universal dongle, and pairing, reconnect, input, latency, coexistence, and recovery criteria
  EVIDENCE: use each layout's Left/Right UF2 from firmware/experimental with firmware/NocFree_And_Rust_ZMK_Based_Dongle.uf2. Tester must verify first pairing, both-half input, fast alternating/modifier input without loss or stuck keys, dongle unplug/replug recovery, BLE-to-2.4G return, and all three 1200-baud UF2 recovery paths. No ISO/JIS/KR hardware-success claim is made until those results are returned.
