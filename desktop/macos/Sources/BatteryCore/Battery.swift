import Foundation

public enum SnapshotError: Error { case malformed, unsupportedVersion }

public struct Snapshot {
    public let id: UInt64
    public let sequence: UInt16
    public var flags: UInt8
    public let left: UInt8
    public let right: UInt8
    public var leftAge: Double
    public var rightAge: Double
    public let output: UInt8

    public init(_ data: Data) throws {
        let b = Array(data)
        guard b.count == 20 else { throw SnapshotError.malformed }
        guard b[0] == 1 else { throw SnapshotError.unsupportedVersion }
        guard b[1] & 0xf0 == 0, b[19] == 0,
              b[12] <= 100 || b[12] == 255, b[13] <= 100 || b[13] == 255,
              [0, 1, 2, 255].contains(b[18]) else { throw SnapshotError.malformed }
        func word(_ i: Int) -> UInt16 { UInt16(b[i]) | UInt16(b[i + 1]) << 8 }
        id = (0..<8).reduce(UInt64(0)) { $0 | UInt64(b[2 + $1]) << (8 * $1) }
        sequence = word(10); flags = b[1]; left = b[12]; right = b[13]
        leftAge = Double(word(14)); rightAge = Double(word(16)); output = b[18]
        guard (flags & 4 == 0 || (flags & 1 != 0 && left <= 100 && leftAge <= 180)),
              (flags & 8 == 0 || (flags & 2 != 0 && right <= 100 && rightAge <= 180)),
              (flags & 4 != 0 || left == 255), (flags & 8 != 0 || right == 255),
              (id != 0 || (flags == 0 && output == 255 && leftAge == 65535 && rightAge == 65535))
        else { throw SnapshotError.malformed }
    }

    public func percent(right side: Bool, elapsed: Double) -> Int? {
        let connected: UInt8 = side ? 2 : 1
        let valid: UInt8 = side ? 8 : 4
        let age = (side ? rightAge : leftAge) + max(0, elapsed)
        guard flags & connected != 0, flags & valid != 0, age <= 180 else { return nil }
        return Int(side ? right : left)
    }

    public func state(right side: Bool, elapsed: Double) -> String {
        if flags & (side ? 2 : 1) == 0 { return "연결 끊김" }
        if let value = percent(right: side, elapsed: elapsed) { return "\(value)%" }
        return (side ? rightAge : leftAge) == 65535 ? "측정 대기" : "업데이트 지연"
    }
}

public enum Transport: Int { case usb, dongle, bluetooth }
public struct Reading {
    public var snapshot: Snapshot
    public var received: TimeInterval
    public let transport: Transport
    public func elapsed(_ now: TimeInterval) -> Double { max(0, now - received) }
    public func representative(_ now: TimeInterval) -> Int? {
        [snapshot.percent(right: false, elapsed: elapsed(now)),
         snapshot.percent(right: true, elapsed: elapsed(now))].compactMap { $0 }.min()
    }
}

public struct BatteryStore {
    public private(set) var paths: [String: Reading] = [:]
    public init() {}

    public mutating func remove(_ path: String) { paths.removeValue(forKey: path) }
    public mutating func clear() { paths.removeAll() }

    @discardableResult
    public mutating func receive(_ data: Data, path: String, transport: Transport, now: TimeInterval) throws -> Bool {
        var value = try Snapshot(data)
        if let previous = paths[path], previous.snapshot.id == value.id {
            let delta = value.sequence &- previous.snapshot.sequence
            if delta >= 32768 { return false }
            if delta == 0 {
                let incoming = value
                value = previous.snapshot
                func older(_ incoming: Double, _ prior: Double) -> Double {
                    if incoming == 65535 || prior == 65535 { return 65535 }
                    return min(65534, max(incoming, prior + previous.elapsed(now)))
                }
                value.leftAge = older(incoming.leftAge, previous.snapshot.leftAge)
                value.rightAge = older(incoming.rightAge, previous.snapshot.rightAge)
                // Duplicate queries may invalidate a sample, but cannot resurrect one.
                value.flags &= incoming.flags
            }
        }
        paths[path] = Reading(snapshot: value, received: now, transport: transport)
        return true
    }

    public func keyboards(now: TimeInterval) -> [(UInt64, Reading)] {
        let groups = Dictionary(grouping: paths.values.filter { $0.snapshot.id != 0 }, by: { $0.snapshot.id })
        return groups.map { id, values in
            let selected = values.sorted {
                let a = $0.representative(now) != nil, b = $1.representative(now) != nil
                return a != b ? a : $0.transport.rawValue < $1.transport.rawValue
            }[0]
            return (id, selected)
        }.sorted { $0.0 < $1.0 }
    }
}
