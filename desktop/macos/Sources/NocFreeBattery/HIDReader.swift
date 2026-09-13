import Foundation
import IOKit.hid
import BatteryCore

final class HIDReader {
    var onData: (String, Transport, Data) -> Void = { _, _, _ in }
    var onRemove: (String) -> Void = { _ in }
    var onStatus: (String, String?) -> Void = { _, _ in }
    private let manager = IOHIDManagerCreate(kCFAllocatorDefault, IOOptionBits(kIOHIDOptionsTypeNone))
    private var devices: [String: IOHIDDevice] = [:]
    private var pending: [String: UUID] = [:]
    private let reads = DispatchQueue(label: "com.nocfree.battery.hid", qos: .utility)
    private var suspended = false

    private func key(_ device: IOHIDDevice) -> String {
        var id: UInt64 = 0
        IORegistryEntryGetRegistryEntryID(IOHIDDeviceGetService(device), &id)
        return "usb:\(id)"
    }

    func start() {
        let matches: [[String: Any]] = [(0x2886, 0x8029), (0x239a, 0x80d8)].map { vid, pid in
            [kIOHIDVendorIDKey: vid, kIOHIDProductIDKey: pid,
             kIOHIDDeviceUsagePageKey: 0xff60, kIOHIDDeviceUsageKey: 1]
        }
        IOHIDManagerSetDeviceMatchingMultiple(manager, matches as CFArray)
        let context = Unmanaged.passUnretained(self).toOpaque()
        IOHIDManagerRegisterDeviceMatchingCallback(manager, { context, result, _, device in
            guard let context else { return }
            let owner = Unmanaged<HIDReader>.fromOpaque(context).takeUnretainedValue()
            let key = owner.key(device)
            guard result == kIOReturnSuccess else {
                owner.onStatus(key, "USB 장치 오류 (\(result))"); return
            }
            // Nonexclusive open of the matched vendor collection only.
            let status = IOHIDDeviceOpen(device, IOOptionBits(kIOHIDOptionsTypeNone))
            guard status == kIOReturnSuccess else {
                owner.onStatus(key, "USB 접근 실패 (\(status))"); return
            }
            owner.devices[key] = device
            if !owner.suspended { owner.read(key, device) }
        }, context)
        IOHIDManagerRegisterDeviceRemovalCallback(manager, { context, _, _, device in
            guard let context else { return }
            let owner = Unmanaged<HIDReader>.fromOpaque(context).takeUnretainedValue()
            let key = owner.key(device)
            owner.devices.removeValue(forKey: key)
            owner.pending.removeValue(forKey: key)
            IOHIDDeviceClose(device, IOOptionBits(kIOHIDOptionsTypeNone))
            owner.onRemove(key)
        }, context)
        IOHIDManagerScheduleWithRunLoop(manager, CFRunLoopGetMain(), CFRunLoopMode.commonModes.rawValue)
        let result = IOHIDManagerOpen(manager, IOOptionBits(kIOHIDOptionsTypeNone))
        if result != kIOReturnSuccess { onStatus("usb", "USB 관리자 오류 (\(result))") }
    }

    func refresh() {
        guard !suspended else { return }
        for (key, device) in devices { read(key, device) }
    }

    private func read(_ key: String, _ device: IOHIDDevice) {
        guard pending[key] == nil else { return }
        let token = UUID()
        pending[key] = token
        reads.async { [weak self] in
            var report = [UInt8](repeating: 0, count: 21)
            report[0] = 1
            var length = report.count
            let result = IOHIDDeviceGetReport(device, kIOHIDReportTypeFeature, 1, &report, &length)
            let vendor = (IOHIDDeviceGetProperty(device, kIOHIDVendorIDKey as CFString) as? NSNumber)?.intValue
            let payload = length == 21 && report[0] == 1 ? Data(report.dropFirst()) : nil
            DispatchQueue.main.async {
                guard let self, self.pending[key] == token else { return }
                self.pending.removeValue(forKey: key)
                guard !self.suspended, self.devices[key] != nil else { return }
                guard result == kIOReturnSuccess else {
                    self.onStatus(key, "USB 조회 실패 (\(result))"); return
                }
                guard let payload else { self.onStatus(key, "배터리 보고서 형식 오류"); return }
                self.onData(key, vendor == 0x239a ? .dongle : .usb, payload)
            }
        }
    }

    func suspend(_ value: Bool) { suspended = value; if value { pending.removeAll() } }

    func stop() {
        for device in devices.values { IOHIDDeviceClose(device, IOOptionBits(kIOHIDOptionsTypeNone)) }
        devices.removeAll()
        pending.removeAll()
        IOHIDManagerUnscheduleFromRunLoop(manager, CFRunLoopGetMain(), CFRunLoopMode.commonModes.rawValue)
        IOHIDManagerClose(manager, IOOptionBits(kIOHIDOptionsTypeNone))
    }
}
