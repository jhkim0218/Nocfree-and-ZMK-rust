import Foundation
import CoreBluetooth

final class BluetoothReader: NSObject, CBCentralManagerDelegate, CBPeripheralDelegate {
    static let service = CBUUID(string: "f3641600-00b0-4240-ba50-05ca45bf8abc")
    static let characteristic = CBUUID(string: "f3641601-00b0-4240-ba50-05ca45bf8abc")
    private static let hid = CBUUID(string: "1812")
    var onData: (String, Data) -> Void = { _, _ in }
    var onRemove: (String) -> Void = { _ in }
    var onStatus: (String, String?) -> Void = { _, _ in }
    private var central: CBCentralManager!
    private var devices: [UUID: CBPeripheral] = [:]
    private var deadlines: [UUID: DispatchWorkItem] = [:]
    private var releasing: Set<UUID> = []
    private var suspended = false

    func start() { central = CBCentralManager(delegate: self, queue: .main) }
    private func key(_ device: CBPeripheral) -> String { "ble:\(device.identifier)" }

    func centralManagerDidUpdateState(_ central: CBCentralManager) {
        if central.state == .poweredOn { onStatus("bluetooth", nil); refresh(); return }
        for device in devices.values { onRemove(key(device)) }
        cancelSessions()
        devices.removeAll()
        releasing.removeAll()
        onStatus("bluetooth", central.state == .unauthorized ? "Bluetooth 권한 필요" :
            central.state == .unsupported ? "Bluetooth 미지원" : "Bluetooth 사용 불가")
    }

    func refresh() {
        guard !suspended, central?.state == .poweredOn else { return }
        // No scan: HID may already own the OS connection even when the custom UUID is not advertised.
        let known = central.retrieveConnectedPeripherals(withServices: [Self.service])
        let hid = central.retrieveConnectedPeripherals(withServices: [Self.hid]).filter {
            $0.name?.localizedCaseInsensitiveContains("nocfree") == true
        }
        let connected = Dictionary((known + hid).map { ($0.identifier, $0) }, uniquingKeysWith: { a, _ in a })
        for (id, device) in devices where connected[id] == nil && deadlines[id] == nil {
            onRemove(key(device)); devices.removeValue(forKey: id)
        }
        for (id, device) in connected where deadlines[id] == nil && !releasing.contains(id) {
            devices[id] = device
            device.delegate = self
            // A bounded session prevents this companion from owning an indefinite BLE connection.
            let expiry = DispatchWorkItem { [weak self] in self?.release(device) }
            deadlines[id] = expiry
            DispatchQueue.main.asyncAfter(deadline: .now() + 5, execute: expiry)
            central.connect(device, options: nil)
        }
    }

    func centralManager(_ central: CBCentralManager, didConnect peripheral: CBPeripheral) {
        guard deadlines[peripheral.identifier] != nil, !suspended else { release(peripheral); return }
        onRemove(key(peripheral))
        peripheral.discoverServices([Self.service])
    }

    func peripheral(_ peripheral: CBPeripheral, didDiscoverServices error: Error?) {
        guard active(peripheral) else { return }
        guard error == nil, let service = peripheral.services?.first(where: { $0.uuid == Self.service }) else {
            onStatus(key(peripheral), error == nil ? "펌웨어 배터리 서비스 미지원" : "Bluetooth 서비스 조회 실패")
            release(peripheral); return
        }
        peripheral.discoverCharacteristics([Self.characteristic], for: service)
    }

    func peripheral(_ peripheral: CBPeripheral, didDiscoverCharacteristicsFor service: CBService, error: Error?) {
        guard active(peripheral) else { return }
        guard error == nil, let value = service.characteristics?.first(where: { $0.uuid == Self.characteristic }),
              value.properties.contains(.read) else {
            onStatus(key(peripheral), "펌웨어 배터리 조회 미지원"); release(peripheral); return
        }
        // Read-only polling avoids CCCD writes and app-owned persistent notifications.
        peripheral.readValue(for: value)
    }

    func peripheral(_ peripheral: CBPeripheral, didUpdateValueFor characteristic: CBCharacteristic, error: Error?) {
        guard active(peripheral), characteristic.uuid == Self.characteristic else { return }
        guard error == nil, let data = characteristic.value else {
            onStatus(key(peripheral), "Bluetooth 조회 실패 · 권한 및 연결 확인"); return
        }
        onData(key(peripheral), data)
        release(peripheral)
    }

    func centralManager(_ central: CBCentralManager, didFailToConnect peripheral: CBPeripheral, error: Error?) {
        finish(peripheral)
        onRemove(key(peripheral)); onStatus(key(peripheral), "Bluetooth 접근 실패")
    }

    func centralManager(_ central: CBCentralManager, didDisconnectPeripheral peripheral: CBPeripheral, error: Error?) {
        let intentional = releasing.remove(peripheral.identifier) != nil
        finish(peripheral)
        if !intentional { onRemove(key(peripheral)); devices.removeValue(forKey: peripheral.identifier) }
        // Never reconnect from this callback: the user may have changed keyboard host profiles.
    }

    private func active(_ peripheral: CBPeripheral) -> Bool {
        !suspended && deadlines[peripheral.identifier] != nil && !releasing.contains(peripheral.identifier)
    }
    private func finish(_ peripheral: CBPeripheral) {
        deadlines.removeValue(forKey: peripheral.identifier)?.cancel()
    }
    private func release(_ peripheral: CBPeripheral) {
        finish(peripheral)
        guard !releasing.contains(peripheral.identifier) else { return }
        releasing.insert(peripheral.identifier)
        central.cancelPeripheralConnection(peripheral)
    }
    private func cancelSessions() {
        for device in devices.values where deadlines[device.identifier] != nil { release(device) }
    }
    func suspend(_ value: Bool) {
        suspended = value
        if value { cancelSessions(); devices.removeAll() }
        else { refresh() }
    }
    func stop() { suspend(true) }
}
