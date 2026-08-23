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
| Rust left ANSI | 136,192 | `0x27000..0x379FF` |
| Rust right ANSI | 80,384 | `0x27000..0x30CFF` |

The separate stock dongle image confirms that factory 2.4 GHz support is a
three-firmware system. It cannot be restored by changing only the left and
right Rust images.

## Feature gaps

| Priority | Area | Stock behavior / target | Current Rust state | Completion evidence |
|---|---|---|---|---|
| Done | Right `P0.05` pin role | PCA9555 `INT`, active low | The erroneous output was removed and the pin now wakes the right scanner | Key scanning and warm recovery pass after the pin change |
| P0 | Battery accuracy | Meaningful levels for both 1100 mAh batteries | Recovered stock V2.3.0 conversion and 75/25 filter; both fully charged halves reported 100% | DMM error and percentage error are recorded for both halves across a discharge cycle |
| P0 | Status LEDs | Pairing/connected/wired, low battery, charging, and full indications | Blue pairing and open-drain red low-battery flashing are implemented. Charging/full and other patterns remain | Blue pairing passes hardware tests; red flashing is observed below 10%; remaining stock truth table is captured without electrical contention |
| P0 | Power baseline | Approximately two weeks per charge is the published expectation | No measured idle-current or battery-life result | Stock and Rust current are measured on the same half under identical modes |
| Done | Idle scanning | Wake from PCA9555 `INT` with periodic safety polling | Left `P0.31` and right `P0.05` wake immediately; active debounce remains 3 ms and a 250 ms full scan covers missed interrupts | Both halves pass idle first-key, hold, release, and mixed-order hardware tests; idle scans fall from about 100/s to 4/s |
| Done | Backlight timeout | Backlight off after 30 seconds without a NocFree key press | Both halves now turn off independently of USB power and wake without losing the first key | A 10-second diagnostic image passed hardware testing; release images use the same path with a 30-second constant |
| Done | Left-central System OFF | Sleep after five minutes on battery; wake from a left key or left USB | A 10-second diagnostic passed USB blocking, battery System OFF, both-backlight preparation, USB/key wake, BLE restore, and right split reconnect; release timeout is five minutes | Measure sleep current and repeat long-duration wake cycles. A quick wake-key tap is not guaranteed to survive reset boot |
| Done | Periodic battery manager | Ongoing level and low-battery monitoring | Both halves sample every 60 seconds and on `Fn+I`; filtered values feed `Fn+I`, split battery transport, and low-battery LEDs | Automated tests, `L 100 R 100` hardware result, and stable long-running readings |
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

The red outputs now use `Standard0Disconnect1`: firmware pulls low only during
the 0.5-second-on/0.5-second-off low-battery pattern at 10% or below and releases
the line otherwise. The blue active-low LED uses the same timing while a BLE
profile is pairing. Hardware testing confirmed that blue keeps flashing after
releasing `Fn+3` and stops after selecting the bonded slot with `Fn+1`. Both
batteries were full, so physical red flashing remains unverified.

The remaining work is to record stock V2.3.0 behavior for boot, Wired,
connected, charging, fully charged, and link loss, then reproduce only verified
patterns. The official manual explicitly states that blue flashes during
pairing and red flashes for low battery, but it does not define every state or
timing pattern.

## Battery validation and correction plan

The stock V2.3.0 battery path has been recovered and implemented:

```text
adc_mV = raw * 3300 / 4095
battery_mV = adc_mV * 130 / 100
filtered_mV = previous * 0.75 + new * 0.25
percent = clamp(((floor(filtered_mV / 33) - 70) * 10) / 3, 0, 100)
```

The divider is enabled only for measurement, eight samples are averaged, both
halves sample every 60 seconds, and a disconnected right half starts as unknown
instead of 0%. The steps below validate the recovered behavior on real cells;
they should change the stock curve only if measurements demonstrate a problem.

### 1. Measure the ADC path first

Temporarily expose raw SAADC counts and calculated millivolts through CDC or a
diagnostic `Fn+I` format. For each half, compare firmware voltage with a DMM at
the battery terminals near 4.20, 4.00, 3.80, 3.65, and 3.50 V. Record whether
USB is disconnected and whether the backlight is on.

Fit a per-hardware-revision gain and offset:

```text
corrected_mV = measured_mV * gain + offset
```

Do not tune the percentage curve until this voltage error is known. The
`130/100` divider factor is now known to be the stock implementation, but board
and resistor tolerance still require measurement.

### 2. Capture a real discharge curve

Fully charge both 1100 mAh batteries, disconnect USB, let them rest, and run a
repeatable workload with a fixed backlight level. Log corrected voltage and
elapsed time or, preferably, discharged mAh from a battery analyzer/power
profiler. Repeat for the left and right halves because their radio roles and
loads differ.

Compare the resulting curve with the recovered stock calculation. Keep the
stock calculation if it is sufficiently accurate; otherwise use a small
piecewise lookup table derived from the measurements. Do not copy a generic
Li-ion table without validating it on this cell and board.

### 3. Stabilize the displayed value

- Keep taking samples only while the divider-enable pin is high, then turn it off.
- The current 75/25 IIR filter matches stock; add outlier rejection only if logs require it.
- Add hysteresis so the displayed percentage does not jump around a boundary.
- Verify that the current 60-second period does not materially increase power use.
- Keep the explicit unknown value for a right half that has not reported yet.

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
2. **Done:** stop driving right `P0.05` as an output.
3. **Done:** replace idle 10 ms polling with left `P0.31`/right `P0.05`
   interrupt wake plus a 250 ms safety scan; keep 3 ms active debounce scans.
4. **Done:** add the 30-second backlight timeout; first verify the same path with a 10-second diagnostic image.
5. **Done:** add five-minute battery-only System OFF for the left central with
   wake from left `P0.31` or left USB. Keep the right in System ON idle because
   the BLE-only split has no physical path for the left to wake a powered-off right.
6. **Partially done:** after the first split connection, slow right-side
   disconnected advertising from 250 ms to 1 second. Measure the actual current
   reduction before claiming a battery-life improvement.
7. Re-run latency, rollover, split reconnect, BLE multi-pairing, DFU, and warm
   I2C recovery tests after every power change.

Use the stock image on the same physical unit as the primary current target.
The published two-week lifetime is a useful end-to-end check, but it is too
dependent on backlight and workload to replace direct current measurement.

## Sources

- [Original NocFree community ZMK porting data](https://github.com/NocFreeKB/NocFree-and-zmk)
- [Official NocFree & manual](https://www.nocfree.com/pages/nocfree-and-manual)
- [Official product specifications](https://www.nocfree.com/products/nocfree-and-reservation)
