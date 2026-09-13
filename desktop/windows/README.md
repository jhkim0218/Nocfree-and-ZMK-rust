# NocFree Battery for Windows (experimental)

Windows 11 tray companion for USB, direct Bluetooth and the Rust dongle. Reads only the vendor battery HID collection or the custom encrypted GATT service; does not open COM ports, intercept keys, change profiles or flash firmware.

```powershell
dotnet build desktop/windows/NocFreeBattery.csproj -c Release
dotnet run --project desktop/windows/NocFreeBattery.csproj -c Release --no-build -- --self-test
dotnet run --project desktop/windows/NocFreeBattery.csproj -c Release --no-build
```

Build requires .NET 8 SDK. Self-tests use `desktop/protocol/vectors.json` and deterministic freshness/sequence/disconnect scenarios; they do not access devices. A compiled app still needs physical USB/BLE/dongle qualification. Right-click the tray icon to see both halves and refresh. Login startup is off unless explicitly selected; disable it before removing the application.

USB reads every 30 seconds and on device/menu events. Direct BLE reads already connected NocFree devices in sessions limited to five seconds, then releases app resources; it never sets MaintainConnection or writes subscription descriptors. Between observations the cache continues aging. A disconnected path is invalidated on device events or the next discovery. Bluetooth access errors do not disable USB.

Publish an unsigned local test package with `dotnet publish desktop/windows/NocFreeBattery.csproj -c Release -r win-x64 --self-contained true -o target/battery-preview/windows-x64` (use `win-arm64` on ARM64). Run `NocFreeBattery.exe` from the published folder. Public distribution still requires signing and clean-machine/hardware tests. No auto-updater or installer changes system services.

See `docs/BATTERY_DESKTOP_TESTING_ko.md` for firmware hashes, physical acceptance and limits. macOS sources are in `desktop/macos` and require a Mac build.
