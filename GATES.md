# Gates: P3.3 configured right +8 dBm

OWNS: src/**, tools/**, firmware/**, README.md, README_ko.md, HANDOFF.md, ROADMAP.md, ROADMAP_ko.md, RECOVERY.md, PROGRESS.md, GATES.md

Scope: apply +8 dBm to the right split advertiser and accepted connection. The user explicitly waived additional hardware comparison and asked to proceed to P4, so documents must label this setting hardware-unverified and the keyboard remains on P3.2 until separately flashed.

- [x] G0: this ledger states outcomes that can fail
  CHECK: node "C:\Users\kjh\.codex\skills\unlazy\scripts\gate-lint.mjs" GATES.md
  EXPECT: LINT OK
  EVIDENCE: exit=0; shell=C:\WINDOWS\system32\cmd.exe; cwd=D:\study\nocfree\NocFree-and-rust; path=cca8a72878d1/33 entries; output=WARN  2/5 gates are runnable; a mostly manual ledger is prose with checkboxes  [mostly-manual] | LINT OK (7 warning(s))

- [x] G1: host/contract tests prove +8 dBm applies to both right advertising and accepted connection; the full release build and artifact validation pass
  CHECK: pwsh -NoProfile -ExecutionPolicy Bypass -File tools\build-release.ps1
  EXPECT: NocFree release verification passed
  EVIDENCE: exit=0; shell=C:\WINDOWS\system32\cmd.exe; cwd=D:\study\nocfree\NocFree-and-rust; path=cca8a72878d1/33 entries; output=Ran 17 tests in 0.022s | OK

- [x] G2: only right split TX power changes; P3.2 schedule, left TX, MTU, connection timing, latency, GATT, storage, and protected flash remain unchanged
  EVIDENCE: diff from aca104f adds RIGHT_SPLIT_TX_POWER_DBM=8, TxPower::Plus8dBm to the right advertiser, and the same +8 dBm to the accepted right connection. central.rs, the staged schedule, ATT MTU 23, interval 6, latency 30, timeout 400, GATT, memory.x, storage, and protected boundaries are unchanged.

- [x] G3: English/Korean documents and artifacts state +8 dBm is configured and automatically verified but not hardware-tested or deployed, with current hashes/ranges accurate
  EVIDENCE: README.md, README_ko.md, HANDOFF.md, PROGRESS.md, ROADMAP.md, ROADMAP_ko.md, and RECOVERY.md separate the configured repository artifact from the P3.2 image still on the physical right. Direct SHA-256 and size checks match the current left/right UF2, BIN, and DFU ZIP artifacts; written ranges remain left 0x27000..0x395ff and right 0x27000..0x327ff. Repository tests pass 17/17.

- [x] G4: the configured +8 dBm implementation, artifacts, documentation, and gate ledger are committed before P4 starts
  EVIDENCE: implementation, generated artifacts, documentation, and verification evidence were committed as fc53b9ac85b594380906f07f426756dc8762474a; this final gate closure is recorded in the immediately following documentation commit.
