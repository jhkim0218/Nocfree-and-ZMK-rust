# NocFree-and-rust

[한국어](README_ko.md)

This project ports the ZMK behavior of the NocFree & ANSI keyboard to a
`no_std` Rust firmware for the nRF52833. The original project is
[`NocFreeKB/NocFree-and-zmk`](https://github.com/NocFreeKB/NocFree-and-zmk),
and this repository is an independent Rust port.

This firmware supports **only the 84-key NocFree & ANSI model**. Other NocFree
models and ISO layouts are not supported.

As of 2026-08-21, the latest Rust images are installed on both halves and have
passed hardware tests for USB, BLE, all 84 physical keys, the physical
mode/power switches, shortcuts, NocFree Link key changes, DFU on both halves,
and restoring each half to the stock firmware. New contributors should read
[HANDOFF.md](HANDOFF.md) first.

For the stock-firmware comparison, missing-feature priorities, and battery
calibration procedure, see [ROADMAP.md](ROADMAP.md).

> [!IMPORTANT]
> Battery percentages are not calibrated yet, control of the status LEDs beside
> the physical switches does not accurately reproduce the stock firmware, and
> the factory USB dongle/2.4G function is not implemented or verified.

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

After cloning the repository, open Windows PowerShell in the repository root.
The Rust MSVC toolchain and Python 3 are required. The commands below install
the additional build dependencies and create both UF2 files.

```powershell
$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

& rustup target add thumbv7em-none-eabihf
$targetExit = $LASTEXITCODE
if ($targetExit -ne 0) { throw ('rustup target add failed with exit code {0}' -f $targetExit) }

& rustup component add llvm-tools-preview
$componentExit = $LASTEXITCODE
if ($componentExit -ne 0) { throw ('rustup component add failed with exit code {0}' -f $componentExit) }

& python -m pip install 'adafruit-nrfutil==0.5.3.post16'
$pipExit = $LASTEXITCODE
if ($pipExit -ne 0) { throw ('dependency installation failed with exit code {0}' -f $pipExit) }

& pwsh -NoProfile -ExecutionPolicy Bypass -File '.\tools\build-release.ps1'
$buildExit = $LASTEXITCODE
if ($buildExit -ne 0) { throw ('build failed with exit code {0}' -f $buildExit) }
```

The script runs formatting, Windows host tests, host/ARM Clippy, release builds
for both halves, BIN/UF2/serial-DFU ZIP generation, and checks for address,
family, vector, round-trip, and ZIP-contained BIN consistency. The latest run
passed 52 Rust tests and 17 Python/contract/artifact tests.

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
| `firmware/NocFree_Rust_Left.bin` | 66,884 | `7EDCCD0259F2040DA9CF04DAA1B95C6945B7882414BA37AC680C3C8004443F22` |
| [`firmware/NocFree_And_Rust_ZMK_Based_ANSI_Left.uf2`](firmware/NocFree_And_Rust_ZMK_Based_ANSI_Left.uf2) | 134,144 | `F4583B2532FF5CA75B3EC57BDCADCC43418787335FF8CA4840C23614BCF54144` |
| `firmware/NocFree_Rust_Left_DFU.zip` | 67,760 | `BD990075FA73D94A2CF7A5BC064972AB42425602C1559A5D731EA72983C8FEC5` |
| `firmware/NocFree_Rust_Right.bin` | 39,076 | `A20B48DD7782319C6006B8590E473936D2FB6EA95B504CF053194836762F591E` |
| [`firmware/NocFree_And_Rust_ZMK_Based_ANSI_Right.uf2`](firmware/NocFree_And_Rust_ZMK_Based_ANSI_Right.uf2) | 78,336 | `094C7BEB0722C34F97C9C2E4247C6B4CD73C3D81BF944C052A6CEB97D105909D` |
| `firmware/NocFree_Rust_Right_DFU.zip` | 39,958 | `4E15CF2A159B267F537A28769A8C010FDFCFE164002116852576A73DA947993C` |

The UF2 files write only from the application start at `0x27000` through
`0x375ff` on the left and `0x308ff` on the right. They preserve the SoftDevice,
storage, factory filesystem, and UF2 bootloader.

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
| Complete | Split connection | Encrypted BLE transport of right-side input and battery data to the left, with automatic recovery after link loss |
| Complete | BLE multi-pairing | Three host bond slots with persistent selection. Slots 1 and 2 were paired with Windows 11 and Android; a third host and other operating systems are untested |
| Complete | Backlight | Synchronized white backlight on both halves, toggle, and 20% brightness steps |
| Complete | Physical switches | Left-side Wired/Bluetooth selection and safe no-output behavior at 2.4G; right-side physical power switch |
| Complete | NocFree Link keymap | 8×84 keys, 16 hotkeys, execution/deletion/default restoration, and flash storage with CRCs |
| Complete | Recovery | Independent 1200-baud CDC DFU on both halves, Fn DFU shortcuts, and Rust↔stock V2.3.0 round trips |
| Partial | Battery | ADC measurement on both halves and `Fn+I` output work. Percentage-curve calibration against measured full/empty cells remains; the current conversion is linear from 3.45 to 4.20 V |
| Partial | NocFree Link compatibility | Keymaps and hotkeys work. Link battery display is unsupported; Quick Text supports empty queries only, not storage, deletion, or execution |
| Partial | Stock power management | The battery divider is enabled only while measuring. Stock-equivalent idle current, deep sleep, charging, and low-voltage thresholds have not been measured and verified |
| Not implemented | Factory USB dongle / 2.4 GHz communication | Dongle pairing and input are not functional or verified. The factory USB receiver, left external nRF24L01, and ESB links to the right half/separate numpad are unused; the current split connection is BLE-only |
| Not implemented | LEDs beside the physical switches | Their indication and control do not accurately match the stock firmware. Stock behavior of the red charge/low-voltage LED and blue status LED remains to be reproduced |
| Not implemented | Other configuration paths | Full compatibility with ZMK Studio and the factory firmware updater |

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
| USB input and ordering from both halves | Passed `asdfjkljamjamjam`, the full character layout, and alternating `jam` repetitions |
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
| Battery | `Fn+I` held for three seconds typed `L 24 R 11`; full-charge percentage calibration is still required |
| New DFU shortcuts | Verified left DFU with `Fn+5`, no action on short `Fn+5`, right DFU after holding `Fn+0` for three seconds, and restoration to the latest images |
| Physical-switch-only output selection | Pressing `Fn+U` and `Fn+B` typed `ub` without changing the output |

BLE host hardware testing was performed **only on Windows 11 and Android**.
macOS, iOS, Linux, and a third host have not been tested. `NocFree 1`/`2`/`3`
are advertising names for the selected pairing slot, not separate Bluetooth
identities. On the same Windows 11 PC, the existing device name can therefore
appear to change with the selected slot. New pairing in an empty slot was
verified with another host, Android.

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
