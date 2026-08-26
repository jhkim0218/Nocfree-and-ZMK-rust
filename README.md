# NocFree-and-rust

[한국어](README_ko.md) · [日本語](README_ja.md)

> [!CAUTION]
> The `develop` branch contains work in progress. Its firmware artifacts have
> passed automated checks but have **not been verified on physical hardware**.

> [!NOTE]
> The cross-half input-ordering window was changed from the tested **3 ms** value,
> through an **8 ms** candidate, to the current **5 ms** compromise. Builders can
> tune `REORDER_WINDOW_MS` in `src/scanner.rs`; rebuild and flash both halves after
> changing it, then test rapid alternating input over both USB and Bluetooth.

This project ports the ZMK behavior of the NocFree & ANSI keyboard to a
`no_std` Rust firmware for the nRF52833. The original project is
[`NocFreeKB/NocFree-and-zmk`](https://github.com/NocFreeKB/NocFree-and-zmk),
and this repository is an independent Rust port.

> [!IMPORTANT]
> The firmware starts in **Windows mode by default**. To use macOS mappings,
> hold `Fn+M` for one second; the choice is saved across reboots. Hold `Fn+N`
> for one second to return to Windows mode. A short press still types M or N.

The 84-key ANSI physical layout remains the only locally hardware-verified target.
Separately selected ISO, JIS, and KR builds are available as **Experimental,
hardware-unverified** variants. See [LAYOUTS.md](LAYOUTS.md) before using them.
The 2026-08-25 refactored ANSI artifact passed all automated checks but was not
flashed today, so it still needs a short ANSI regression pass before release.

| Layout | Firmware status | Physical hardware validation |
|---|---|---|
| ANSI | 5 ms ordering candidate | Previous 3 ms builds were tested; the current 5 ms build is not yet hardware-tested |
| ISO | Experimental | **Not tested on an ISO keyboard** |
| JIS | Experimental | **Not tested on a JIS keyboard** |
| KR | Experimental | **Not tested on a KR keyboard** |

As of 2026-08-23, the latest Rust images are installed on both halves and have
passed hardware tests for USB, BLE, all 84 physical keys, the physical
mode/power switches, shortcuts, NocFree Link key changes, DFU on both halves,
and restoring each half to the stock firmware. New contributors should read
[HANDOFF.md](HANDOFF.md) first.

For the stock-firmware comparison, missing-feature priorities, and battery
calibration procedure, see [ROADMAP.md](ROADMAP.md).

> [!IMPORTANT]
> Battery conversion now follows the recovered stock V2.3.0 algorithm, but a
> full discharge cycle and power-consumption measurements are still pending.
> Blue pairing and red low-battery LED patterns are implemented; charging/full
> and other stock LED states remain incomplete. The factory USB dongle/2.4G
> function is not implemented or verified.
>
> Real-hardware testing on 2026-08-24 reopened rapid cross-half ordering,
> desk-distance split reconnect, and backlight convergence. Absolute backlight
> synchronization has passed hardware testing. P4 timestamp ordering is a
> hardware-tested candidate after limited Wired/Windows 11 checks, not yet an
> end-to-end long-run qualification. Controlled +8 dBm range/power comparison
> remains open. See
> [PROGRESS.md](PROGRESS.md).

## Roles

- **Left (`central`)**: Handles the 37 left-side keys, split input from the
  right half, the complete 84-key keymap, USB/BLE HID, three BLE multi-pairing
  slots, and NocFree Link.
- **Right (`right`)**: Scans the 47 right-side keys and sends them to the left
  half over an encrypted BLE split connection. Right-side USB is an independent
  CDC recovery interface, not a HID interface.

Both halves read three PCA9555 devices (`0x20`, `0x22`, and `0x24`) over SDA
P0.11/SCL P1.09 at 100 kHz. They use active-low inputs, 5 ms debounce, and the
1M BLE PHY. Before TWIM initialization, the firmware sends up to nine bus-clear
clocks followed by STOP so the external expanders do not remain stuck if the
MCU restarts during an I2C transfer from the bootloader or another firmware.

## Build

The same standard-library Python entry point builds on Windows, macOS, and
Linux. Install Rust, Python 3, a C/C++ toolchain with libclang, and the listed
firmware tools first:

```text
rustup target add thumbv7em-none-eabihf
rustup component add llvm-tools-preview
python3 -m pip install adafruit-nrfutil==0.5.3.post16
```

On Windows, use `python` instead of `python3` if that is the installed command.
On macOS, install LLVM with Homebrew if Xcode's tools do not provide libclang;
on Debian/Ubuntu install `clang` and `libclang-dev`. If discovery still fails,
set `LIBCLANG_PATH` to the directory containing the libclang shared library.
Build one layout from the repository root:

```text
python3 -B tools/build_release.py --layout ANSI
```

Use `--layout ISO`, `--layout JIS`, or `--layout KR` for an Experimental
variant, or verify and package every layout with:

```text
python3 -B tools/build_release.py --all-layouts
```

The existing PowerShell command remains a Windows-compatible wrapper:

```powershell
& pwsh -NoProfile -ExecutionPolicy Bypass -File '.\tools\build-release.ps1' -Layout ANSI
```

ANSI writes the stable files under `firmware`; the other layouts write role-
and layout-specific files under `firmware/experimental`. The builder runs
formatting, native host tests, host/ARM Clippy, release builds
for both halves, BIN/UF2/serial-DFU ZIP generation, and checks for address,
family, vector, round-trip, and ZIP-contained BIN consistency. The latest run
passed 74 Rust tests per layout and 27 Python/contract/artifact tests.

After a successful build, use the UF2 for the matching half only:

- **Left/central:** [`firmware/NocFree_And_Rust_ZMK_Based_ANSI_Left.uf2`](firmware/NocFree_And_Rust_ZMK_Based_ANSI_Left.uf2)
- **Right/peripheral:** [`firmware/NocFree_And_Rust_ZMK_Based_ANSI_Right.uf2`](firmware/NocFree_And_Rust_ZMK_Based_ANSI_Right.uf2)

These two files are also committed to this repository and can be downloaded
directly without building. See [RECOVERY.md](RECOVERY.md) before flashing.

All code that runs on the keyboard is Rust `no_std`. The `tools/*.py` files and
the Python package `adafruit-nrfutil` are used only on the PC to create and
verify artifacts; they are not installed on the keyboard.

## Latest artifacts

| File | Size (bytes) | SHA-256 |
|---|---:|---|
| `firmware/NocFree_Rust_Left.bin` | 81,140 | `D00E33E4EF6A6783433F10770850FA4C0D59E08742F8EAAA11AA7C39B613EB7E` |
| [`firmware/NocFree_And_Rust_ZMK_Based_ANSI_Left.uf2`](firmware/NocFree_And_Rust_ZMK_Based_ANSI_Left.uf2) | 162,304 | `3CCA14536448F7E69597FFBB2CA88B5FB3BD9E8186B8B5C7F575788335674B76` |
| `firmware/NocFree_Rust_Left_DFU.zip` | 82,016 | `6960EC1B54358BDC8F3B24A50E12C33D5FABCF2CFED4F65FBF07492839EFC119` |
| `firmware/NocFree_Rust_Right.bin` | 47,564 | `C1C20269444F3A9C55911025F93942325CBEA7BD4E343E3D21369D9BFE9F47AC` |
| [`firmware/NocFree_And_Rust_ZMK_Based_ANSI_Right.uf2`](firmware/NocFree_And_Rust_ZMK_Based_ANSI_Right.uf2) | 95,232 | `1C489F55FB77C2827E1AB2F0BB9F10180045A2534977D10558B523633A5354FC` |
| `firmware/NocFree_Rust_Right_DFU.zip` | 48,446 | `6F576F621B10670691C36DCE088B6C0B901EDE0C23C5B76A4729E7F2A58E09BE` |

Experimental UF2 pairs are committed under [`firmware/experimental`](firmware/experimental).
They passed software and artifact checks but were not flashed or tested on matching
hardware. Never mix halves from different layouts or builds.

The UF2 files write only from the application start at `0x27000` through
`0x3acf3` on the left and `0x329cb` on the right. They preserve the SoftDevice,
storage, factory filesystem, and UF2 bootloader.

## Split reconnect diagnostics

Each half keeps its 32 most recent split events in RAM. Connect the matching
USB cable and read them from Windows PowerShell without changing keyboard state:

```powershell
& '.\tools\read-split-diagnostics.ps1' -Role Left
& '.\tools\read-split-diagnostics.ps1' -Role Right
```

The left log distinguishes scan, advertisement identity/RSSI, connection,
security, GATT discovery, split-ready, disconnect reason, attempts, and active
connection parameters. The right log includes advertising mode/interval,
connection/security, disconnect reason, and keys pressed while disconnected.
Opening the diagnostic port uses 115200 baud; the existing 1200-baud DFU path
remains reserved for recovery.

### P3.1 reconnect result

The P2 failure occurred in an unnecessary ATT MTU exchange. P3.1 requests the
standard ATT MTU 23 for the split connection; its 20-byte value capacity is
already larger than the largest split value, which is 8 bytes. Advertising,
TX power, the 7.5 ms connection interval, latency 30, and the 4-second
supervision timeout were not changed.

At roughly 30 cm, two user-approved right power cycles both reconnected without
moving the halves closer or requiring recovery. Their measured split-ready
times were 55,431 and 17,736 ms at -85/-79 dBm, and neither repeated the MTU
failure. Right-side input passed through Wired USB and the existing Windows 11
Bluetooth output. After left 1200-baud recovery, the first attempt reached
split-ready in 4,358 ms with ATT MTU 23. P3.1 therefore resolves the observed
MTU stage failure, but P3 remains in progress because discovery/reconnect can
still be slow. Here, advertising means the small BLE "I am available" packets
the disconnected right half broadcasts for the left to scan. The next
experiment is staged right advertising with a key press returning immediately
to frequent/fast advertising; TX power and latency stay unchanged
until that result is measured.

### P3.2 advertising result

The disconnected right now advertises every 250 ms for 10 seconds, every
500 ms for the next 50 seconds, and every 1 second afterward. A right key press
while disconnected immediately restarts the 250 ms stage. A shortened
2-second/3-second diagnostic image recorded all three stages and the key reset
before the final durations were built.

At roughly 30 cm, two right power cycles both restored input in 6.142 and
17.208 seconds. The second cycle connected at -85 dBm but timed out during
security, then recovered automatically at -79 dBm. Wired USB and the existing
Windows 11 Bluetooth output both passed, although the user still noticed a
Bluetooth delay. Right 1200-baud recovery restored the same final image and
reconnected in 1.295 seconds. P3.2 is complete, but P3 remains open: the next
controlled comparison is right TX power because weak-signal security still
failed once. Connection latency remains unchanged.

### P3.3 configured TX power

The current P4 right image configures +8 dBm for both disconnected split
advertising and the accepted split connection and was deployed during P4 input
testing. Wired and Bluetooth input passed, but controlled +8 dBm range,
security, current-consumption, and battery comparisons remain unverified.

### P4 global cross-half ordering candidate

RIGHT snapshots now carry source time, sequence, and reconciliation metadata in
one 20-byte ATT value. LEFT estimates the clock offset with three samples before
split-ready, refreshes it every 60 seconds, and holds both local and remote
updates in one 5 ms reorder queue. The earlier synthetic comparison of 1–5 ms
over 10,000 events found 3 ms as the smallest clean window. An 8 ms candidate
was prepared on 2026-08-25 for extra margin; the current compromise is 5 ms.
Real `jam`/`ja` stress passed
through Wired USB and Windows 11 Bluetooth. Runtime queues, long-run drift,
reconnect-edge stress, Android P4, and measured BLE arrival jitter remain before
stable status. The 5 ms setting has not yet been tested on hardware.

Builders can tune `REORDER_WINDOW_MS` in `src/scanner.rs`. A smaller value
reduces cross-half latency; a larger value tolerates more BLE transport jitter.
Always rebuild and flash a matching pair, then test rapid L-R-L and R-L-R input
over both USB and BLE. Do not change this value from compile success alone.

## Changing keys with NocFree Link

The left half provides VID/PID `2886:8029`, product name `NocFree & ANSI`, and
the WinUSB vendor bulk interface recognized by `link.nocfree.com`.

- 8 layers × 84 physical keys
- 16 hotkey slots with actual HID chord execution
- Persistent flash storage with keymap/hotkey CRCs
- Hardware-tested single-key changes, persistence after reboot, hotkey
  creation/deletion, and default restoration

ZMK Studio is not implemented; NocFree Link was selected from the two requested
configuration paths. Quick Text queries return empty slots to prevent web-app
timeouts, but storing and executing Quick Text is not implemented. Link battery
queries currently return `0xff` (unavailable), so use `Fn+I` below to check the
batteries.

## Implementation status

| Status | Feature | Current scope and limitations |
|---|---|---|
| Complete | 84-key ANSI input | 37 left-side and 47 right-side keys, Fn on both halves, non-text keys, and Korean Windows special keys tested on hardware |
| Complete | USB/BLE HID | Left-side USB HID, BLE HID, immediate CCCD save/restore, USB↔BLE switching, and BLE automatic reconnection with the same image |
| P4 software candidate / P3 tuning partial | Split connection and ordering | Source timestamps, right sequence, three-sample/periodic clock sync, and a 5 ms global queue. The earlier 3 ms value passed limited Wired/Windows 11 Bluetooth stress; the current 5 ms value is automated-test-only. End-to-end queue loss, real BLE jitter/drift, reconnect-edge stress, and controlled +8 dBm power/range remain unverified |
| Complete | BLE multi-pairing | Three host bond slots with persistent selection. Slots 1 and 2 were paired with Windows 11 and Android; a third host and other operating systems are untested |
| Software candidate | Backlight | `Fn+F5` decreases and `Fn+F6` increases both halves through 0/20/40/60/80/100% settings. A 10 kHz PWM and perceptual duty curve make the six settings distinct; the new curve still needs hardware confirmation. Absolute synchronization, 30-second timeout, and first-key wake remain supported |
| Complete | Physical switches | Left-side Wired/Bluetooth selection and safe no-output behavior at 2.4G; right-side physical power switch |
| Complete | NocFree Link keymap | 8×84 keys, 16 hotkeys, execution/deletion/default restoration, and flash storage with CRCs |
| Complete | Recovery | Independent 1200-baud CDC DFU on both halves, Fn DFU shortcuts, and Rust↔stock V2.3.0 round trips |
| Partial | Battery | Both halves use the recovered stock V2.3.0 ADC/divider conversion, 75/25 voltage filter, 2.31–3.30 V percentage curve, 60-second sampling, and `Fn+I` output. Both fully charged halves reported 100%; a complete discharge cycle and DMM validation remain |
| Partial | NocFree Link compatibility | Keymaps and hotkeys work. Link battery display is unsupported; Quick Text supports empty queries only, not storage, deletion, or execution |
| Partial | Stock power management | The battery divider is enabled only while measuring. Idle key scanning waits on the left `P0.31`/right `P0.05` PCA9555 interrupt lines with a 250 ms safety scan. On battery, the left central enters System OFF after five minutes without a NocFree key press; USB power blocks System OFF but not the 30-second backlight timeout. The right stays in System ON idle so it can reconnect, changing disconnected advertising from 250 ms to 1 second after its first split connection. Charging state, measured sleep current, and stock-equivalent battery life remain unverified |
| Not implemented | Factory USB dongle / 2.4 GHz communication | Dongle pairing and input are not functional or verified. The factory USB receiver, left external nRF24L01, and ESB links to the right half/separate numpad are unused; the current split connection is BLE-only |
| Partial | LEDs beside the physical switches | The left blue LED flashes during pairing until bonding or profile selection, and both red shared charger lines use open-drain-style 0.5-second low-battery flashing at 10% or below. Blue pairing was hardware-tested; red low-battery flashing passed automated tests but could not be observed with both batteries full. Charging/full and other stock indications remain |
| Not implemented | Other configuration paths | Full compatibility with ZMK Studio and the factory firmware updater |

## Power and wake behavior

- After 30 seconds without a NocFree key press, both backlights turn off even when USB power is connected. The next key press wakes the backlights and is normally reported.
- On battery power, the left central enters nRF52 System OFF after five minutes without a NocFree key press. It first requests both backlights to turn off. USB power and active BLE pairing prevent System OFF.
- Wake the sleeping left half by connecting USB or holding a key on the left half until the keyboard reconnects. Because System OFF wake resets the MCU, a very short wake-key tap can be consumed during boot.
- The right half remains in low-activity System ON so it can automatically reconnect over the BLE-only split. After its first split connection, disconnected advertising is slowed from 250 ms to 1 second. Right-half System OFF and the resulting current reduction are not claimed or verified.

## Default shortcuts

The left and right Fn keys use the same layer. This table lists the firmware
defaults before any changes made with NocFree Link.

| Key | Activation | Action |
|---|---|---|
| `Fn+Esc` | Immediate | Restart the left application; this is not DFU |
| `Fn+F1` / `Fn+F2` | Immediate | Display brightness down / up |
| `Fn+F3` | Immediate | macOS: Mission Control; Windows: Task View |
| `Fn+F4` | Immediate | macOS: Spotlight; Windows: Search |
| `Fn+F5` / `Fn+F6` | Immediate | Backlight brightness down / up by 20% on both halves |
| `Fn+F7` / `F8` / `F9` | Immediate | Previous track / play-pause / next track |
| `Fn+F10` / `F11` / `F12` | Immediate | Mute / volume down / volume up |
| `Fn+1` / `Fn+2` / `Fn+3` | Short press | Select BLE pairing slot 1 / 2 / 3 |
| `Fn+1` / `Fn+2` / `Fn+3` | Hold 1 second | Delete the selected slot's bond and start pairing a new host |
| `Fn+5` | Hold 3 seconds | Enter the **left** UF2 bootloader; a short press does nothing |
| `Fn+0` | Hold 3 seconds | Enter the **right** UF2 bootloader; a short press does nothing |
| `Fn+Tab` | Immediate | Toggle the backlight on both halves |
| `Fn+I` | Hold 3 seconds | Type `L {left percentage} R {right percentage}` through the active output; a short press does nothing |
| `Fn+Delete` | Hold 3 seconds | Compatibility alias for **right** DFU; use `Fn+0` for new workflows |
| `Fn+M` | Short press / hold 1 second | Type M on a short press / persist macOS mode on hold |
| `Fn+N` | Short press / hold 1 second | Type N on a short press / persist Windows mode on hold |

An unlisted `Fn+key` is transparent and types the original key. In particular,
the baseline ZMK positions `Fn+6` (clear active profile) and `Fn+7` (toggle
USB/BLE) are not implemented at those positions in Rust. Hold the target
`Fn+1/2/3` for one second to clear and pair a slot, and use the left physical
switch to select the output. `Fn+U` and `Fn+B` have no output-switching behavior
and type their original letters. DFU was changed from immediate activation in
the baseline ZMK behavior to a three-second hold to prevent accidental entry.

Use each half's CDC 1200-baud touch for left-side DFU or right-side DFU while
the split connection is down. The NocFree & has no external reset button. Do
not short unverified PCB pads. See [RECOVERY.md](RECOVERY.md) for the exact
procedure.

## Physical switches

The left three-position switch uses active-low inputs on P0.15/P0.17. Position
meanings were verified against the
[official NocFree & manual](https://www.nocfree.com/pages/nocfree-and-manual).

- Top, **2.4G**: Safely blocks USB and BLE output because the 2.4G transport is
  not implemented yet
- Middle, **Wired**: Selects USB HID output
- Bottom, **Bluetooth**: Selects BLE HID output

During a transition where both sensing pins are low, the previous output is
kept. The mode changes only after the same position remains stable for 20 ms.
If left-side USB is disconnected in Wired mode, there is no HID output, but the
switch does not turn off the keyboard; the MCU and scanner continue consuming
battery power.

The right switch is not a firmware input; it physically switches the battery
power line. Up is OFF and down is ON. When right-side USB is connected, VBUS
bypasses the battery switch, so the board remaining powered while switched OFF
is expected hardware behavior.

## Hardware verification

| Requirement | Evidence from the latest images |
|---|---|
| USB input and ordering from both halves | Historical tests passed, but `jam -> ajm` was reproduced on 2026-08-24; ordering is reopened |
| All 84 physical keys | Automated sweep of 50 character keys and 33 non-text keys, plus manual Print Screen verification |
| Korean Windows special keys | Verified the Right Alt `KanaMode` mapping and OS handling of Print Screen |
| BLE | Passed fresh pairing `freshcapture` → Wired `freshwired2` → BLE `reconnectcapture` without deleting or re-pairing the device |
| BLE multi-pairing | Pairing and connection verified with slot 1 on Windows 11 and slot 2 on Android |
| Left physical mode switch | Passed Wired `wiredswitchok`, BLE `bleswitchok`/`bleagainok`, and no output at 2.4G with `24silentok` |
| Right physical power switch | With right-side USB disconnected: ON produced `jkluiop`, OFF produced no input, and returning to ON produced `jkluiop` |
| Link | Passed A→B, Shift+B hotkey, persistence after reboot, deletion, and restoring default A |
| Stock restoration on both halves | Passed Rust→stock V2.3.0→serial DFU→same Rust→input |
| Right DFU shortcut | Passed `Fn+Delete`→UF2 detection→latest Rust→`jkluiop` without a power cycle |
| Media | Passed mute/unmute and volume down/up |
| Battery | After restoring the stock V2.3.0 conversion and filter, `Fn+I` held for three seconds typed `L 100 R 100` on two fully charged halves; discharge behavior remains a long-running hardware test |
| Status LEDs | Left blue LED continued flashing after releasing a held `Fn+3`, then stopped when a short `Fn+1` restored the bonded Windows slot. Red low-battery flashing was not physically observable because both halves were above 10% |
| Interrupt-driven idle scan | After three seconds idle, both halves immediately detected the first key, repeated held keys, stopped on release, and preserved mixed left/right input order. A 250 ms safety scan remains as a missed-interrupt fallback |
| Backlight synchronization and auto-off | Absolute state converged after a deliberate right reboot while left remained manually off; 10 toggles stayed aligned, 30-second auto-off affected both halves, and a right key woke both |
| Split reconnect diagnostics | At roughly 30 cm, logs captured `-75/-74 dBm`, one MTU-exchange failure, the next successful connection/security/GATT path, HCI disconnect reason `0x08`, and a right key pressed while disconnected. Both halves also passed 1200-baud recovery and returned to working input |
| P3.1 split reconnect | ATT MTU 23 removed the observed MTU-exchange stage failure. Two user-approved right power cycles at roughly 30 cm both recovered input without moving the halves, but took 55,431/17,736 ms; Wired USB and Windows 11 Bluetooth output passed. Left 1200-baud recovery in Wired mode restored the same P3.1 image and reached split-ready in 4,358 ms |
| P3.2 staged advertising | A short diagnostic proved 250/500/1,000 ms stages and disconnected-key return to 250 ms. Final 10/50-second durations passed two desk-distance input cycles in 6.142/17.208 seconds, Wired/Bluetooth output, and right 1200-baud recovery. One -85 dBm security timeout and noticeable Bluetooth delay keep P3 open for TX-power comparison |
| P4 cross-half ordering | Compared 1–5 ms over 10,000 delayed alternating snapshots; 1/2 ms reordered and 3/4/5 ms were clean, so 3 ms was selected. Wired `jam`×10 and rapid `ja`, plus Windows 11 Bluetooth `asdfjkljam…`, passed without reordering or stuck keys. Both role-specific Fn DFU paths deployed and returned the matching images |
| System OFF and split recovery | A 10-second diagnostic image blocked System OFF while USB was present, entered left-central System OFF on battery, shut off both backlights before sleeping, woke from left USB or a held left key, restored BLE, and accepted right-side input after split reconnect. Release images change only the System OFF timeout to five minutes. A short wake-key tap can be consumed by reset boot; hold the left key until reconnect if its character is required |
| New DFU shortcuts | Verified left DFU with `Fn+5`, no action on short `Fn+5`, right DFU after holding `Fn+0` for three seconds, and restoration to the latest images |
| Physical-switch-only output selection | Pressing `Fn+U` and `Fn+B` typed `ub` without changing the output |

BLE host hardware testing was performed **only on Windows 11 and Android**.
macOS, iOS, Linux, and a third host have not been tested. `NocFree 1`/`2`/`3`
are advertising names for the selected pairing slot, not separate Bluetooth
identities. On the same Windows 11 PC, the existing device name can therefore
appear to change with the selected slot. New pairing in an empty slot was
verified with another host, Android.

Selecting a slot bonded to another host can make Windows rename the same device
entry and repeatedly alternate between Paired and Connected while that other
host's slot is active. This is a limitation of using one BLE identity for three
bond slots, not three separately discoverable keyboards. Return to the slot
bonded to the current host; deleting and re-pairing is not the intended profile
switch procedure.

For the latest image verification, the existing slot 1 bond and Windows device
were each deleted once before fresh pairing. The same bond was then switched
Bluetooth→Wired→Bluetooth, and HID input recovered immediately without deleting
the Windows device or pairing again. The firmware stores Windows CCCD system
attributes in RAM/flash as soon as it receives the CCCD write and restores them
after reconnection security completes, before sending notifications.

## Suggested reading order

1. `src/keymap.rs`, `src/link_keymap.rs`
2. `src/scanner.rs`, `src/pca9555.rs`, `src/hardware_scanner.rs`
3. `src/report.rs`
4. `src/bin/right.rs`, `src/split_ble.rs`
5. `src/bin/central.rs`, `src/link_usb.rs`, `src/link_protocol.rs`
6. `src/platform.rs`, `src/bond_store.rs`, `src/bond_record.rs`

The `vendor/nrf-softdevice` and macro directories are local copies of upstream
commit `b0ac850c0a5a05b8a5aef4f752b48115755b8542`. Their respective
`README.nocfree.md` files document the reasons for the 1M PHY and secure CCCD
changes.
