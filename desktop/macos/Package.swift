// swift-tools-version: 5.9
import PackageDescription

let package = Package(
    name: "NocFreeBattery", platforms: [.macOS(.v13)],
    products: [.executable(name: "NocFreeBattery", targets: ["NocFreeBattery"])],
    targets: [
        .target(name: "BatteryCore"),
        .executableTarget(name: "NocFreeBattery", dependencies: ["BatteryCore"]),
        .testTarget(name: "BatteryCoreTests", dependencies: ["BatteryCore"])
    ]
)
