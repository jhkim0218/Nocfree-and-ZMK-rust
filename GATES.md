# Gates: P4 global cross-half ordering

OWNS: src/**, tools/**, firmware/**, README.md, README_ko.md, HANDOFF.md, ROADMAP.md, ROADMAP_ko.md, RECOVERY.md, PROGRESS.md, GATES.md

Scope: preserve physical ordering across the local left scanner and BLE-connected right scanner by carrying source time and sequence, converting right time into the left clock domain, and holding both sources in one short bounded reorder queue. LEFT remains authoritative. Do not change protected flash, recovery, keymap, output routing, backlight policy, host BLE profiles, or the P3.3 +8 dBm setting.

Post-closure qualification note: these gates establish a hardware-tested
candidate, not stable end-to-end completion. The 10,000-event gate exercises
the reorder model rather than every runtime queue and transport; follow the
P4.1–P4.3 plan in `PROGRESS.md`/`ROADMAP.md` before promoting it to stable.

- [x] G0: this ledger states outcomes that can fail
  CHECK: node "C:\Users\kjh\.codex\skills\unlazy\scripts\gate-lint.mjs" GATES.md
  EXPECT: LINT OK
  EVIDENCE: gate lint completed with LINT OK; warnings identify the expected manual hardware and documentation gates.

- [x] G1: every right snapshot carries state, a monotonic source timestamp, sequence, and reconciliation flag in at most the default ATT payload; codec and wrap tests pass
  CHECK: cargo test --target x86_64-pc-windows-msvc split_protocol::tests
  EXPECT: test result: ok
  EVIDENCE: the 20-byte frame round-trips state u64, source time u64, sequence u16, and flags/reserved bytes; ATT MTU 23 leaves exactly 20 value bytes. All split_protocol tests pass.

- [x] G2: reconnect performs a bounded three-sample clock-offset estimate before split-ready and refreshes it periodically without synchronizing on each key
  CHECK: python -m unittest tools.test_repository_contract.RepositoryContractTests.test_split_clock_is_synced_before_ready_and_refreshed
  EXPECT: OK
  EVIDENCE: the central takes three echo-validated samples, chooses the minimum-RTT estimate before recording SplitReady, converts right source time to the left domain, and repeats after 60 seconds. The focused repository contract passes.

- [x] G3: windows 1, 2, 3, 4, and 5 ms are compared over at least 10,000 delayed alternating snapshots; the smallest window with lost=0, duplicate=0, reordered=0, and stuck=0 is selected
  CHECK: cargo test --target x86_64-pc-windows-msvc scanner::tests::selects_the_smallest_clean_window_over_ten_thousand_events -- --exact --nocapture
  EXPECT: test result: ok
  EVIDENCE: 10,000 alternating LEFT/RIGHT source snapshots with 4 ms RIGHT transport delay were processed. Windows 1 and 2 ms reordered events; 3, 4, and 5 ms finished with lost=0, duplicate=0, reordered=0, stuck=0. REORDER_WINDOW_MS=3.

- [x] G4: both local LEFT and converted RIGHT updates enter the same bounded reorder queue, sequence gaps/duplicates are detected, and reconciliation clears stale source entries
  CHECK: cargo test --target x86_64-pc-windows-msvc scanner::tests
  EXPECT: test result: ok
  EVIDENCE: central INPUT_STATE receives timestamped local scanner updates and clock-converted right frames, then feeds SnapshotOrderer<32>. Unit tests cover duplicate, gap, and reconciliation clearing; all scanner tests pass.

- [x] G5: formatting, all host tests, host/ARM Clippy, both release builds, protected-range checks, DFU packages, and repository contracts pass
  CHECK: powershell -NoProfile -ExecutionPolicy Bypass -File tools\build-release.ps1
  EXPECT: NocFree release verification passed
  EVIDENCE: build-release.ps1 completed with 71 Rust tests and 18 Python/contract/artifact tests, formatting, host/ARM Clippy, both release builds, UF2 bounds/round trips, and both DFU package checks; output ended NocFree release verification passed.

- [x] G6: the final artifacts are deployed only to their verified LEFT/RIGHT serials and both return with independent DFU/recovery paths intact
  EVIDENCE: right Fn+0 entered F: only after USB DFU serial D82A03513BB02626 was verified; right SHA 453BC8AAB3A762C9A9CBC6A7E6D4159B97FEAE55DE5D79CF1E3C0C075FCBDACE returned as RUST-RIGHT. Left Fn+5 then entered G: only after serial 52CF50988BD1E6EE was verified; left SHA 031F4FE1299F439153A405358B52E5C92E7D3E68B5F0DB803918FE1798699DF3 returned as RUST-LEFT. Right-first preserved the old split command path until right DFU entry.

- [x] G7: real hardware produces correctly ordered stress text in Wired USB and Windows 11 Bluetooth with no stuck keys or objectionable added lag
  EVIDENCE: Korean-layout asdfjkl produced the expected ㅁㄴㅇㄹㅓㅏㅣ basic split mapping. Wired produced jam ten times and a longer rapid jajaja sequence exactly. Windows 11 Bluetooth produced asdfjkljamjamjamjamjamjam exactly. The user reported no stuck key or delay objection in these checks.

- [x] G8: English/Korean docs, current artifact hashes/ranges, automated evidence, real-hardware evidence, and remaining limitations are accurate
  EVIDENCE: README.md, README_ko.md, HANDOFF.md, PROGRESS.md, ROADMAP.md, ROADMAP_ko.md, and RECOVERY.md record the 3 ms selection, clock protocol, 71/18 tests, Wired/Windows 11 Bluetooth scope, role-verified deployment, current hashes/sizes, ranges left 0x27000..0x3acff and right 0x27000..0x329ff, and the unmeasured +8 dBm radio/power effect. Direct metadata-presence and 18 repository/artifact checks pass.

- [x] G9: the verified P4 implementation, artifacts, documentation, and gate ledger are committed
  EVIDENCE: P4 implementation, generated artifacts, automated and hardware evidence, and all required documents were committed as 556cbaf07280b9daca028ea8c52b158f2dd7f030; this gate closure is recorded in the immediately following documentation commit.
