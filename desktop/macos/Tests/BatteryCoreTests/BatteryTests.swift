import XCTest
@testable import BatteryCore

final class BatteryTests: XCTestCase {
    private func packet(sequence: UInt16 = 1, id: UInt8 = 1, flags: UInt8 = 15,
                        left: UInt8 = 80, right: UInt8 = 65, age: UInt16 = 0) -> Data {
        Data([1, flags, id, 0, 0, 0, 0, 0, 0, 0, UInt8(truncatingIfNeeded: sequence), UInt8(sequence >> 8),
              left, right, UInt8(truncatingIfNeeded: age), UInt8(age >> 8),
              UInt8(truncatingIfNeeded: age), UInt8(age >> 8), 0, 0])
    }

    func testWireRejectsMalformedAndUnknownVersions() throws {
        let valid = packet()
        for length in [0, 19, 21] { XCTAssertThrowsError(try Snapshot(Data(repeating: 0, count: length))) }
        for (offset, value) in [(0, 2), (1, 0x80), (12, 101), (13, 254), (18, 3), (19, 1)] {
            var bad = valid; bad[offset] = UInt8(value)
            XCTAssertThrowsError(try Snapshot(bad))
        }
        XCTAssertThrowsError(try Snapshot(packet(flags: 4)))
        XCTAssertThrowsError(try Snapshot(packet(age: 181)))
        let zero = try Snapshot(packet(left: 0))
        XCTAssertEqual(zero.percent(right: false, elapsed: 0), 0)
        XCTAssertEqual(zero.percent(right: false, elapsed: 180), 0)
        XCTAssertNil(zero.percent(right: false, elapsed: 180.001))
    }

    func testPollingDoesNotRefreshAStaleMeasurementOrUndoInvalidation() throws {
        var store = BatteryStore()
        try store.receive(packet(age: 170), path: "usb", transport: .usb, now: 0)
        try store.receive(packet(age: 0), path: "usb", transport: .usb, now: 11)
        XCTAssertNil(store.paths["usb"]?.representative(11))
        try store.receive(packet(sequence: 2, flags: 5, right: 255), path: "usb", transport: .usb, now: 12)
        try store.receive(packet(sequence: 2), path: "usb", transport: .usb, now: 13)
        XCTAssertNil(store.paths["usb"]?.snapshot.percent(right: true, elapsed: 0))
        try store.receive(packet(sequence: 3, left: 79), path: "usb", transport: .usb, now: 14)
        XCTAssertEqual(store.paths["usb"]?.representative(14), 65)
        try store.receive(packet(sequence: 3, left: 1), path: "usb", transport: .usb, now: 15)
        XCTAssertEqual(store.paths["usb"]?.representative(15), 65)
    }

    func testRepeatedUnknownReadsPreserveUnknownAgeSentinel() throws {
        var store = BatteryStore()
        let unknown = packet(flags: 3, left: 255, right: 255, age: 65535)
        try store.receive(unknown, path: "usb", transport: .usb, now: 0)
        try store.receive(unknown, path: "usb", transport: .usb, now: 1000)
        XCTAssertEqual(store.paths["usb"]?.snapshot.leftAge, 65535)
        XCTAssertEqual(store.paths["usb"]?.snapshot.state(right: false, elapsed: 0), "측정 대기")
    }

    func testWrapReadNotifyOrderingAndReconnect() throws {
        var store = BatteryStore()
        try store.receive(packet(sequence: 65535), path: "ble", transport: .bluetooth, now: 0)
        XCTAssertTrue(try store.receive(packet(sequence: 0, left: 30), path: "ble", transport: .bluetooth, now: 1))
        XCTAssertFalse(try store.receive(packet(sequence: 65535), path: "ble", transport: .bluetooth, now: 2))
        XCTAssertEqual(store.paths["ble"]?.representative(2), 30)
        XCTAssertFalse(try store.receive(packet(sequence: 32768), path: "ble", transport: .bluetooth, now: 3))
        store.remove("ble")
        XCTAssertTrue(try store.receive(packet(sequence: 50000), path: "ble", transport: .bluetooth, now: 4))
        store.clear()
        XCTAssertTrue(store.paths.isEmpty)
    }

    func testIdentityDeduplicationTransportPreferenceAndUnpairedReceiver() throws {
        var store = BatteryStore()
        try store.receive(packet(left: 20, age: 170), path: "usb", transport: .usb, now: 0)
        try store.receive(packet(left: 30), path: "dongle", transport: .dongle, now: 0)
        try store.receive(packet(left: 40), path: "ble", transport: .bluetooth, now: 0)
        XCTAssertEqual(store.keyboards(now: 0).count, 1)
        XCTAssertEqual(store.keyboards(now: 0).first?.1.representative(0), 20)
        XCTAssertEqual(store.keyboards(now: 11).first?.1.representative(11), 30)
        try store.receive(packet(id: 2, left: 50), path: "dongle", transport: .dongle, now: 12)
        XCTAssertEqual(store.keyboards(now: 12).count, 2)
        var empty = packet(id: 0, flags: 0, left: 255, right: 255, age: 65535)
        empty[18] = 255
        try store.receive(empty, path: "dongle", transport: .dongle, now: 13)
        XCTAssertEqual(store.keyboards(now: 13).count, 1)
        XCTAssertNil(store.paths["dongle"]?.representative(13))
        let previous = store.paths["usb"]!.received
        XCTAssertThrowsError(try store.receive(Data([0]), path: "usb", transport: .usb, now: 14))
        XCTAssertEqual(store.paths["usb"]!.received, previous)
    }

    func testSharedGoldenVectors() throws {
        struct Vector: Decodable { let name: String; let hex: String; let valid: Bool }
        let override = ProcessInfo.processInfo.environment["BATTERY_VECTORS"]
        let source = URL(fileURLWithPath: #filePath).deletingLastPathComponent()
            .deletingLastPathComponent().deletingLastPathComponent()
            .deletingLastPathComponent().appendingPathComponent("protocol/vectors.json")
        let url = override.map { URL(fileURLWithPath: $0) } ?? source
        let vectors = try JSONDecoder().decode([Vector].self, from: Data(contentsOf: url))
        XCTAssertFalse(vectors.isEmpty)
        for vector in vectors {
            let hex = vector.hex.filter { !$0.isWhitespace }
            XCTAssertEqual(hex.count % 2, 0, vector.name)
            var bytes = [UInt8](); var index = hex.startIndex
            while index < hex.endIndex {
                let end = hex.index(index, offsetBy: 2)
                bytes.append(try XCTUnwrap(UInt8(hex[index..<end], radix: 16), vector.name)); index = end
            }
            if vector.valid { XCTAssertNoThrow(try Snapshot(Data(bytes)), vector.name) }
            else { XCTAssertThrowsError(try Snapshot(Data(bytes)), vector.name) }
        }
    }
}
