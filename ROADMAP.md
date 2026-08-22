# Stock-parity roadmap

[한국어](ROADMAP_ko.md)

This roadmap compares the Rust firmware with the local NocFree & ANSI V2.3.0
UF2 images and the official NocFree documentation. A binary-size difference
does not prove that a feature exists, so feature claims below come from
documented or hardware-observed behavior rather than size alone.

## Comparison baseline

All images use nRF52833 family ID `0x621E937A` and start at application address
`0x27000`.

| V2.3.0 image | UF2 bytes | Written range |
|---|---:|---|
| Stock left ANSI | 295,936 | `0x27000..0x4B1FF` |
| Stock right ANSI | 162,304 | `0x27000..0x3ACFF` |
| Stock 2.4 GHz dongle | 143,872 | `0x27000..0x388FF` |
| Rust left ANSI | 134,144 | `0x27000..0x375FF` |
| Rust right ANSI | 78,336 | `0x27000..0x308FF` |

The separate stock dongle image confirms that factory 2.4 GHz support is a
three-firmware system. It cannot be restored by changing only the left and
right Rust images.

## Feature gaps

| Priority | Area | Stock behavior / target | Current Rust state | Completion evidence |
|---|---|---|---|---|
| P0 | Right `P0.05` pin role | PCA9555 `INT`, active low | Configured as an unused output named `_backlight_enable` | Pin is high-impedance input; key scanning and warm recovery still pass |
| P0 | Battery accuracy | Meaningful levels for both 1100 mAh batteries | Eight-sample ADC average, fixed `130/100` divider scale, linear 3.45–4.20 V percentage | DMM error and percentage error are recorded for both halves across a discharge cycle |
| P0 | Status LEDs | Pairing/connected/wired, low battery, charging, and full indications | Not implemented | Stock truth table is captured and reproduced without electrical contention |
| P0 | Power baseline | Approximately two weeks per charge is the published expectation | No measured idle-current or battery-life result | Stock and Rust current are measured on the same half under identical modes |
| P1 | Idle scanning | Wake from PCA9555 `INT` with periodic safety polling | All three expanders are polled every 10 ms | No missed/stuck keys; idle bus activity and current are materially reduced |
| P1 | Backlight timeout | Backlight off after 5 minutes of inactivity | No inactivity timeout | Turns off at 5 minutes and restores on input without losing the first key |
| P1 | Deep sleep | Sleep after 30 minutes; a left-side key wakes after long sleep | No deep sleep or soft-off state | Measured sleep current and reliable wake/reconnect across repeated cycles |
| P1 | Periodic battery manager | Ongoing level and low-battery monitoring | Samples only when `Fn+I` is requested | Periodic filtered readings feed every battery consumer |
| P1 | Battery output paths | `Fn+I` and NocFree Link expose useful battery information | `Fn+I` works; Link returns `0xff`; no standard BLE Battery Service | `Fn+I`, Link, and BLE report consistent values; a missing right half is not shown as 0% |
| P1 | Charging awareness | Charging and fully charged are distinct from discharge percentage | VBUS/charger state is not incorporated | Charging/full states are correct and voltage under charge does not falsely imply 100% |
| P1 | Backlight effects/settings | Static control, automatic behavior, and documented breathing support | Toggle and 20% static steps only | Selected effects work on both halves and persist if exposed in Link |
| P2 | Factory 2.4 GHz dongle | Left, right, and optional numpad communicate through the USB receiver | 2.4G switch position intentionally disables output | Dongle pairing, reconnect, input ordering, latency, recovery, and coexistence pass |
| P2 | NocFree Link completeness | Battery, lighting, power settings, macros/Quick Text, and updater-related paths | Keymap and hotkeys work; other paths are partial or absent | Each advertised Link screen completes without timeout and survives reboot |
| Out of ANSI scope | Numpad | Separate stock-supported component | Not supported by this 84-key ANSI project | Track separately if project scope expands |

## LED work

The porting data identifies these signals:

- Left red charge/low-battery line: `P0.09`. It is shared with charger status.
- Left blue status LED: `P0.10`, active low.
- Right red charge/low-battery line: `P0.17`. It is shared with charger status.

The red lines must be treated as open-drain-style signals: pull low only when
the firmware owns an indication, and release to high impedance while USB power
is present so the charger circuit can drive the line. Never drive these lines
high until the schematic and board revision have been verified.

Before implementing patterns, flash stock V2.3.0 and record both halves in a
truth table for boot, Wired, BLE advertising, BLE pairing, connected, low
battery, charging, fully charged, and link loss. The official manual explicitly
states that blue flashes during pairing and red flashes for low battery, but it
does not define every timing pattern.

## Battery correction plan

### 1. Measure the ADC path first

Temporarily expose raw SAADC counts and calculated millivolts through CDC or a
diagnostic `Fn+I` format. For each half, compare firmware voltage with a DMM at
the battery terminals near 4.20, 4.00, 3.80, 3.65, and 3.50 V. Record whether
USB is disconnected and whether the backlight is on.

Fit a per-hardware-revision gain and offset:

```text
corrected_mV = measured_mV * gain + offset
```

Do not tune the percentage curve until this voltage error is corrected. The
current `130/100` divider factor is a documented starting point, not a complete
calibration.

### 2. Capture a real discharge curve

Fully charge both 1100 mAh batteries, disconnect USB, let them rest, and run a
repeatable workload with a fixed backlight level. Log corrected voltage and
elapsed time or, preferably, discharged mAh from a battery analyzer/power
profiler. Repeat for the left and right halves because their radio roles and
loads differ.

Use a small piecewise lookup table derived from those measurements instead of
the current linear 3.45–4.20 V conversion. Do not copy a generic Li-ion table
without validating it on this cell and board.

### 3. Stabilize the displayed value

- Take samples only while the divider-enable pin is high, then turn it off.
- Reject outliers and apply a small moving average or exponential filter.
- Add hysteresis so the displayed percentage does not jump around a boundary.
- Sample periodically, but slowly enough that measurement itself does not waste
  power; start with 30–60 seconds and adjust from measured current.
- Use an explicit unknown value for a disconnected right half. The current
  initial value of 0 can be mistaken for an empty battery.

### 4. Separate charging from discharge estimation

Battery voltage rises while charging and is not a reliable state-of-charge
estimate. Combine VBUS detection with the released red charger-status line to
distinguish discharging, charging, and full. Mark 100% only after full-charge
status is observed, not merely because voltage reaches 4.20 V.

### 5. Feed every consumer from one reading

One battery manager should supply:

- `Fn+I` left/right output;
- NocFree Link battery replies instead of `0xff`;
- the standard BLE Battery Service for the left/central battery;
- right-battery information over the encrypted split link;
- low-battery LED and power-policy decisions.

## Power optimization order

1. Measure stock and Rust current before changing code: left/right, connected
   and disconnected, backlight off/20%/100%, and USB absent/present.
2. Stop driving right `P0.05` and validate it as the PCA9555 interrupt input.
3. Replace idle 10 ms polling with interrupt wake plus a conservative periodic
   full scan. Keep 3 ms active scans for debounce.
4. Add the 5-minute backlight timeout.
5. Add 30-minute deep sleep with wake from left `P0.31`, right `P0.05`, USB, and
   any required mode-switch source. Verify the first wake key is not lost.
6. Measure BLE advertising/scanning and reconnect current; add backoff only
   where measurements show a significant cost.
7. Re-run latency, rollover, split reconnect, BLE multi-pairing, DFU, and warm
   I2C recovery tests after every power change.

Use the stock image on the same physical unit as the primary current target.
The published two-week lifetime is a useful end-to-end check, but it is too
dependent on backlight and workload to replace direct current measurement.

## Sources

- [Original NocFree community ZMK porting data](https://github.com/NocFreeKB/NocFree-and-zmk)
- [Official NocFree & manual](https://www.nocfree.com/pages/nocfree-and-manual)
- [Official product specifications](https://www.nocfree.com/products/nocfree-and-reservation)
