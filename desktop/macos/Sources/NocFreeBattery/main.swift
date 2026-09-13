import AppKit
import BatteryCore
import ServiceManagement

final class AppDelegate: NSObject, NSApplicationDelegate, NSMenuDelegate {
    private let item = NSStatusBar.system.statusItem(withLength: NSStatusItem.variableLength)
    private let menu = NSMenu()
    private let usb = HIDReader()
    private let bluetooth = BluetoothReader()
    private var store = BatteryStore()
    private var issues: [String: String] = [:]
    private var timer: Timer?
    private var observers: [NSObjectProtocol] = []
    private var lastRefresh = -Double.infinity
    private var lastPoll = -Double.infinity
    private var suspended = false
    private var now: TimeInterval { ProcessInfo.processInfo.systemUptime }

    func applicationDidFinishLaunching(_ notification: Notification) {
        menu.delegate = self
        item.menu = menu
        item.button?.title = "NF --"
        item.button?.setAccessibilityLabel("NocFree 배터리")
        usb.onData = { [weak self] path, transport, data in self?.receive(data, path: path, transport: transport) }
        bluetooth.onData = { [weak self] path, data in self?.receive(data, path: path, transport: .bluetooth) }
        let remove: (String) -> Void = { [weak self] path in
            self?.store.remove(path); self?.issues.removeValue(forKey: path); self?.render()
            self?.issues.removeValue(forKey: path + ":notify")
        }
        let status: (String, String?) -> Void = { [weak self] path, message in
            self?.issues[path] = message; self?.render()
        }
        usb.onRemove = remove; bluetooth.onRemove = remove
        usb.onStatus = status; bluetooth.onStatus = status
        let center = NSWorkspace.shared.notificationCenter
        observers.append(center.addObserver(forName: NSWorkspace.willSleepNotification, object: nil, queue: .main) { [weak self] _ in
            self?.sleep(true)
        })
        observers.append(center.addObserver(forName: NSWorkspace.didWakeNotification, object: nil, queue: .main) { [weak self] _ in
            self?.sleep(false)
        })
        usb.start(); bluetooth.start()
        timer = Timer(timeInterval: 1, repeats: true) { [weak self] _ in
            guard let self, !self.suspended else { return }
            if self.now - self.lastPoll >= 30 { self.refresh() }
            self.render()
        }
        RunLoop.main.add(timer!, forMode: .common)
        refresh()
    }

    private func receive(_ data: Data, path: String, transport: Transport) {
        do {
            if try store.receive(data, path: path, transport: transport, now: now) {
                issues.removeValue(forKey: path)
            }
        } catch SnapshotError.unsupportedVersion {
            issues[path] = "배터리 프로토콜 업데이트 필요"
        } catch { issues[path] = "배터리 데이터 형식 오류" }
        render()
    }

    private func sleep(_ sleeping: Bool) {
        suspended = sleeping
        store.clear(); issues.removeAll()
        usb.suspend(sleeping); bluetooth.suspend(sleeping)
        lastRefresh = -Double.infinity; lastPoll = -Double.infinity
        if !sleeping { refresh() }
        render()
    }

    @objc private func refresh() {
        guard !suspended, now - lastRefresh >= 2 else { return }
        lastRefresh = now; lastPoll = now
        usb.refresh(); bluetooth.refresh()
        render()
    }

    func menuWillOpen(_ menu: NSMenu) { refresh(); render() }

    private func line(_ title: String, action: Selector? = nil) -> NSMenuItem {
        let entry = NSMenuItem(title: title, action: action, keyEquivalent: "")
        entry.target = self
        menu.addItem(entry)
        return entry
    }

    private func render() {
        let readings = store.keyboards(now: now)
        let values = readings.compactMap { $0.1.representative(now) }
        item.button?.title = values.min().map { "NF \($0)%" } ?? "NF --"
        menu.removeAllItems()
        if readings.isEmpty { _ = line(suspended ? "절전 · 재조회 대기" : "NocFree 연결 또는 지원 펌웨어 확인") }
        for (index, entry) in readings.enumerated() {
            let reading = entry.1
            let output = [0: "유선", 1: "Bluetooth", 2: "동글"][Int(reading.snapshot.output)] ?? "출력 알 수 없음"
            _ = line("NocFree\(readings.count > 1 ? " \(index + 1)" : "") · \(output)")
            _ = line("왼쪽   \(reading.snapshot.state(right: false, elapsed: reading.elapsed(now)))")
            _ = line("오른쪽 \(reading.snapshot.state(right: true, elapsed: reading.elapsed(now)))")
            let ages = [(false, reading.snapshot.leftAge), (true, reading.snapshot.rightAge)].compactMap { side, age -> Double? in
                reading.snapshot.percent(right: side, elapsed: reading.elapsed(now)) == nil ? nil : age + reading.elapsed(now)
            }
            if let age = ages.max() { _ = line("마지막 측정 최대 \(Int(age))초 전") }
            menu.addItem(.separator())
        }
        if store.paths.values.contains(where: { $0.snapshot.id == 0 }) { _ = line("동글 · 키보드 연결 끊김") }
        for issue in Set(issues.values).sorted() { _ = line(issue) }
        _ = line("새로 고침", action: #selector(refresh))
        let login = line("로그인 시 실행", action: #selector(toggleLogin))
        login.state = SMAppService.mainApp.status == .enabled ? .on : .off
        if SMAppService.mainApp.status == .requiresApproval { _ = line("로그인 항목 · 시스템 설정 승인 필요") }
        _ = line("종료", action: #selector(quit))
    }

    @objc private func toggleLogin() {
        do {
            if SMAppService.mainApp.status == .enabled { try SMAppService.mainApp.unregister() }
            else { try SMAppService.mainApp.register() }
            issues.removeValue(forKey: "login")
        } catch { issues["login"] = "로그인 항목 변경 실패: \(error.localizedDescription)" }
        render()
    }

    @objc private func quit() { NSApp.terminate(nil) }
    func applicationWillTerminate(_ notification: Notification) {
        timer?.invalidate(); usb.stop(); bluetooth.stop()
        for observer in observers { NSWorkspace.shared.notificationCenter.removeObserver(observer) }
    }
}

let app = NSApplication.shared
let delegate = AppDelegate()
app.delegate = delegate
app.setActivationPolicy(.accessory)
app.run()
