# DFU and role-specific stock recovery

[English](RECOVERY.md) · [한국어](RECOVERY_ko.md) · [日本語](RECOVERY_ja.md)

NocFree & has no external reset button. Do not short undocumented PCB pads or
assume that reconnecting power is equivalent to a reset-pin double tap. The
Rust firmware deliberately keeps an independent 1200-baud recovery path on
each half and the experimental dongle, in addition to role-specific Fn shortcuts.

## Protected flash ranges

| Range | Purpose | Policy |
|---|---|---|
| `0x00000..0x26fff` | MBR + S140 7.3.0 | Preserve |
| `0x27000..0x64fff` | Rust application | Writable |
| `0x65000..0x67fff` | BLE host profiles | Persisted data |
| `0x68000..0x68fff` | reserved | Preserve |
| `0x69000..0x69fff` | dedicated dongle bond | Persisted data |
| `0x6a000..0x6afff` | selected profile/settings | Persisted data |
| `0x6b000..0x6bfff` | split bond | Persisted data |
| `0x6c000..0x6cfff` | Link keymap/hotkeys | Persisted data |
| `0x6d000..0x73fff` | factory filesystem | Preserve |
| `0x74000..0x7ffff` | Adafruit UF2 bootloader/metadata | Preserve |

The build and UF2 round-trip tests reject applications crossing `0x65000`.
Always match Left, Right, and Dongle artifacts to their exact roles and layout.

## Recovery entry points

- Left Rust CDC opened/touched at 1200 baud: independent left UF2 entry.
- Right Rust CDC opened/touched at 1200 baud: independent right UF2 entry.
- Dongle Rust CDC opened/touched at 1200 baud: independent dongle UF2 entry.
- Dongle Rust CDC opened/touched at 2400 baud: clear its dongle bond and restart.
- Hold `Fn+5` for three seconds: left UF2 entry.
- Hold `Fn+0` for three seconds: right UF2 entry, only while split works.
- `Fn+Esc`: restarts the left application; it is not DFU.
- An unhandled panic stores GPREGRET `0x57` and restarts into UF2.

For left 1200-baud recovery, put the physical switch in the middle **Wired**
position. The serial port may disappear quickly enough that opening or closing
it reports a missing-device error. That can mean reset already succeeded; check
the UF2 volume and bootloader CDC before retrying.

## Identifying the correct half on Windows

COM numbers are not stable. Match the port's `DEVPKEY_Device_Parent` instead.

| Role/state | Parent instance ID |
|---|---|
| Rust left | `USB\VID_2886&PID_8029\RUST-LEFT` |
| Rust right | `USB\VID_1D50&PID_615E\RUST-RIGHT` |
| Experimental Rust dongle | `USB\VID_239A&PID_80D8\RUST-DONGLE` |
| stock left | `USB\VID_2886&PID_8029\52CF50988BD1E6EE` |
| stock right | `USB\VID_239A&PID_80D8\D82A03513BB02626` |
| UF2 CDC left | `USB\VID_239A&PID_0029\52CF50988BD1E6EE` |
| UF2 CDC right | `USB\VID_239A&PID_0029\D82A03513BB02626` |
| serial-DFU CDC left | `USB\VID_239A&PID_002A\52CF50988BD1E6EE` |
| serial-DFU CDC right | `USB\VID_239A&PID_002A\D82A03513BB02626` |

On macOS or Linux, identify the new serial device before and after the reset.
The portable 1200-baud touch forms are typically:

```text
# macOS
stty -f /dev/cu.usbmodemXXXX 1200

# Linux
stty -F /dev/ttyACM0 1200
```

Device names differ by host. Do not run the command until the role is known.

## UF2 bootloader check

`INFO_UF2.TXT` must describe NocFree &, S140 7.3.0, and the NocFree & board.
Both halves show the same volume model, so also verify which CDC disappeared
and that the opposite application is still present. Then copy only the matching
file:

- `firmware/NocFree_And_Rust_ZMK_Based_ANSI_Left.uf2`
- `firmware/NocFree_And_Rust_ZMK_Based_ANSI_Right.uf2`
- `firmware/experimental/NocFree_And_Rust_ZMK_Based_ANSI_Experimental_Dongle.uf2`

The ANSI dongle bootloader and application recovery path passed physical
verification. ISO, JIS, and KR require their matching layout-named Dongle UF2
and matching-keyboard verification. Never copy a mismatched layout or role. A current serial DFU package can be sent
only to a verified `PID_002A` bootloader port:

```text
adafruit-nrfutil dfu serial --package firmware/NocFree_Rust_Right_DFU.zip --port PORT --baudrate 115200
```

`adafruit-nrfutil` has previously printed `No data received on serial port` but
returned exit code 0 when asked to touch a Rust application and transfer in one
operation. Success therefore requires all three: no failure text,
`Device programmed.`, and the expected Rust parent returning.

## Verified stock round trip

Both halves completed this sequence on 2026-08-21:

1. Identify the target Rust parent and CDC; confirm the opposite Rust half is present.
2. Touch only the target CDC at 1200 baud.
3. Verify the NocFree & / S140 UF2 bootloader.
4. Verify and copy the role-specific stock V2.3.0 UF2.
5. Confirm the role-specific stock parent and chip serial.
6. Touch the stock CDC at 1200 baud.
7. Identify the same chip's `PID_002A` serial-DFU CDC.
8. Send the role-specific Rust DFU ZIP.
9. Confirm `RUST-LEFT` or `RUST-RIGHT` and actual key input.

The known stock files are outside this repository:

| Role | File | SHA-256 |
|---|---|---|
| Left | `D:\study\nocfree\NocFree_and_V2.3.0_Left_ANSI.uf2` | `A3FF612B94E9CE0C12BEFD9FF19ECDE5D6E4DB964C0345E432B71ED7A2C5BC91` |
| Right | `D:\study\nocfree\NocFree_and_V2.3.0_Right_ANSI.uf2` | `E1F851906B3E35117B8A8AAC09E5C8273D75921F64D1D3B1496E60A53D3E1C66` |

## Verified dongle serial-DFU round trip

The factory dongle completed an application-only recovery on 2026-08-27.
Its application baseline was `NocFree_Dongle`, `VID_2886&PID_8029`, serial
`E19D2CEA0B437049`, CDC interface `MI_00`, and HID interface `MI_02`.

Opening and closing its application CDC at 1200 baud entered a serial-only
bootloader: `VID_239A&PID_002A`, product `NocFree &`, and interface
`nRF Serial`. Unlike the left/right UF2 recovery, the dongle did not expose a
mass-storage volume. COM numbers changed from COM6 to COM14 during this test;
identify the serial and VID/PID instead of hard-coding either number.

The restored official V2.3.1 package is outside this repository:

| File | SHA-256 | Contents |
|---|---|---|
| `D:\study\nocfree\official_v2_3_1\dongle.zip` | `02C1EE2BB420E374E51AC6B0C0EE7A422796DFDFF0AC2707CA28096A564C0567` | application only, `softdevice_req=0x0123` |

Its application binary SHA-256 is
`5E3AD6CE64F41DB164A83A5FAB88C28F7C92E0EDFBC7DCF99B9D23EC3C9010CD`
and its verified UF2 range is `0x27000..0x388ff`. After the user approved the
deployment, this command completed with `Device programmed.`:

```text
adafruit-nrfutil dfu serial --package D:\study\nocfree\official_v2_3_1\dongle.zip --port COM14 --baudrate 115200
```

The original product, VID/PID, serial, CDC, and HID interfaces all returned.
Factory ESB key input was not tested because both keyboard halves run Rust
firmware; this proves dongle recovery and USB identity, not factory-radio
compatibility.

## Stop immediately if

- the wrong half disappears;
- more than one NocFree UF2 volume makes the role ambiguous;
- layout, role, or SHA-256 does not match;
- the expected Rust parent does not return;
- a key remains stuck, the device gets hot, or neither app nor bootloader appears.

Do not touch the other half after any stop condition. Physical flashing is not
part of an automated build and requires the user's explicit approval.
