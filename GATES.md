# Gates: v2 regression diagnosis and layout variants

OWNS: src/**, tools/**, firmware/**, README.md, README_ko.md, HANDOFF.md, ROADMAP.md, ROADMAP_ko.md, PROGRESS.md, GATES.md, Cargo.toml

Scope: preserve the current ANSI release behavior while preparing software-testable diagnostics and separately selected Experimental ISO/JIS/KR variants. The attached master work order is design input; this ledger records the implementation accepted for this repository. No firmware is copied to hardware in this phase.

- [x] G0: the untouched main-branch baseline passes formatting, host tests, host/ARM Clippy, both ARM release builds, protected-range checks, UF2 round trips, DFU packaging, and repository contracts
  CHECK: powershell -NoProfile -ExecutionPolicy Bypass -File tools\build-release.ps1 -Layout ANSI
  EXPECT: NocFree ANSI release verification passed

- [ ] G1: mandatory cross-half order regressions include both R-L-R and L-R-L (`T`, `K`, `A`) at 1/2/3/5/8/10 ms gaps, right jitter from 0 through 20 ms, duplicate/gap/reconcile cases, and at least 10,000 events with lost=0, duplicate=0, reordered=0, stuck=0 for the accepted model
  CHECK: cargo test --target x86_64-pc-windows-msvc scanner::tests
  EXPECT: test result: ok

- [x] G2: the BLE-host wake diagnostic disables only LEFT System OFF while BLE is the selected output; Wired/Disabled behavior, backlight timeout, BLE service, and split operation remain compiled, and policy tests cover every output mode
  CHECK: cargo test --target x86_64-pc-windows-msvc power_policy::tests
  EXPECT: test result: ok

- [x] G3: keymap code is split into shared code plus separate ANSI, ISO, JIS, and KR modules; exactly one layout feature is required and ANSI remains the default
  CHECK: cargo test --target x86_64-pc-windows-msvc --no-default-features --features layout-ansi keymap::tests
  EXPECT: test result: ok

- [x] G4: ANSI retains 37 LEFT + 47 RIGHT keys, every raw position resolves exactly once, default/Fn actions match the pre-refactor map, and its persisted keymap record carries version, ANSI identity, key count, and CRC
  CHECK: cargo test --target x86_64-pc-windows-msvc --no-default-features --features layout-ansi
  EXPECT: test result: ok

- [x] G5: ISO uses the official updater evidence (38 LEFT + unchanged 47 RIGHT), adds the LEFT `0x24/P0` seventh populated bit and HID Non-US positions, builds both ARM roles, and is labeled Experimental/hardware-unverified
  CHECK: powershell -NoProfile -ExecutionPolicy Bypass -File tools\build-release.ps1 -Layout ISO
  EXPECT: NocFree ISO release verification passed

- [x] G6: KR uses the official updater/product evidence (39 LEFT + 50 RIGHT), reads the documented `0x21/P0` additional port, maps the duplicated Y/H and F6/6/B positions without changing ANSI, builds both ARM roles, and is labeled Experimental/hardware-unverified
  CHECK: powershell -NoProfile -ExecutionPolicy Bypass -File tools\build-release.ps1 -Layout KR
  EXPECT: NocFree KR release verification passed

- [x] G7: JIS uses the hardware-tested `electricdoc187/NocFree-and-zmk` `jis-custom` scan map (37 LEFT + 48 RIGHT), builds both ARM roles, and documents the deferred left Eisu tap/hold behavior and Rust hardware-unverified status
  CHECK: python -m unittest tools.test_repository_contract.RepositoryContractTests.test_layout_variants_are_explicit_and_separate
  EXPECT: Ran 1 test

- [x] G8: all default ANSI release checks still pass after the changes and ISO/JIS/KR artifact ranges stay inside `0x27000..0x64fff`; neither protected flash nor recovery paths change
  CHECK: powershell -NoProfile -ExecutionPolicy Bypass -File tools\build-release.ps1 -Layout ANSI
  EXPECT: NocFree ANSI release verification passed

- [x] G9: English/Korean documentation distinguishes Stable ANSI, Experimental hardware-unverified ISO/JIS/KR, exact build commands, today’s no-hardware limitation, and the next physical test order; verified code is committed but not pushed unless requested
  CHECK: python -m unittest discover -s tools -p 'test_*.py'
  EXPECT: Ran 20 tests
