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

### R3: backlight divergence — resolved in P1

- Observed state: left backlight ON, right backlight OFF, while right-side input still works.
- Repeated `Fn+Tab` swaps the two states instead of converging them.
- Relative toggle commands can preserve a permanent inversion.
- P1 replaced relative split commands with left-owned absolute state and passed the hardware checks below.

## Phase status

| Phase | Status | Exit condition |
|---|---|---|
| P0 Baseline and regression capture | Complete | Full build/test/artifact validation passed and regressions are documented |
| P1 Absolute backlight state | Complete | Automated and hardware tests passed; deliberate divergence converged after reconnect |
| P2 BLE reconnect observability | Complete | Both halves expose stage-specific logs; hardware captured an MTU-exchange failure and the following successful attempt |
| P3 BLE reconnect tuning | Pending | Repeated normal desk-distance reconnect succeeds without moving the halves closer |
| P4 Cross-half ordering | Pending | 10,000+ automated events and USB/BLE hardware stress pass without loss, duplication, reordering, or stuck keys |
| P5 Stability and power | Pending | Long-running wake/reconnect tests pass and left/right power is measured |
| D0 Dongle recovery | Pending | Stock dongle recovery is proven before feature firmware is flashed |
| D1-D4 Rust-native dongle and 2.4G mode | Pending | Unified `RIGHT -> LEFT -> dongle -> PC` path passes HID, reconnect, release, and recovery tests |
| Layout variants | Deferred | ANSI remains stable; non-ANSI mappings come from verified sources and remain hardware-unverified until tested |

## P1 absolute backlight — complete

- Left owns `enabled`, `percent`, `timed_out`, and wrapping `generation`.
- Split backlight synchronization uses a versioned four-byte absolute state and encrypted GATT write-with-response.
- Relative split toggle/brightness/idle/wake commands were removed.
- The current complete state is sent after every split GATT discovery, so a rebooted or reconnected right half can converge.
- Host tests: 62 passed; Python/contract/artifact tests: 17 passed.
- Formatting, host/ARM Clippy, and left/right release builds passed.
- Both P1 images were flashed right first and then left. `RUST-RIGHT` and `RUST-LEFT` re-enumerated normally.
- New right with the previous left preserved right-side input before the left upgrade.
- Left `asdf` plus right `jkl` input passed after both upgrades.
- With left canonical state manually off, power-cycling the right created a reboot/default mismatch that converged back to off after reconnect.
- Right input worked after that reconnect while manual off remained off.
- Ten `Fn+Tab` toggles stayed aligned.
- The 30-second timeout turned off both halves; the next right key woke both.

| Role | UF2 | SHA-256 | Written range |
|---|---|---|---|
| Right first | `firmware/NocFree_And_Rust_ZMK_Based_ANSI_Right.uf2` | `B916485001857FD5C18157C4747054F1C2F4831BEC30D3EFA560E5794B340F34` | `0x27000..0x30cff` |
| Left second | `firmware/NocFree_And_Rust_ZMK_Based_ANSI_Left.uf2` | `4BBEB3B1DCC9C8D615E0CB2F6555F9882FBDBE1BE76A3CF61BDDB8B419468A1A` | `0x27000..0x37fff` |

## P2 BLE reconnect observability — complete

- Each half retains the most recent 32 split events in a fixed-size RAM ring.
- Opening the role-specific CDC port at 115200 baud with
  `tools/read-split-diagnostics.ps1 -Role Left|Right` dumps a snapshot; 1200
  baud remains the independent DFU trigger.
- Left records timestamped scan attempts, advertisement identity/RSSI,
  connect request/result, requested and active parameters, security, GATT,
  split-ready latency, and HCI disconnect reason.
- Right records advertising mode/interval, connection parameters, security,
  disconnect reason, and key activity while disconnected.
- Automated validation passed: 64 Rust tests, 17 Python/contract/artifact
  tests, formatting, host/ARM Clippy, both release builds, and artifact checks.
- Hardware at roughly 30 cm captured RSSI `-75/-74 dBm`. Attempt 2 failed
  during MTU exchange; attempt 3 reached split-ready in 2,724 ms. This proves
  the diagnostic exit condition but does not resolve reconnect reliability.
- A right key press while disconnected and HCI disconnect reason `0x08` were
  captured. Both subsequent right-side key presses reached the host.
- Right and left 1200-baud recovery were exercised independently using UF2
  serials `D82A03513BB02626` and `52CF50988BD1E6EE`, then restored to the same
  P2 images. Left `qwer` and right `jkl` worked afterward.

| Role | UF2 | SHA-256 | Written range |
|---|---|---|---|
| Right | `firmware/NocFree_And_Rust_ZMK_Based_ANSI_Right.uf2` | `AA21211CE20625ED80EE3307DC2EF147D1EEAF373BE94AECEA24A75BCEEAEDA5` | `0x27000..0x326ff` |
| Left | `firmware/NocFree_And_Rust_ZMK_Based_ANSI_Left.uf2` | `5113421A1BAF1E9C5EE461F41506F5CAED4F3A561CD39E2F7D62C4FF9C8FF875` | `0x27000..0x394ff` |

## Protected facts

- Left is the central and owns the complete logical keyboard state.
- Right sends input and battery data to left over an encrypted BLE split.
- Current split uses BLE 1M PHY; factory USB dongle/2.4 GHz output is not implemented.
- The 2.4G physical-switch position currently disables HID output.
- Both halves have independent 1200-baud CDC recovery; left `Fn+5` and right `Fn+0` require a three-second hold.
- Current application artifacts preserve the SoftDevice, Rust storage, factory filesystem, and bootloader regions.
- ANSI 84-key is the only hardware-verified layout.
