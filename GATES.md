# Gates: experimental Rust dongle firmware

OWNS: Cargo.toml, src/**, tools/**, firmware/experimental/**, README.md, README_ko.md, README_ja.md, ROADMAP.md, ROADMAP_ko.md, ROADMAP_ja.md, RECOVERY.md, RECOVERY_ko.md, RECOVERY_ja.md, GATES.md

Scope: build matching ANSI, ISO, JIS, and KR keyboard-to-dongle firmware sets, verify their software contracts, and leave exact hardware-test handoffs.

- [x] G1: dongle framing, advertisement matching, sequence handling, output routing, and dedicated bond records pass for every layout
  CHECK: cargo test --target x86_64-pc-windows-msvc --lib --no-default-features --features layout-ansi && cargo test --target x86_64-pc-windows-msvc --lib --no-default-features --features layout-iso && cargo test --target x86_64-pc-windows-msvc --lib --no-default-features --features layout-jis && cargo test --target x86_64-pc-windows-msvc --lib --no-default-features --features layout-kr && echo Dongle host verification passed
  EXPECT: Dongle host verification passed
  EVIDENCE: exit=0; shell=C:\WINDOWS\system32\cmd.exe; cwd=D:\study\nocfree\NocFree-and-rust; path=186adcb2655b/36 entries; output=Finished `test` profile [optimized + debuginfo] target(s) in 0.26s | Running unittests src\lib.rs (target\x86_64-pc-windows-msvc\debug\deps\nocfree_and_rust-895f109436db97e1.exe)

- [x] G2: every layout's left, right, and dongle ARM applications compile without warnings
  CHECK: for %L in (ansi iso jis kr) do @cargo clippy --release --target thumbv7em-none-eabihf --no-default-features --features layout-%L --bin central --bin right --bin dongle -- -D warnings || exit /b 1 & echo Dongle ARM verification passed
  EXPECT: Dongle ARM verification passed
  EVIDENCE: exit=0; shell=C:\WINDOWS\system32\cmd.exe; cwd=D:\study\nocfree\NocFree-and-rust; path=186adcb2655b/36 entries; output=Compiling nocfree-and-rust v0.1.0 (D:\study\nocfree\NocFree-and-rust) | Finished `release` profile [optimized] target(s) in 0.85s

- [x] G3: every layout-matched dongle UF2 is regenerated from the ARM application and stays inside the application partition
  CHECK: python -B tools/test_nocfree_uf2.py && echo Dongle UF2 verification passed
  EXPECT: Dongle UF2 verification passed
  EVIDENCE: exit=0; shell=C:\WINDOWS\system32\cmd.exe; cwd=D:\study\nocfree\NocFree-and-rust; path=186adcb2655b/36 entries; output=Ran 5 tests in 0.070s | OK

- [x] G4: repository artifact, documentation, and recovery contracts pass
  CHECK: python -B -m unittest discover -s tools -p test_*.py && echo Dongle repository verification passed
  EXPECT: Dongle repository verification passed
  EVIDENCE: exit=0; shell=C:\WINDOWS\system32\cmd.exe; cwd=D:\study\nocfree\NocFree-and-rust; path=186adcb2655b/36 entries; output=Ran 34 tests in 0.018s | OK

- [x] G5: the official v2.3.0 dongle UF2 remains untracked and unchanged while its hardware boundaries are recorded
  CHECK: python -B tools/test_dongle_firmware.py && echo Official dongle evidence verification passed
  EXPECT: Official dongle evidence verification passed
  EVIDENCE: exit=0; shell=C:\WINDOWS\system32\cmd.exe; cwd=D:\study\nocfree\NocFree-and-rust; path=186adcb2655b/36 entries; output=Ran 3 tests in 0.001s | OK

- [x] G6: matching experimental ANSI left, right, and dongle images pass pairing, reconnect, input, latency, coexistence, and recovery on physical hardware
  EVIDENCE: 2026-08-28: RUST-LEFT received ANSI Left UF2 sha256=189EA57EA47B21238313696EEAC298E9FBDDEB6FDF893CF36B4170FC07D778A1; RUST-RIGHT received ANSI Right UF2 sha256=2E6ADB5E9DA632018C3D2EB9451196EE2F0507A357EB47847E4B1E268E90876C; RUST-DONGLE received ANSI Dongle UF2 sha256=117AB0808667F24DB1EBE1BC9678FA2919A241C6A27D14BDC974738F2B5B52A2 and re-enumerated as USB\\VID_239A&PID_80D8 with keyboard, consumer, and COM7 interfaces. User confirmed initial pairing, dongle-routed ANSI input, unplug/replug reconnect with both halves, fast alternating input without delay/loss/stuck keys, and BLE-to-2.4G return without interference. All three devices entered their verified UF2 bootloaders and returned to their expected Rust USB identities after the matching ANSI images were written.

- [x] G7: ISO, JIS, and KR tester handoff specifies matching image names and pairing, reconnect, input, latency, coexistence, and recovery criteria
  EVIDENCE: use each same-layout Left, Right, and Dongle UF2 from firmware/experimental; ISO=NocFree_And_Rust_ZMK_Based_ISO_Experimental_{Left,Right,Dongle}.uf2, JIS=NocFree_And_Rust_ZMK_Based_JIS_Experimental_{Left,Right,Dongle}.uf2, KR=NocFree_And_Rust_ZMK_Based_KR_Experimental_{Left,Right,Dongle}.uf2. Tester must verify first pairing, both-half input, fast alternating/modifier input without loss or stuck keys, dongle unplug/replug recovery, BLE-to-2.4G return, and all three 1200-baud UF2 recovery paths. No ISO/JIS/KR hardware-success claim is made until those results are returned.
