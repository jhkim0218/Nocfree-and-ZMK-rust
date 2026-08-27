# Gates: D1 dongle USB HID

OWNS: GATES.md, Cargo.toml, src/lib.rs, src/bin/central.rs, src/bin/right.rs, src/bin/dongle.rs, src/usb_descriptor.rs, src/platform.rs, tools/build_release.py, tools/test_repository_contract.py, tools/test_nocfree_uf2.py, firmware/**, README.md, README_ko.md, README_ja.md, PROGRESS.md, PROGRESS_ko.md, PROGRESS_ja.md

Scope: build a radio-free Rust dongle application that exposes keyboard, consumer-control, and recovery CDC interfaces without changing the bootloader, SoftDevice, or UICR.

- [x] G0: this D1 ledger states mechanically valid gates
  CHECK: node C:\Users\kjh\.codex\skills\unlazy\scripts\gate-lint.mjs GATES.md
  EXPECT: LINT OK
  EVIDENCE: exit=0; shell=C:\WINDOWS\system32\cmd.exe; cwd=D:\study\nocfree\NocFree-and-rust; path=735d250c57b9/36 entries; output=WARN  G6: title states a number that nothing measures: "D1 behavior, limitations, artifact hashes, and recovery instructions are documented and committed"  [unmeasured-number] | LINT OK (6 warning(s))

- [x] G1: dongle USB descriptors and 1200-baud recovery behavior pass host tests
  CHECK: cargo test --target x86_64-pc-windows-msvc --no-default-features --features layout-ansi,backlight-perceptual,standalone-critical-section && echo Dongle host verification passed
  EXPECT: Dongle host verification passed
  EVIDENCE: exit=0; shell=C:\WINDOWS\system32\cmd.exe; cwd=D:\study\nocfree\NocFree-and-rust; path=735d250c57b9/36 entries; output=Running unittests src\lib.rs (target\x86_64-pc-windows-msvc\debug\deps\nocfree_and_rust-3f872567f7f30d4c.exe) | Doc-tests nocfree_and_rust

- [x] G2: the radio-free dongle binary passes ARM clippy and release compilation
  CHECK: cargo clippy --release --target thumbv7em-none-eabihf --no-default-features --features layout-ansi,backlight-perceptual,standalone-critical-section --bin dongle -- -D warnings && echo Dongle ARM verification passed
  EXPECT: Dongle ARM verification passed
  EVIDENCE: exit=0; shell=C:\WINDOWS\system32\cmd.exe; cwd=D:\study\nocfree\NocFree-and-rust; path=735d250c57b9/36 entries; output=Dongle ARM verification passed | Finished `release` profile [optimized] target(s) in 0.13s

- [x] G3: D1 UF2 and serial-DFU artifacts contain only the dongle application inside verified flash bounds
  CHECK: python -B tools/build_release.py --layout ANSI --dongle
  EXPECT: NocFree ANSI dongle D1 release verification passed
  EVIDENCE: exit=0; shell=C:\WINDOWS\system32\cmd.exe; cwd=D:\study\nocfree\NocFree-and-rust; path=735d250c57b9/36 entries; output=Ran 35 tests in 0.027s | OK

- [x] G4: the D1 dongle enumerates on Windows 11 as keyboard, consumer control, and CDC without unsolicited key input
  EVIDENCE: On 2026-08-27 Windows 11 reported `NocFree Rust Dongle`, parent `USB\VID_2886&PID_8029\RUST-DONGLE`, two healthy HID interfaces, keyboard and consumer-control children, and CDC COM5. No unsolicited report was observed; D1 intentionally holds both HID writers without calling `write`.

- [x] G5: 1200-baud recovery enters the verified factory UF2 and CDC bootloader and restores the same D1 application
  EVIDENCE: A verified COM5 1200-baud touch entered `USB\VID_239A&PID_0029\E19D2CEA0B437049` with CDC COM11 and UF2 mass storage. User-approved serial DFU of D1 package SHA-256 `2AB41BE31B78157994BB84A645A866A8D36DD6597D28C8B4E1B7C3F57818E362` reported `Device programmed.`; the same D1 product, two HID interfaces, keyboard/consumer children, and COM5 returned with zero bootloader nodes.

- [x] G6: D1 behavior, limitations, artifact hashes, and recovery instructions are documented and committed
  EVIDENCE: README.md/README_ko.md/README_ja.md, PROGRESS*, ROADMAP*, RECOVERY*, and HANDOFF.md record the radio-free boundary, Windows 11 evidence, PID_0029 recovery path, artifact hashes, and remaining D2-D4 work; this gate is included in the D1 commit.
