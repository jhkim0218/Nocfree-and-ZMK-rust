# Gates: current ANSI firmware and reproducible backlight A/B pair

OWNS: Cargo.toml, src/backlight.rs, tools/build_release.py, tools/test_documentation.py, tools/test_nocfree_uf2.py, firmware/**, README.md, README_ko.md, README_ja.md, GATES.md

Scope: make the latest develop source, documentation, tests, and ANSI artifacts agree on a visible linear default while publishing a reproducible perceptual-curve A/B pair for physical ANSI testing.

- [x] G0: this ledger states executable outcomes that can fail
  CHECK: node C:\Users\kjh\.codex\skills\unlazy\scripts\gate-lint.mjs GATES.md
  EXPECT: LINT OK
  EVIDENCE: exit=0; shell=C:\WINDOWS\system32\cmd.exe; cwd=D:\study\nocfree\NocFree-and-rust; path=735d250c57b9/36 entries; output=WARN  G8: title states a number that nothing measures: "the committed source, documentation, and ANSI comparison UF2 files are pushed to origin/develop"  [unmeasured-number] | LINT OK (2 warning(s))

- [x] G1: host tests prove distinct linear-default and perceptual-candidate duty sequences
  CHECK: cargo test --target x86_64-pc-windows-msvc --lib --no-default-features --features layout-ansi backlight_uses_selected_curve && cargo test --target x86_64-pc-windows-msvc --lib --no-default-features --features layout-ansi,backlight-perceptual backlight_uses_selected_curve && echo Backlight A-B host verification passed
  EXPECT: Backlight A-B host verification passed
  EVIDENCE: exit=0; shell=C:\WINDOWS\system32\cmd.exe; cwd=D:\study\nocfree\NocFree-and-rust; path=735d250c57b9/36 entries; output=Finished `test` profile [optimized + debuginfo] target(s) in 0.30s | Running unittests src\lib.rs (target\x86_64-pc-windows-msvc\debug\deps\nocfree_and_rust-bdb99773bed8c535.exe)

- [x] G2: source, portable builder, artifact names, and three README editions agree on the two backlight curves
  CHECK: python -B -m unittest tools.test_documentation.DocumentationTests.test_backlight_contract tools.test_documentation.DocumentationTests.test_backlight_ab_build_contract && echo Backlight A-B contract verification passed
  EXPECT: Backlight A-B contract verification passed
  EVIDENCE: exit=0; shell=C:\WINDOWS\system32\cmd.exe; cwd=D:\study\nocfree\NocFree-and-rust; path=735d250c57b9/36 entries; output=Ran 2 tests in 0.001s | OK

- [x] G3: the perceptual ANSI comparison pair passes host and ARM checks and packages successfully
  CHECK: python -B tools/build_release.py --layout ANSI --backlight-curve perceptual
  EXPECT: NocFree ANSI perceptual backlight release verification passed
  EVIDENCE: exit=0; shell=C:\WINDOWS\system32\cmd.exe; cwd=D:\study\nocfree\NocFree-and-rust; path=735d250c57b9/36 entries; output=Ran 31 tests in 0.017s | OK

- [x] G4: the default latest-source ANSI pair passes host and ARM checks and packages successfully
  CHECK: python -B tools/build_release.py --layout ANSI
  EXPECT: NocFree ANSI release verification passed
  EVIDENCE: exit=0; shell=C:\WINDOWS\system32\cmd.exe; cwd=D:\study\nocfree\NocFree-and-rust; path=735d250c57b9/36 entries; output=Ran 31 tests in 0.032s | OK

- [x] G5: canonical and A/B ANSI UF2 pairs are valid nRF52833 applications inside the protected partition
  CHECK: python -B tools/test_nocfree_uf2.py && echo ANSI A-B UF2 verification passed
  EXPECT: ANSI A-B UF2 verification passed
  EVIDENCE: exit=0; shell=C:\WINDOWS\system32\cmd.exe; cwd=D:\study\nocfree\NocFree-and-rust; path=735d250c57b9/36 entries; output=Ran 5 tests in 0.008s | OK

- [x] G6: the complete Python repository, documentation, and artifact contract suite passes
  CHECK: python -B -m unittest discover -s tools -p test_*.py && echo Full Python contract verification passed
  EXPECT: Full Python contract verification passed
  EVIDENCE: exit=0; shell=C:\WINDOWS\system32\cmd.exe; cwd=D:\study\nocfree\NocFree-and-rust; path=735d250c57b9/36 entries; output=Ran 31 tests in 0.016s | OK

- [x] G7: README editions identify both A/B candidates as hardware-unverified and provide exact flash-test guidance
  CHECK: python -B -m unittest tools.test_documentation.DocumentationTests.test_hardware_validation_disclosure && echo Hardware disclosure verification passed
  EXPECT: Hardware disclosure verification passed
  EVIDENCE: exit=0; shell=C:\WINDOWS\system32\cmd.exe; cwd=D:\study\nocfree\NocFree-and-rust; path=735d250c57b9/36 entries; output=Ran 1 test in 0.000s | OK

- [ ] G8: the committed source, documentation, and ANSI comparison firmware are pushed to origin/develop
  EVIDENCE: pending
