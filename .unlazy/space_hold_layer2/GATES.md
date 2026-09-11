# Gates: space hold Layer 2

OWNS: src/keymap.rs, src/report.rs, src/usb_descriptor.rs, src/ble_hid.rs, src/bond_store.rs, src/link_keymap.rs, src/link_protocol.rs, src/output_router.rs, src/bin/central.rs, docs/**, README_ko.md, firmware/**

Scope: Caps Lock tap/hold Layer 2 navigation and pointer controls over USB and Bluetooth without changing existing layouts.

- [x] G1: Caps Lock tap/hold and Layer 2 navigation/pointer behavior are covered by the ANSI report tests.
  CHECK: cargo test --lib --target x86_64-pc-windows-msvc --no-default-features --features layout-ansi space_hold_layer2
  EXPECT: test result: ok
  EVIDENCE: 2026-09-07, 6 matching tests passed: QWERTY mapping, tap/hold, browser/navigation, pointer/scroll/click, Link binding round-trip, and fully customized Layer 2~7 preservation.

- [x] G2: Every supported layout preserves its keymap and passes the complete library regression suite.
  CHECK: python -B tools/build_release.py --all-layouts --dongle
  EXPECT: NocFree all-layout release verification passed
  EVIDENCE: 2026-09-06, ANSI/ISO/JIS/KR host tests, ARM clippy/builds, UF2 validation, and Dongle release build passed.

- [x] G3: The installed ANSI Left image is identified and the connected left half restarts with split diagnostics.
  EVIDENCE: 2026-09-12, SHA-256 4151F728F3111DE1145B9486E4E571878CBA925E5D5449FD21E726781C4E9DC0 installed through the NocFree UF2 bootloader. `RUST-LEFT` and COM19 re-enumerated, Windows registered the MI_01 mouse collection, and COM19 reported a split-ready event after 2,716 ms.
