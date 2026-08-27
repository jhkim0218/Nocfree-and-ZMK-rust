# NocFree-and-rust

[한국어](README_ko.md) · [日本語](README_ja.md)

> [!CAUTION]
> The `develop` branch contains work in progress. The D1 dongle USB and recovery
> shell is hardware-verified, but its wireless input link is not implemented.
> Do not treat the branch as a complete or stable 2.4 GHz release.

An independent `no_std` Rust firmware for the nRF52833-based NocFree & keyboard.
It ports behavior from the original
[`NocFreeKB/NocFree-and-zmk`](https://github.com/NocFreeKB/NocFree-and-zmk)
project; it is not an official NocFree firmware release.

> [!IMPORTANT]
> - The firmware starts in **Windows mode by default**. Hold `Fn+M` for one
>   second for macOS mode; hold `Fn+N` for one second to return to Windows.
> - Flash only the file matching the keyboard half and layout. Never mix left
>   and right files or files from different builds.
> - NocFree & has no external reset button. Read [RECOVERY.md](RECOVERY.md)
>   before flashing so both halves can always return to DFU or stock V2.3.0.
> - D1 provides a radio-free Rust dongle USB/recovery shell only. Actual 2.4 GHz
>   keyboard input is not implemented, so the left 2.4G switch still produces
>   no output.

| Layout | Current status | Physical validation |
|---|---|---|
| ANSI | Default build; 5 ms ordering and 1 kHz perceptual backlight | The current 5 ms build passed wired input and synchronized backlight testing; full BLE regression remains |
| ISO | Experimental | Not tested on matching hardware |
| JIS | Experimental | Not tested on matching hardware |
| KR | Experimental | Not tested on matching hardware |

## Start here

The keyboard is a split system with fixed roles:

- **Left (`central`)** scans 37 keys, receives the right half, owns the complete
  keymap, and sends USB or Bluetooth HID output.
- **Right (`right`)** scans 47 keys and sends them to the left over an encrypted
  BLE split. Its USB port is for recovery and diagnostics, not keyboard HID.

The left physical switch selects the output:

| Position | Mode | Behavior |
|---|---|---|
| Top | 2.4G | No output; factory dongle transport is not implemented |
| Middle | Wired | USB HID from the left port |
| Bottom | Bluetooth | Bluetooth HID from the left half |

The right switch physically controls battery power: up is OFF and down is ON.
USB power bypasses this switch, so the right board remains powered with USB
connected even when the switch is OFF. Wired mode without left-side USB has no
HID output, but it does not power the keyboard off.

The D1 dongle enumerates on Windows 11 as `NocFree Rust Dongle` with keyboard,
consumer-control, and CDC interfaces. It intentionally sends no HID reports
until the dedicated left-to-dongle link is implemented.

For first use:

1. Read [RECOVERY.md](RECOVERY.md) and identify the left and right firmware.
2. Build or download the matching UF2 pair.
3. Flash one half at a time, then confirm both halves over Wired USB.
4. Test Bluetooth, the physical switches, DFU shortcuts, and rapid alternating
   left/right input before relying on the firmware.

## Build and firmware

Install Rust, Python 3, a C/C++ toolchain with libclang, and the packaging tool:

```text
rustup target add thumbv7em-none-eabihf
rustup component add llvm-tools-preview
python3 -m pip install adafruit-nrfutil==0.5.3.post16
```

On Windows, `python` may replace `python3`. On macOS, install LLVM with Homebrew
if libclang is unavailable. On Debian/Ubuntu, install `clang` and
`libclang-dev`. Set `LIBCLANG_PATH` only when automatic discovery fails.

Build and verify ANSI on Windows, macOS, or Linux:

```text
python3 -B tools/build_release.py --layout ANSI
```

Build the separate ANSI D1 dongle image with:

```text
python3 -B tools/build_release.py --layout ANSI --dongle
```

Use `ISO`, `JIS`, or `KR` for an experimental layout, or build all four:

```text
python3 -B tools/build_release.py --all-layouts
```

The Windows PowerShell wrapper remains available:

```powershell
& pwsh -NoProfile -ExecutionPolicy Bypass -File '.\tools\build-release.ps1' -Layout ANSI
```

The builder formats the source, runs host and ARM checks, builds both halves,
creates BIN/UF2/serial-DFU artifacts, and verifies their address and contents.
ANSI output is stored under `firmware`; experimental layouts are stored under
`firmware/experimental`.

Committed ANSI UF2 files:

- [Left/central UF2](firmware/NocFree_And_Rust_ZMK_Based_ANSI_Left.uf2)
- [Right/peripheral UF2](firmware/NocFree_And_Rust_ZMK_Based_ANSI_Right.uf2)
- [D1 dongle UF2](firmware/NocFree_And_Rust_ZMK_Based_ANSI_Dongle_D1.uf2)
- [D1 dongle serial-DFU package](firmware/NocFree_And_Rust_ZMK_Based_ANSI_Dongle_D1_DFU.zip)

The D1 dongle build is intentionally radio-free. Its application-only artifacts
preserve the factory SoftDevice, filesystem, UICR, and UF2 bootloader. The
hardware-tested application BIN SHA-256 is
`B80808F56226FBCB59FC20A39AE8CD297F4099BA18063F650368E284AA864648`.
The committed DFU ZIP is `FA8D3A03A0661C32FB78CAFA8D36505C2C946697895C28D6049272895175F223`;
regenerating it changes ZIP timestamps but not the tested embedded BIN.

Those canonical files use the physically tested 1 kHz perceptual curve. Build
the linear comparison with:

```text
python3 -B tools/build_release.py --layout ANSI --backlight-curve linear
```

The reproducible comparison pair is stored separately:

- [Linear left UF2](firmware/experimental/NocFree_And_Rust_ZMK_Based_ANSI_Linear_Backlight_Experimental_Left.uf2)
- [Linear right UF2](firmware/experimental/NocFree_And_Rust_ZMK_Based_ANSI_Linear_Backlight_Experimental_Right.uf2)

Both curves were tested on ANSI hardware at 1 kHz. Linear made roughly the first
three rising levels distinct; perceptual spread the electrical duty values, but
the upper brightness steps remain visually close. Always flash a matching pair.

All keyboard firmware is Rust `no_std`. Python and `adafruit-nrfutil` run only
on the build computer and are not installed on the keyboard.

## Incomplete or unverified work

| Area | Remaining work |
|---|---|
| Current ANSI regression | Repeat the complete USB/BLE, switch, sleep/wake, reconnect, and recovery matrix; wired input and both 1 kHz backlight curves have been checked |
| ISO/JIS/KR | Software builds pass, but every layout still needs a matching physical keyboard test |
| Split reliability | Measure long-run drift, reconnect-edge input, real BLE jitter, desk-distance recovery, and controlled +8 dBm range/current tradeoffs |
| Battery | Complete a full discharge cycle, compare against a DMM, and measure active/idle/System OFF current and real battery life |
| Status LEDs | Verify red low-battery behavior on a discharged unit and implement factory-equivalent charging/full indications |
| Factory 2.4 GHz/dongle | D1 USB keyboard/consumer/CDC enumeration and recovery are complete; external nRF24L01 communication, left-to-dongle link, pairing, input, and separate numpad communication are not implemented |
| NocFree Link extras | Battery display returns unavailable; Quick Text storage/deletion/execution is not implemented |
| Other tools | Factory updater and ZMK Studio compatibility are not implemented and are not current project requirements |
| Platform coverage | Bluetooth host testing covers Windows 11 and Android only; macOS, iOS, Linux, and a third host remain untested |

The prioritized comparison with stock firmware and battery-calibration procedure
are in [ROADMAP.md](ROADMAP.md). Detailed experiment records belong in
[PROGRESS.md](PROGRESS.md), not in this overview.

## Implemented features

- **ANSI input:** 37 left and 47 right keys, both Fn keys, media/system keys,
  Korean Windows special keys, and encrypted BLE split transport.
- **USB and Bluetooth HID:** physical output selection, automatic BLE
  reconnection, persisted CCCD state, and three persistent pairing slots.
- **NocFree Link:** 8 × 84 keymaps, 16 executable hotkeys, CRC-protected flash
  persistence, deletion, and default restoration through `link.nocfree.com`.
- **Recovery:** independent 1200-baud CDC DFU on both halves, held Fn shortcuts,
  UF2 bootloader entry, and Rust ↔ stock V2.3.0 restoration paths.
- **Backlight:** synchronized on/off and 0/20/40/60/80/100% settings on both
  halves, 1 kHz PWM, a perceptual default curve, an optional reproducible linear
  comparison, 30-second idle off, and first-key wake. The direction is `Fn+F5`
  down and `Fn+F6` up; the upper perceived levels remain close on tested hardware.
- **Power behavior:** interrupt-driven idle scanning with a 250 ms safety scan,
  battery-divider activation only while measuring, and left System OFF after
  five minutes on battery.
- **Battery reporting:** recovered stock V2.3.0 conversion/filter logic and
  `Fn+I` text output. Two fully charged halves reported `L 100 R 100`.
- **Status and diagnostics:** persistent blue pairing indication, automated red
  low-battery logic, and 32 in-memory split events readable from either USB port.
- **Safe flash layout:** application images preserve the SoftDevice, persistent
  storage, factory filesystem, and UF2 bootloader regions.
- **Dongle D1 foundation:** radio-free keyboard/consumer/CDC USB enumeration,
  application-only artifacts, and a hardware-verified 1200-baud UF2/CDC recovery
  round trip. D1 deliberately emits no key reports.

The current 5 ms ANSI firmware passed wired input and synchronized 1 kHz
backlight control on both halves. Previously tested ANSI firmware passed all 84 keys, Wired/Bluetooth output,
Windows 11 and Android multi-pairing, both physical switches, NocFree Link,
backlight synchronization and timeout, power wake, both DFU paths, and stock
restoration. The complete regression matrix has not yet been repeated on the
current artifacts. See [HANDOFF_en.md](HANDOFF_en.md) for the latest handoff status.

## Default shortcuts

Both Fn keys use the same layer. NocFree Link can replace ordinary key actions;
the table below lists firmware defaults.

| Key | Activation | Action |
|---|---|---|
| `Fn+Esc` | Immediate | Restart the left application; not DFU |
| `Fn+F1` / `Fn+F2` | Immediate | Display brightness down / up |
| `Fn+F3` / `Fn+F4` | Immediate | Mission Control/Task View and Spotlight/Search |
| `Fn+F5` / `Fn+F6` | Immediate | Keyboard backlight down / up by 20% on both halves |
| `Fn+F7` / `Fn+F8` / `Fn+F9` | Immediate | Previous / play-pause / next track |
| `Fn+F10` / `Fn+F11` / `Fn+F12` | Immediate | Mute / volume down / volume up |
| `Fn+1` / `Fn+2` / `Fn+3` | Short press | Select Bluetooth pairing slot 1 / 2 / 3 |
| `Fn+1` / `Fn+2` / `Fn+3` | Hold 1 second | Delete that slot's bond and begin new pairing |
| `Fn+5` | Hold 3 seconds | Enter the **left** UF2 bootloader |
| `Fn+0` | Hold 3 seconds | Enter the **right** UF2 bootloader |
| `Fn+Tab` | Immediate | Toggle both backlights |
| `Fn+I` | Hold 3 seconds | Type `L {left %} R {right %}` through the active output |
| `Fn+Delete` | Hold 3 seconds | Compatibility alias for right DFU; prefer `Fn+0` |
| `Fn+M` | Short / hold 1 second | Type M / save macOS mode |
| `Fn+N` | Short / hold 1 second | Type N / save Windows mode |

Unlisted Fn combinations are transparent. `Fn+6` and `Fn+7` do not clear a
profile or switch output; use held `Fn+1/2/3` and the physical switch instead.
`Fn+U` and `Fn+B` type their original letters.

## Technical reference

### Cross-half ordering

Right snapshots carry source time, sequence, and reconciliation metadata. The
left estimates clock offset before split-ready, refreshes it every 60 seconds,
and merges local and remote events through one queue.

The ordering window changed from the tested **3 ms** value, through an **8 ms**
candidate, to the current **5 ms** compromise. Builders can tune
`REORDER_WINDOW_MS` in `src/scanner.rs`: smaller values reduce latency, while
larger values tolerate more BLE transport jitter. Rebuild and flash both halves
together after changing it, then test rapid L-R-L and R-L-R input over USB and
Bluetooth. Compile success alone cannot select a safe value.

### Bluetooth profiles and split connection

Pairing slots 1/2/3 are bonds under one BLE identity, not three independent
keyboards. `NocFree 1`, `NocFree 2`, and `NocFree 3` are advertising names for
the selected slot. Select the slot bonded to the current host; deleting the
Windows device is not the normal switching procedure.

The right split uses 1M BLE, encryption, a 7.5 ms connection interval, latency
30, a four-second supervision timeout, standard ATT MTU 23, staged disconnected
advertising, and configured +8 dBm TX power. These settings improved recovery,
but their range, power, and long-run behavior are still listed above as
unverified.

### Power, diagnostics, and recovery

- Both backlights turn off after 30 seconds without a NocFree key press,
  including while USB is connected.
- On battery, the left enters System OFF after five minutes. USB power and
  active pairing prevent System OFF. Hold a left key through reboot or connect
  USB to wake it.
- The right remains in low-activity System ON so the BLE split can reconnect;
  right-side System OFF is not implemented.
- Read split logs without changing keyboard state:

```powershell
& '.\tools\read-split-diagnostics.ps1' -Role Left
& '.\tools\read-split-diagnostics.ps1' -Role Right
```

Use 115200 baud for diagnostics. The 1200-baud touch is reserved for DFU. Do
not short unverified PCB pads; follow [RECOVERY.md](RECOVERY.md).

## Documentation

- [Recovery and stock restoration](RECOVERY.md)
- [Layout-specific notes](LAYOUTS.md)
- [Missing-feature roadmap and battery calibration](ROADMAP.md)
- [Detailed development and verification history](PROGRESS.md)
- [Continuation handoff](HANDOFF_en.md)

The local `vendor/nrf-softdevice` and `vendor/nrf-softdevice-macro` copies track
upstream commit `b0ac850c0a5a05b8a5aef4f752b48115755b8542`. Their
`README.nocfree.md` files explain the 1M PHY and secure CCCD changes.
