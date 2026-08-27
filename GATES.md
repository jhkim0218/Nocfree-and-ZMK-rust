# Gates: D0 dongle recovery

OWNS: GATES.md, RECOVERY.md, RECOVERY_ko.md, RECOVERY_ja.md, PROGRESS.md, PROGRESS_ko.md, PROGRESS_ja.md, ROADMAP.md, ROADMAP_ko.md, ROADMAP_ja.md, tools/verify_dongle_stock.py, tools/test_documentation.py

Scope: prove the factory dongle can enter serial DFU and return to verified stock operation before any Rust feature firmware is flashed.

- [x] G0: this recovery ledger states mechanically valid gates
  CHECK: node C:\Users\kjh\.codex\skills\unlazy\scripts\gate-lint.mjs GATES.md
  EXPECT: LINT OK
  EVIDENCE: exit=0; shell=C:\WINDOWS\system32\cmd.exe; cwd=D:\study\nocfree\NocFree-and-rust; path=735d250c57b9/36 entries; output=WARN  G4: title states a number that nothing measures: "reflashing the preserved factory V2.3.1 application restores the recorded USB identity and interface set"  [unmeasured-number] | LINT OK (4 warning(s))

- [x] G1: the preserved factory dongle images have recorded digests and the recovery package is application-only
  CHECK: python -B tools/verify_dongle_stock.py
  EXPECT: Stock dongle image verification passed
  EVIDENCE: exit=0; shell=C:\WINDOWS\system32\cmd.exe; cwd=D:\study\nocfree\NocFree-and-rust; path=735d250c57b9/36 entries; output=UF2 range: 0x27000..0x388FF | Stock dongle image verification passed

- [x] G2: the connected dongle application identity is captured before recovery
  EVIDENCE: Windows reported `NocFree_Dongle`, USB `VID_2886&PID_8029`, serial `E19D2CEA0B437049`, COM6, and HID interface MI_02 on 2026-08-27 before any DFU command.

- [x] G3: the connected dongle enters its recovery bootloader without modifying protected flash or UICR
  EVIDENCE: A 1200-baud touch of verified stock COM6 entered the same serial `E19D2CEA0B437049` as `VID_239A&PID_002A`, product `NocFree &`, `nRF Serial`, COM14. It exposed serial DFU only; the subsequently verified package manifest contained only an application entry.

- [x] G4: reflashing the preserved factory V2.3.1 application restores the recorded USB identity and interface set
  EVIDENCE: With explicit user approval, `adafruit-nrfutil` sent official V2.3.1 package SHA-256 `02C1EE2BB420E374E51AC6B0C0EE7A422796DFDFF0AC2707CA28096A564C0567` to COM14 at 115200 baud and reported `Device programmed.`. `NocFree_Dongle`, `VID_2886&PID_8029`, serial `E19D2CEA0B437049`, COM6, CDC MI_00, and HID MI_02 all returned.

- [x] G5: the verified recovery procedure, identifiers, hashes, and flash boundaries are documented without claiming Rust dongle support
  CHECK: python -B tools/test_documentation.py && echo Dongle recovery documentation verification passed
  EXPECT: Dongle recovery documentation verification passed
  EVIDENCE: exit=0; shell=C:\WINDOWS\system32\cmd.exe; cwd=D:\study\nocfree\NocFree-and-rust; path=735d250c57b9/36 entries; output=Ran 12 tests in 0.007s | OK
