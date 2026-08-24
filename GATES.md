# Gates: P2 BLE reconnect observability

OWNS: src/**, tools/**, firmware/**, README.md, README_ko.md, HANDOFF.md, ROADMAP.md, ROADMAP_ko.md, RECOVERY.md, PROGRESS.md, GATES.md

Scope: distinguish every split reconnect stage on both halves, preserve recovery, document measured hardware evidence, and commit the verified phase.

- [x] G0: this ledger states outcomes that can fail
  CHECK: node "C:\Users\kjh\.codex\skills\unlazy\scripts\gate-lint.mjs" GATES.md
  EXPECT: LINT OK
  EVIDENCE: exit=0; shell=C:\WINDOWS\system32\cmd.exe; cwd=D:\study\nocfree\NocFree-and-rust; path=cca8a72878d1/33 entries; output=WARN  2/8 gates are runnable; a mostly manual ledger is prose with checkboxes  [mostly-manual] | LINT OK (11 warning(s))

- [x] G1: the complete release build and artifact validation succeed
  CHECK: pwsh -NoProfile -ExecutionPolicy Bypass -File tools\build-release.ps1
  EXPECT: NocFree release verification passed
  EVIDENCE: exit=0; shell=C:\WINDOWS\system32\cmd.exe; cwd=D:\study\nocfree\NocFree-and-rust; path=cca8a72878d1/33 entries; output=Ran 17 tests in 0.022s | OK

- [x] G2: both halves expose enough live evidence to distinguish reconnect stages
  EVIDENCE: 2026-08-24 hardware log recorded left scan, RSSI/address, connect, requested/actual parameters, security, GATT, split-ready, disconnect reason, and attempts; right log recorded advertising, disconnected key, connection parameters, and security. A failed attempt was identified as MTU exchange rather than a generic BLE failure.

- [x] G3: right 1200-baud CDC recovery enters its role-specific bootloader and returns to the same firmware
  EVIDENCE: COM18 1200-baud touch enumerated UF2 serial D82A03513BB02626; right UF2 SHA-256 AA21211CE20625ED80EE3307DC2EF147D1EEAF373BE94AECEA24A75BCEEAEDA5 was copied and USB\VID_1D50&PID_615E\RUST-RIGHT returned with status OK.

- [x] G4: left 1200-baud CDC recovery enters its role-specific bootloader and returns to the same firmware
  EVIDENCE: 2026-08-24 COM19 1200-baud touch enumerated UF2 serial 52CF50988BD1E6EE; left UF2 SHA-256 5113421A1BAF1E9C5EE461F41506F5CAED4F3A561CD39E2F7D62C4FF9C8FF875 was copied and USB\VID_2886&PID_8029\RUST-LEFT returned with status OK.

- [x] G5: left and right input still work after both recovery cycles
  EVIDENCE: 2026-08-24 after both 1200-baud recovery/reflash cycles, user entered left qwer followed by right jkl; Korean IME displayed ㅂㅈㄷ거ㅏㅣ, proving both halves reached the host through the restored split.

- [x] G6: English and Korean project documents describe the diagnostic command, measured P2 evidence, remaining reconnect regression, current artifacts, and protected ranges accurately
  EVIDENCE: 2026-08-24 P2_DOCUMENTATION_VERIFIED independently compared all six current artifact sizes/hashes with README.md, README_ko.md, and HANDOFF.md; checked four recovery hashes, diagnostic script syntax, P2 MTU evidence, current UF2 ranges, and git diff whitespace.

- [x] G7: the verified P2 implementation, artifacts, documentation, and gate ledger are committed together
  EVIDENCE: commit 3535f0340e02d984cee3f2df12eb7930d225c2e5 contains the P2 firmware, diagnostics tool, current artifacts, English/Korean documentation, and this gate ledger.
