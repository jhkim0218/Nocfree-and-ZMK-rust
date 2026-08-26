# Gates: portable build, documentation, ordering, and backlight correction

OWNS: src/**, tools/**, firmware/**, README*.md, ROADMAP*.md, RECOVERY*.md, LAYOUTS*.md, HANDOFF*.md, PROGRESS*.md, GATES.md, Cargo.toml

Scope: ship a 5 ms configurable ordering candidate, visibly stepped synchronized backlight control, portable Windows/macOS/Linux builds, and complete English/Korean/Japanese user documentation without regressing recovery or existing ANSI behavior.

- [x] G0: this ledger states executable outcomes that can fail
  CHECK: node C:\Users\kjh\.codex\skills\unlazy\scripts\gate-lint.mjs GATES.md
  EXPECT: LINT OK
  EVIDENCE: 2026-08-26 LINT OK; warnings are limited to explicit manual hardware evidence and unittest success text

- [x] G1: the configured cross-half window is 5 ms and its tuning contract is tested and documented
  CHECK: python -B -m unittest tools.test_documentation.DocumentationTests.test_ordering_window_contract
  EXPECT: OK
  EVIDENCE: DocumentationTests.test_ordering_window_contract passed in the final 26-test suite

- [x] G2: backlight controls use 10 kHz, correct Fn directions, and six distinct monotonic levels including off
  CHECK: python -B -m unittest tools.test_documentation.DocumentationTests.test_backlight_contract
  EXPECT: OK
  EVIDENCE: Rust backlight/keymap tests and DocumentationTests.test_backlight_contract passed for all four layouts

- [x] G3: one standard-library Python entry point supports Windows, macOS, and Linux build workflows while PowerShell remains compatible
  CHECK: python -B -m unittest tools.test_documentation.DocumentationTests.test_portable_build_contract
  EXPECT: OK
  EVIDENCE: portable builder help/contract passed; Windows executed the full all-layout pipeline; macOS/Linux commands are documented but not run on those OSes

- [x] G4: every user documentation family has English, Korean, and Japanese editions with valid navigation
  CHECK: python -B -m unittest tools.test_documentation.DocumentationTests.test_language_coverage
  EXPECT: OK
  EVIDENCE: DocumentationTests.test_language_coverage passed for six user guides and both NocFree vendor patch notes

- [x] G5: README editions prominently state Windows default mode and the macOS selection shortcut
  CHECK: python -B -m unittest tools.test_documentation.DocumentationTests.test_default_os_guidance
  EXPECT: OK
  EVIDENCE: default LinkKeymap system=1 Rust test and three-language README contract passed

- [x] G6: all four layouts pass host tests, host/ARM Clippy, both release builds, packaging, and artifact checks
  CHECK: python -B tools/build_release.py --all-layouts
  EXPECT: NocFree all-layout release verification passed
  EVIDENCE: 2026-08-26 python -B tools/build_release.py --all-layouts ended with NocFree all-layout release verification passed

- [x] G7: JIS and KR expanded key paths are represented by scanner and HID regression tests
  CHECK: python -B -m unittest tools.test_documentation.DocumentationTests.test_expanded_layout_input_contract
  EXPECT: OK
  EVIDENCE: JIS 19-byte HID ARM builds and KR 50-key right scanner ARM builds passed; standard layouts retained 16-byte HID tests

- [x] G8: both independent 1200-baud DFU paths and protected flash boundaries remain present
  CHECK: python -B -m unittest tools.test_repository_contract.RepositoryContractTests.test_dfu_uses_softdevice_system_calls tools.test_repository_contract.RepositoryContractTests.test_only_left_exposes_host_hid tools.test_repository_contract.RepositoryContractTests.test_linker_and_storage_boundaries_preserve_factory_regions
  EXPECT: OK
  EVIDENCE: final repository and UF2/DFU artifact contract suite passed

- [x] G9: with no keyboard available, every new hardware-facing candidate is explicitly labeled hardware-unverified and no flash/deployment claim is made
  CHECK: python -B -m unittest tools.test_documentation.DocumentationTests.test_hardware_validation_disclosure
  EXPECT: OK
  EVIDENCE: user explicitly waived physical testing on 2026-08-26; the three README editions preserve the unverified status
