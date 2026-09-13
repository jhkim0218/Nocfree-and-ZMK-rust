# NocFree Battery for macOS (experimental)

Native AppKit menu application for macOS 13+, Intel and Apple Silicon. This source has not been compiled on a Mac or qualified with hardware. It uses only Apple frameworks; no package dependencies, keyboard input monitoring, CDC/serial ports, background daemon or cloud service.

## Build and test on a Mac

Install Xcode 15 or later and select its command line tools. Swift tools minimum is 5.9; the exact release toolchain must be recorded when qualification runs (none is certified yet). From this directory:

```sh
xcodebuild -version
swift --version
swift test
bash build-app.sh
open "dist/NocFree Battery.app"
```

`swift test` executes protocol/state tests and reads the actual shared `../protocol/vectors.json`. Set `BATTERY_VECTORS=/absolute/path/vectors.json` to override. Missing fixtures fail the test. `build-app.sh` runs tests, builds both architectures, validates the plist and architecture slices, ad-hoc signs the local bundle and prints the executable SHA-256. Run the bundle instead of `swift run` so Bluetooth usage metadata and login-item identity are present. Copy the app into `/Applications` before enabling login at startup. Startup is opt-in using `SMAppService.mainApp`; disable it before deleting the app. No app data is persisted.

## Behavior and prototype limits

* USB uses nonexclusive IOKit matching for exactly `2886:8029` or `239a:80d8`, usage page `FF60`, usage `1`. It reads Feature Report ID `1`, requiring 21 bytes including ID. No input reports or keyboard collections are opened. Blocking HID requests run off the menu thread; one request per device is outstanding and responses arriving after removal/sleep are discarded.
* BLE discovers already OS-connected peripherals by the custom service or HID `1812` plus a NocFree name, confirms the custom service, then reads without writing subscription descriptors. It does not scan, change host profiles, delete bonds, or reconnect from disconnect callbacks. Each app-owned connection is released after its read or at most 5 seconds; canceling it releases this client's interest and must be verified to preserve the OS HID connection on each target macOS version.
* Discovery/read is repeated every 30 seconds and at menu open/manual refresh, with a 2-second refresh debounce. Bounded BLE observation intentionally changes the design's direct-BLE 5-second continuous-notify target to up to **35 seconds** between sessions. Notifications are consumed while a session is open. This is a phase-1 power-conscious prototype choice, not proven power behavior. Verify it does not wake/retain the keyboard or switch the active host. No continuous scanning or permanent app BLE subscription is used.
* Original age plus elapsed monotonic time determines freshness. 180 seconds is valid; greater than 180 is stale. Equal sequence queries cannot lower age, change a percentage, or resurrect invalid sides. Modulo sequence rejects stale reads; reconnect, replacement identity and system resume reset the baseline. Malformed data does not refresh an old sample. Sleep invalidates every cached path immediately.
* Same nonzero keyboard IDs are grouped, selecting a fresh USB path, then dongle, then BLE; selected output is shown separately. ID-zero receivers never become a fake keyboard. Different keyboards have separate sections. Unknown, disconnected, stale, unsupported and Bluetooth-permission failures have distinct labels. Battery percentage uses firmware values with no charging inference.
* Permission denial affects BLE only. USB still runs. Legacy firmware without a matched HID collection cannot be distinguished from absence without opening unrelated interfaces, so the empty state asks the user to check connection/support. A discovered legacy BLE NocFree with no battery service is explicitly marked unsupported. Bluetooth read failures are displayed without replacing the last valid measurement.

## Signing and notarization

Use your Developer ID Application identity (requires an Apple developer account):

```sh
SIGN_IDENTITY='Developer ID Application: YOUR NAME (TEAMID)' bash build-app.sh
ditto -c -k --keepParent "dist/NocFree Battery.app" dist/NocFreeBattery.zip
xcrun notarytool submit dist/NocFreeBattery.zip --keychain-profile YOUR_NOTARY_PROFILE --wait
xcrun stapler staple "dist/NocFree Battery.app"
codesign --verify --deep --strict "dist/NocFree Battery.app"
spctl --assess --type execute --verbose=2 "dist/NocFree Battery.app"
ditto -c -k --keepParent "dist/NocFree Battery.app" dist/NocFreeBattery-notarized.zip
shasum -a 256 dist/NocFreeBattery-notarized.zip
```

Create the notarytool keychain profile yourself with Apple's supported credential flow; do not put credentials in scripts or source control. This is a directly distributed, hardened-runtime app, not an App Sandbox/App Store package. No entitlement is added speculatively. Actual Bluetooth and HID permission behavior, signing, notarization and clean-machine installation remain release gates.

## Physical acceptance and evidence

Do not flash firmware as part of the app build. With separately authorized firmware, record macOS version, Mac architecture, Xcode/Swift versions, firmware role/layout/SHA-256 and app executable/archive hashes in `macos-battery-qualification.md`. Keep timestamps and observed values in `macos-battery-observations.csv`, and power measurements in `macos-battery-power.csv`.

1. USB and dongle: launch with device already plugged in; verify left/right within 5 seconds, inspect Feature ID/length, then test hotplug and unplug. New source measurements must appear within 35 seconds. Dongle with no keyboard must show disconnected, never the previous keyboard's value.
2. BLE: pair normally in System Settings; open the app and approve Bluetooth access. Verify custom service access while typing, after the app releases its read session, at the next refresh, during sleep and when switching host profiles. Record whether OS HID remains connected and whether inactive-host discovery ever reconnects. Deny permission, restart, and verify USB/dongle still work. Re-enable permission and refresh/restart as required by the OS.
3. Turn right half off and restore it; verify stale/unknown/0% distinctions and reconnect freshness. Turn all keyboard power off, swap the keyboard paired to a dongle, attach multiple paths to one keyboard and two separate keyboards. Check identity grouping and selected-output labels.
4. Stop source updates: at 180 seconds a valid sample may still display; after 180 it must show delayed. Sleep/wake the Mac and confirm no pre-sleep percentage appears before a new read. Run repeated menu opening to confirm no forced ADC measurement or burst of requests.
5. Exercise fast two-hand typing, media/pointer controls, Fn+I, backlight, Link and existing recovery paths while all transports are observed. Compare idle and System OFF current with and without the app. Record direct-BLE latency and the bounded session's effect on reconnect/sleep; do not claim battery-life neutrality without measurement.
6. On both architectures, test signed install, Bluetooth prompt, menu responsiveness, startup opt-in/out, quit, notarization, uninstall and clean reinstall. Observe macOS's own Bluetooth battery UI separately; its support is not guaranteed by this companion.

API references: [IOHIDDeviceGetReport](https://developer.apple.com/documentation/iokit/1588659-iohiddevicegetreport), [connected peripheral retrieval](https://developer.apple.com/documentation/corebluetooth/cbcentralmanager/retrieveconnectedperipherals(withservices:)), [SMAppService](https://developer.apple.com/documentation/servicemanagement/smappservice), [Bluetooth purpose string](https://developer.apple.com/documentation/bundleresources/information-property-list/nsbluetoothalwaysusagedescription).
