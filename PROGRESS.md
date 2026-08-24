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
- P2 identified an ATT MTU-exchange failure; P3.1 removed that unnecessary exchange. Two desk-distance cycles then succeeded, but 17.7-55.4 second split-ready times leave advertising/discovery speed under investigation.

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
| P3 BLE reconnect tuning | In progress (P3.2 complete) | Standard MTU and staged/key-triggered advertising pass, but one weak-signal security timeout and 6-17 second desk-distance results require TX-power comparison |
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

## P3.1 standard split ATT MTU — complete

- P2 showed that the failing stage was `ConnectError::MtuExchange`. The largest
  split GATT value is 8 bytes, so P3.1 requests standard ATT MTU 23 with a
  20-byte value capacity and skips the unnecessary larger-MTU exchange.
- This experiment did not change advertising, TX power, connection interval
  7.5 ms, latency 30, supervision timeout 4 seconds, or protected flash.
- Automated validation passed: 65 Rust tests, 17 Python/contract/artifact
  tests, formatting, host/ARM Clippy, both release builds, and artifact checks.
- The P3.1 left image was installed while the P2 right remained deployed and
  compatible. Initial diagnostics reached split-ready on attempt 1 with ATT MTU
  23 and no MTU error.
- At roughly 30 cm, the user reduced the requested sample to two right power
  cycles. Both restored right input without moving the halves closer or taking
  a recovery action. Diagnostics measured RSSI `-85/-79 dBm` and split-ready
  times `55,431/17,736 ms`; reliability passed this limited sample, but speed
  did not.
- Right input passed through Wired USB and the existing Windows 11 Bluetooth
  output. A separate Windows profile-name/connect-loop limitation was observed
  when selecting a slot bonded to another host; it is not a split-link failure.
- Left 1200-baud recovery passed with the physical switch in Wired: verified
  `RUST-LEFT` COM19 entered `VID_239A&PID_0029` COM9/G: with serial
  `52CF50988BD1E6EE`, restored the same P3.1 UF2, and returned as `RUST-LEFT`.
  Post-recovery diagnostics reached split-ready on attempt 1 in 4,358 ms with
  ATT MTU 23. Two earlier touches while the switch remained in Bluetooth reset
  back to the app without an observed UF2 volume.
- Next P3 experiment: staged right BLE availability packets (advertising) and
  disconnected-key return to a frequent/fast packet interval. Keep TX power
  and peripheral latency unchanged until this discovery-delay result is measured.

| Role | Repository artifact | Hardware during P3.1 | SHA-256 | Written range |
|---|---|---|---|---|
| Left | `firmware/NocFree_And_Rust_ZMK_Based_ANSI_Left.uf2` | P3.1 deployed and recovery-tested | `7BAAA67BE4BB53B577E7591807BAD07CC661573B3630394951EA6A81F29FFF7A` | `0x27000..0x395ff` |
| Right | `firmware/NocFree_And_Rust_ZMK_Based_ANSI_Right.uf2` | P2 remained deployed; current artifact only improves actual-MTU logging | `A511095812F945E4AE67090B5F33993137FF537060E9ED10B12280F575A250D8` | `0x27000..0x326ff` |

## P3.2 staged advertising — complete

- The right advertises at 250 ms for 10 seconds after disconnect, 500 ms for
  the next 50 seconds, then 1 second indefinitely. A disconnected right key
  immediately restarts the 250 ms stage.
- A shortened 2-second/3-second diagnostic image recorded fast -> medium ->
  idle and disconnected-key -> fast before the final durations were built.
- Automated validation passed: 66 Rust tests, 17 Python/contract/artifact
  tests, formatting, host/ARM Clippy, both release builds, and artifact checks.
- At roughly 30 cm, two user-approved right power cycles restored input in
  6,142 and 17,208 ms. The second connected at -85 dBm but security timed out;
  attempt 5 recovered automatically at -79 dBm.
- Final right input passed as `jkljj` through Wired USB and as a Korean-layout
  `ㅓ` through the existing Windows 11 Bluetooth profile. The user noticed a
  Bluetooth delay; no Windows deletion or re-pairing was used.
- Right 1200-baud recovery verified COM18 parent `RUST-RIGHT`, DFU serial
  `D82A03513BB02626`, restored the same final image, and returned as
  `RUST-RIGHT`. It connected from fast advertising in 1,295 ms and secured in
  60 ms.
- P3 remains open. The next single-variable experiment is right TX power;
  connection latency and timing remain unchanged.

| Role | UF2 | SHA-256 | Written range |
|---|---|---|---|
| Left P3.1 | `firmware/NocFree_And_Rust_ZMK_Based_ANSI_Left.uf2` | `7BAAA67BE4BB53B577E7591807BAD07CC661573B3630394951EA6A81F29FFF7A` | `0x27000..0x395ff` |
| Right P3.2 | `firmware/NocFree_And_Rust_ZMK_Based_ANSI_Right.uf2` | `859846B4B119B3469468E6998A2A5292EA436F63771C31D71589C599CB7819A7` | `0x27000..0x327ff` |

## Protected facts

- Left is the central and owns the complete logical keyboard state.
- Right sends input and battery data to left over an encrypted BLE split.
- Current split uses BLE 1M PHY; factory USB dongle/2.4 GHz output is not implemented.
- The 2.4G physical-switch position currently disables HID output.
- Both halves have independent 1200-baud CDC recovery; left `Fn+5` and right `Fn+0` require a three-second hold.
- Current application artifacts preserve the SoftDevice, Rust storage, factory filesystem, and bootloader regions.
- ANSI 84-key is the only hardware-verified layout.
