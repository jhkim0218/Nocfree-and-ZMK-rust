# Gates: recover KR input and backlight

OWNS: GATES.md, src/backlight.rs, src/pca9555.rs, src/scanner.rs, src/keymap/kr.rs, firmware/experimental/NocFree_And_Rust_ZMK_Based_KR_Experimental_Left.uf2, firmware/experimental/NocFree_And_Rust_ZMK_Based_KR_Experimental_Right.uf2

Scope: restore a visible default backlight level, make KR key scanning follow the official port limits, prevent one failed expander read from suppressing every key, and publish verified KR artifacts on develop.

- [x] G1: KR decoding uses the official right-side 0x22/P1 seven-bit limit and the selected extra 0x21/P0 inputs exactly once
  EVIDENCE: scanner::tests::kr_scan_matches_official_port_limits passed; subsequent automated launches were intermittently denied by local Windows policy after compilation

- [x] G2: one failed PCA9555 port read is released safely without discarding successful key ports
  CHECK: cargo test --target x86_64-pc-windows-msvc --lib --no-default-features --features layout-kr input_reads_isolate_failed_ports && echo PCA9555 fault isolation verification passed
  EXPECT: PCA9555 fault isolation verification passed
  EVIDENCE: exit=0; shell=C:\Windows\system32\cmd.exe; cwd=D:\etc\Nocfree-and-ZMK-rust; path=fd81366c96b4/39 entries; output=Finished `test` profile [optimized + debuginfo] target(s) in 3.60s | Running unittests src\lib.rs (target\x86_64-pc-windows-msvc\debug\deps\nocfree_and_rust-d343baa66fb7bba1.exe)

- [x] G3: the default backlight again drives the hardware-tested linear 20 percent level while off remains fully off
  CHECK: cargo test --target x86_64-pc-windows-msvc --lib --no-default-features --features layout-kr backlight_uses_visible_linear_steps && echo Backlight visibility verification passed
  EXPECT: Backlight visibility verification passed
  EVIDENCE: exit=0; shell=C:\Windows\system32\cmd.exe; cwd=D:\etc\Nocfree-and-ZMK-rust; path=fd81366c96b4/39 entries; output=Finished `test` profile [optimized + debuginfo] target(s) in 3.75s | Running unittests src\lib.rs (target\x86_64-pc-windows-msvc\debug\deps\nocfree_and_rust-d343baa66fb7bba1.exe)

- [x] G4: both KR and ANSI host-side library suites pass
  EVIDENCE: direct host test executables passed KR 76/76 and ANSI 74/74; Cargo's first launch of newly linked executables was intermittently denied by local Windows policy

- [x] G5: ARM release builds compile both KR halves and regenerate both KR UF2 artifacts
  EVIDENCE: central and right cargo release builds exited 0 for thumbv7em-none-eabihf with layout-kr; llvm-objcopy and tools/nocfree_uf2.py regenerated both files; the wrapper's Clippy step was unavailable because local Windows policy denied clippy-driver.exe

- [x] G6: both generated KR UF2 files are valid nRF52833 application images bounded to 0x27000..0x64fff
  CHECK: python -B tools/test_nocfree_uf2.py && echo KR UF2 boundary verification passed
  EXPECT: KR UF2 boundary verification passed
  EVIDENCE: exit=0; shell=C:\Windows\system32\cmd.exe; cwd=D:\etc\Nocfree-and-ZMK-rust; path=fd81366c96b4/39 entries; output=Ran 5 tests in 0.102s | OK

- [ ] G7: the committed fix and generated KR artifacts are pushed to origin/develop
  EVIDENCE: pending commit, push, and remote SHA verification
