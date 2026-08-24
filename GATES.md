# Gates: P3.1 split reconnect without ATT MTU exchange

OWNS: src/**, tools/**, firmware/**, README.md, README_ko.md, HANDOFF.md, ROADMAP.md, ROADMAP_ko.md, RECOVERY.md, PROGRESS.md, GATES.md

Scope: remove the unnecessary split ATT MTU exchange identified by P2, then prove reliable desk-distance reconnect without changing advertising, TX power, or latency.

- [x] G0: this ledger states outcomes that can fail
  CHECK: node "C:\Users\kjh\.codex\skills\unlazy\scripts\gate-lint.mjs" GATES.md
  EXPECT: LINT OK
  EVIDENCE: exit=0; shell=C:\WINDOWS\system32\cmd.exe; cwd=D:\study\nocfree\NocFree-and-rust; path=cca8a72878d1/33 entries; output=WARN  2/10 gates are runnable; a mostly manual ledger is prose with checkboxes  [mostly-manual] | LINT OK (16 warning(s))

- [x] G1: the complete release build and artifact validation prove the default ATT MTU fits every split value and both firmware images remain valid
  CHECK: pwsh -NoProfile -ExecutionPolicy Bypass -File tools\build-release.ps1
  EXPECT: NocFree release verification passed
  EVIDENCE: exit=0; shell=C:\WINDOWS\system32\cmd.exe; cwd=D:\study\nocfree\NocFree-and-rust; path=cca8a72878d1/33 entries; output=Ran 17 tests in 0.022s | OK

- [x] G2: only the split ATT MTU request changes; advertising intervals, TX power, connection interval, latency, supervision timeout, and protected flash boundaries remain unchanged
  EVIDENCE: diff from d665c48 sets only the central split ConnectConfig ATT MTU to 23; CONNECTION_INTERVAL_UNITS=6, CONNECTION_LATENCY=30, CONNECTION_TIMEOUT_UNITS=400, FAST_ADVERTISING_INTERVAL_UNITS=400, IDLE_ADVERTISING_INTERVAL_UNITS=1600, TX-power setup, memory.x, and protected boundaries are unchanged. The right-side edit only records the negotiated MTU in the existing RAM diagnostic ring.

- [x] G3: the left P3.1 UF2 is deployed to the verified left serial and returns as RUST-LEFT while the P2 right remains compatible
  EVIDENCE: left DFU serial 52CF50988BD1E6EE was verified on G:, SHA-256 7BAAA67BE4BB53B577E7591807BAD07CC661573B3630394951EA6A81F29FFF7A was copied, USB VID_2886&PID_8029 returned as RUST-LEFT, and the unchanged P2 right produced jkl (Korean layout: ㅓㅏㅣ) through Wired USB.

- [x] G4: two user-approved right power-off/on cycles at roughly 30 cm reconnect without moving the halves closer, losing input, or requiring another recovery action
  EVIDENCE: the user reduced the requested sample from ten cycles to two and produced jj from the right after both cycles. Diagnostics recorded split-ready on attempts 2 and 3 without an error: RSSI -85/-79 dBm and total reconnect times 55,431/17,736 ms. Reliability passed this limited sample, but reconnect speed remains unproven.

- [x] G5: P3.1 diagnostics show successful reconnect stages without the previous MTU-exchange failure
  EVIDENCE: left diagnostics after deployment recorded RSSI -56 dBm, connected in 2014 ms, ATT MTU=23, security-ok in 61 ms, gatt-ok in 376 ms, and split-ready on attempt 1 in 5946 ms; no MTU-exchange error was recorded.

- [x] G6: right-side input works after split reconnect in both Wired USB output and the existing Windows 11 Bluetooth output
  EVIDENCE: the unchanged P2 right produced jkl (Korean layout: ㅓㅏㅣ) through Wired USB, then produced ㅓㅓㅓ after the user switched the left output to the existing Windows 11 Bluetooth setup and power-cycled the right. The user separately observed the known single-identity profile-name/connect-loop limitation when selecting an office-PC slot from this PC; that limitation is not treated as a split-link failure.

- [x] G7: left 1200-baud CDC recovery still enters the role-specific bootloader and returns to the same P3.1 firmware
  EVIDENCE: with the left switch in Wired, verified RUST-LEFT COM19 entered USB VID_239A&PID_0029 COM9/G: with parent serial 52CF50988BD1E6EE. The same left UF2 SHA-256 7BAAA67BE4BB53B577E7591807BAD07CC661573B3630394951EA6A81F29FFF7A was restored; RUST-LEFT returned and diagnostics again showed ATT MTU=23 and split-ready on attempt 1 in 4,358 ms. Two earlier attempts while the switch remained in Bluetooth reset back to the app without an observed UF2 volume, so Wired is the documented recovery position.

- [x] G8: English and Korean documents, artifacts, hashes, ranges, measured P3.1 evidence, and the remaining P3 experiments are accurate
  EVIDENCE: README.md, README_ko.md, HANDOFF.md, PROGRESS.md, ROADMAP.md, ROADMAP_ko.md, and RECOVERY.md record 65 Rust/17 Python tests, measured P3.1 timings/RSSI, the two-cycle sample limit, the single-identity Windows profile limitation, Wired-position left recovery, current UF2 ranges 0x27000..0x395ff/0x27000..0x326ff, and current hashes. A direct metadata check found every current artifact hash in the required documents; the next experiment is staged advertising/key-triggered fast advertising with TX power and latency unchanged.

- [ ] G9: the verified P3.1 implementation, artifacts, documentation, and gate ledger are committed
  EVIDENCE: pending
