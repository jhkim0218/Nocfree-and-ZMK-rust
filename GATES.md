# Gates: P3.2 staged split advertising

OWNS: src/**, tools/**, firmware/**, README.md, README_ko.md, HANDOFF.md, ROADMAP.md, ROADMAP_ko.md, RECOVERY.md, PROGRESS.md, GATES.md

Scope: make a disconnected right half easy to rediscover without keeping its BLE availability packets at the fastest interval forever. Do not change ATT MTU, TX power, connection interval, latency, supervision timeout, or protected flash.

- [x] G0: this ledger states outcomes that can fail
  CHECK: node "C:\Users\kjh\.codex\skills\unlazy\scripts\gate-lint.mjs" GATES.md
  EXPECT: LINT OK
  EVIDENCE: exit=0; shell=C:\WINDOWS\system32\cmd.exe; cwd=D:\study\nocfree\NocFree-and-rust; path=cca8a72878d1/33 entries; output=WARN  2/10 gates are runnable; a mostly manual ledger is prose with checkboxes  [mostly-manual] | LINT OK (16 warning(s))

- [x] G1: host tests prove fast -> medium -> idle stage order, exact release intervals/durations, and key-triggered fast reset; the full release build and artifact validation pass
  CHECK: pwsh -NoProfile -ExecutionPolicy Bypass -File tools\build-release.ps1
  EXPECT: NocFree release verification passed
  EVIDENCE: exit=0; shell=C:\WINDOWS\system32\cmd.exe; cwd=D:\study\nocfree\NocFree-and-rust; path=cca8a72878d1/33 entries; output=Ran 17 tests in 0.033s | OK

- [x] G2: only right disconnected advertising policy and its diagnostics change; ATT MTU, TX power, connection interval, latency, supervision timeout, GATT schema, storage, and protected flash remain unchanged
  EVIDENCE: diff from a9089c5 changes only the right disconnected advertising state machine, its pure stage constants/tests, readable stage diagnostics, and repository contract assertions. central.rs, SPLIT_ATT_MTU=23, CONNECTION_INTERVAL_UNITS=6, CONNECTION_LATENCY=30, CONNECTION_TIMEOUT_UNITS=400, peripheral default TX power, GATT definitions, memory.x, storage addresses, and protected boundaries are unchanged.

- [x] G3: a shortened 2-second fast / 3-second medium diagnostic image on the verified right serial records fast -> medium -> idle, and a disconnected right key records a return to fast
  EVIDENCE: right DFU serial D82A03513BB02626 was verified on F: and diagnostic UF2 SHA-256 031A6720B1017FB1728405976DFB0E23C17D5E16FD1B8FD1EB7EA28DEB7C02E1 was deployed. Right diagnostics recorded fast 400 units, medium 800 units 1,805 ms later, idle 1600 units 2,532 ms later, then a disconnected key followed immediately by fast 400 units. The P3.1 left was restored from verified serial 52CF50988BD1E6EE with its unchanged hash.

- [x] G4: the final right image uses 250 ms for 10 seconds, 500 ms for the next 50 seconds, then 1 second indefinitely; it is deployed to the verified right serial and returns as RUST-RIGHT while the P3.1 left remains compatible
  EVIDENCE: host tests assert 400/800/1600 interval units and 1000/5000/none timeout units. Final right UF2 SHA-256 859846B4B119B3469468E6998A2A5292EA436F63771C31D71589C599CB7819A7 was copied only to verified right serial D82A03513BB02626 on F:, returned as RUST-RIGHT, connected from fast advertising in 274 ms, and produced jkl through the unchanged P3.1 left.

- [x] G5: at roughly 30 cm, two user-approved right power cycles restore right input without moving the halves closer, changing Windows state, or taking a recovery action; measured reconnect times are recorded
  EVIDENCE: the user produced jj after two cycles. Cycle 1 reached split-ready in 6,142 ms at RSSI -90 dBm. Cycle 2 connected at -85 dBm but security timed out after 5,231 ms; automatic attempt 5 then reached split-ready in 4,696 ms at -79 dBm, 17,208 ms after the cycle-2 disconnect. Input reliability passed this sample, but distance reconnect speed and weak-signal security remain open.

- [x] G6: right-side input after the final reconnect works through Wired USB and the existing Windows 11 Bluetooth output
  EVIDENCE: final right produced jkl plus jj after Wired reconnect cycles, then produced Korean-layout ㅓ after Bluetooth/Fn+2 and a further right power cycle. The user reported that Bluetooth input arrived after a noticeable delay; no Windows device deletion or re-pairing was used.

- [x] G7: right 1200-baud CDC recovery enters the role-specific bootloader and returns to the same final P3.2 image
  EVIDENCE: COM18 parent USB VID_1D50&PID_615E RUST-RIGHT was verified, 1200-baud touch entered F: with serial D82A03513BB02626, the same final right hash was restored, and RUST-RIGHT returned. Post-recovery diagnostics recorded fast 250 ms advertising, connection in 1,295 ms, ATT MTU 23, and security-ok in 60 ms.

- [x] G8: English and Korean documents, artifacts, hashes, ranges, measured P3.2 evidence, and the remaining P3 experiments are accurate
  EVIDENCE: README.md, README_ko.md, HANDOFF.md, PROGRESS.md, ROADMAP.md, ROADMAP_ko.md, and RECOVERY.md record the schedule, short diagnostic, two-cycle timings/RSSI/security retry, Wired/Bluetooth input, right recovery, 66/17 tests, current hashes/ranges, and TX power as the next single variable. Direct metadata checks matched every current artifact hash in README EN/KO and HANDOFF; repository tests remain 17/17.

- [ ] G9: the verified P3.2 implementation, artifacts, documentation, and gate ledger are committed
  EVIDENCE: pending
