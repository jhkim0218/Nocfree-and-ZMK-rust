# Gates: ANSI backlight hardware dimming recovery

OWNS: Cargo.toml, src/backlight.rs, tools/build_release.py, tools/test_documentation.py, tools/test_nocfree_uf2.py, tools/test_repository_contract.py, firmware/**, README.md, README_ko.md, README_ja.md, GATES.md

Scope: publish the physically tested 1 kHz perceptual ANSI backlight as the canonical firmware while documenting that upper perceived levels remain close.

- [x] G1: backlight state and PWM conversion expose six ordered 20-percent levels with enough hardware duty resolution
  CHECK: cargo test --target x86_64-pc-windows-msvc --lib --no-default-features --features layout-ansi,backlight-perceptual backlight_uses_selected_curve && echo Backlight duty verification passed
  EXPECT: Backlight duty verification passed
  EVIDENCE: exit=0; shell=C:\WINDOWS\system32\cmd.exe; cwd=D:\study\nocfree\NocFree-and-rust; path=735d250c57b9/36 entries; output=Finished `test` profile [optimized + debuginfo] target(s) in 0.29s | Running unittests src\lib.rs (target\x86_64-pc-windows-msvc\debug\deps\nocfree_and_rust-bdb99773bed8c535.exe)

- [x] G2: the complete ANSI host and ARM verification suite passes
  CHECK: cargo test --target x86_64-pc-windows-msvc --no-default-features --features layout-ansi,backlight-perceptual && cargo clippy --release --target thumbv7em-none-eabihf --no-default-features --features layout-ansi,backlight-perceptual --bin central -- -D warnings && cargo clippy --release --target thumbv7em-none-eabihf --no-default-features --features layout-ansi,backlight-perceptual --bin right -- -D warnings && echo ANSI regression verification passed
  EXPECT: ANSI regression verification passed
  EVIDENCE: exit=0; shell=C:\WINDOWS\system32\cmd.exe; cwd=D:\study\nocfree\NocFree-and-rust; path=735d250c57b9/36 entries; output=Finished `release` profile [optimized] target(s) in 0.14s | Finished `release` profile [optimized] target(s) in 0.08s

- [x] G3: canonical ANSI left and right UF2 artifacts build and pass repository artifact contracts
  CHECK: python -B tools/build_release.py --layout ANSI
  EXPECT: NocFree ANSI release verification passed
  EVIDENCE: exit=0; shell=C:\WINDOWS\system32\cmd.exe; cwd=D:\study\nocfree\NocFree-and-rust; path=735d250c57b9/36 entries; output=Ran 31 tests in 0.032s | OK

- [x] G4: real ANSI hardware retains input and synchronized PWM control with the accepted perceptual curve
  EVIDENCE: User verified input after both 1 kHz flashes. Linear made about three rising levels distinct; the subsequently flashed perceptual pair also controlled both halves, with upper perceived differences still small and explicitly accepted for publication on 2026-08-27.

- [ ] G5: verified behavior, artifact hashes, documentation, commit, origin/develop, and origin/main agree
  EVIDENCE: pending
