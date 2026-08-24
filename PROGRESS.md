# NocFree Rust progress

Last updated: 2026-08-24 (Asia/Seoul)

## Baseline

- Repository: `https://github.com/jhkim0218/Nocfree-and-ZMK-rust.git`
- Branch: `main`
- Baseline commit: `e983eb0c8e36b29de438b886d35867da379ad5cf`
- Local and `origin/main` matched before P0 documentation changes.
- Worktree was clean before P0 documentation changes.
- No firmware behavior has been changed in P0.
- No firmware has been flashed in P0.
- Full build/test/artifact validation: passed on 2026-08-24.
- Rust tests: 59 passed, 0 failed, 0 ignored.
- Python/contract/artifact tests: 17 passed.
- Formatting, host/ARM Clippy, left/right release builds, BIN/UF2/DFU generation, and artifact validation passed.
- UF2 family: `0x621E937A`; left writes only `0x27000..0x37dff`, right only `0x27000..0x30cff`.

## Open real-hardware regressions reported 2026-08-24

### R1: cross-half ordering

- Intended rapid input: `jam`
- Observed output: `ajm`
- Fast alternating input between the right and left halves can be reordered.
- Previous FIFO/arrival-order tests are historical evidence, not current completion evidence.

### R2: split reconnect sensitivity

- At roughly 30 cm, the right half sometimes does not reconnect.
- Bringing the halves very close allows reconnection.
- After connection, returning to the original distance continues to work.
- The failure stage is not observable yet; advertising, scanning, security, GATT readiness, and connection parameters remain under investigation.

### R3: backlight divergence

- Observed state: left backlight ON, right backlight OFF, while right-side input still works.
- Repeated `Fn+Tab` swaps the two states instead of converging them.
- Relative toggle commands can preserve a permanent inversion.

## Phase status

| Phase | Status | Exit condition |
|---|---|---|
| P0 Baseline and regression capture | Complete | Full build/test/artifact validation passed and regressions are documented |
| P1 Absolute backlight state | Pending | Deliberately divergent halves converge after one synchronization or reconnect |
| P2 BLE reconnect observability | Pending | Scan, connect, security, GATT, and disconnect stages are distinguishable |
| P3 BLE reconnect tuning | Pending | Repeated normal desk-distance reconnect succeeds without moving the halves closer |
| P4 Cross-half ordering | Pending | 10,000+ automated events and USB/BLE hardware stress pass without loss, duplication, reordering, or stuck keys |
| P5 Stability and power | Pending | Long-running wake/reconnect tests pass and left/right power is measured |
| D0 Dongle recovery | Pending | Stock dongle recovery is proven before feature firmware is flashed |
| D1-D4 Rust-native dongle and 2.4G mode | Pending | Unified `RIGHT -> LEFT -> dongle -> PC` path passes HID, reconnect, release, and recovery tests |
| Layout variants | Deferred | ANSI remains stable; non-ANSI mappings come from verified sources and remain hardware-unverified until tested |

## Protected facts

- Left is the central and owns the complete logical keyboard state.
- Right sends input and battery data to left over an encrypted BLE split.
- Current split uses BLE 1M PHY; factory USB dongle/2.4 GHz output is not implemented.
- The 2.4G physical-switch position currently disables HID output.
- Both halves have independent 1200-baud CDC recovery; left `Fn+5` and right `Fn+0` require a three-second hold.
- Current application artifacts preserve the SoftDevice, Rust storage, factory filesystem, and bootloader regions.
- ANSI 84-key is the only hardware-verified layout.
