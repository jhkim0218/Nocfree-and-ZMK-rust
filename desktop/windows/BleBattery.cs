using Windows.Devices.Bluetooth;
using Windows.Devices.Bluetooth.GenericAttributeProfile;
using Windows.Devices.Enumeration;
using Windows.Storage.Streams;

namespace NocFreeBattery;

internal sealed class BleBattery : IDisposable
{
    private static readonly Guid ServiceId = new("f3641600-00b0-4240-ba50-05ca45bf8abc");
    private static readonly Guid CharacteristicId = new("f3641601-00b0-4240-ba50-05ca45bf8abc");
    private readonly Dictionary<string, Connection> connections = new();
    private readonly Action<Action> post;
    private readonly Action<string, byte[]?> changed;
    private readonly Action<string> status;
    private int generation;
    private bool disposed;
    private readonly DeviceWatcher watcher;
    private readonly HashSet<string> known = new();
    private sealed record Connection(object Token, BluetoothLEDevice Device, GattDeviceService Service, GattCharacteristic Characteristic,
        Windows.Foundation.TypedEventHandler<BluetoothLEDevice, object> Disconnect);
    public BleBattery(Action<Action> post, Action<string, byte[]?> changed, Action<string> status, Action refresh)
    {
        this.post = post; this.changed = changed; this.status = status;
        watcher = DeviceInformation.CreateWatcher(BluetoothLEDevice.GetDeviceSelectorFromConnectionStatus(BluetoothConnectionStatus.Connected));
        watcher.Added += (_, _) => post(refresh);
        watcher.Updated += (_, _) => { };
        watcher.Removed += (_, e) => post(() => { Remove(e.Id); known.Remove(e.Id); changed("ble:" + e.Id, null); });
        watcher.Start();
    }
    private static byte[] Bytes(IBuffer buffer) { var b = new byte[buffer.Length]; using var reader = DataReader.FromBuffer(buffer); reader.ReadBytes(b); return b; }
    public async Task Refresh()
    {
        int epoch = generation;
        var devices = await DeviceInformation.FindAllAsync(BluetoothLEDevice.GetDeviceSelectorFromConnectionStatus(BluetoothConnectionStatus.Connected));
        if (epoch != generation || disposed) return;
        var ids = devices.Select(d => d.Id).ToHashSet();
        foreach (var id in known.Where(id => !ids.Contains(id)).ToArray()) { Remove(id); known.Remove(id); changed("ble:" + id, null); }
        foreach (var info in devices)
        {
            if (!known.Contains(info.Id) && !info.Name.Contains("NocFree", StringComparison.OrdinalIgnoreCase)) continue;
            using var timeout = new CancellationTokenSource(TimeSpan.FromSeconds(5));
            try
            {
                if (!connections.TryGetValue(info.Id, out var c))
                {
                    var device = await BluetoothLEDevice.FromIdAsync(info.Id).AsTask(timeout.Token);
                    if (device == null) { status("Bluetooth 권한 필요 또는 장치 접근 불가"); continue; }
                    GattDeviceService? service = null;
                    bool retained = false;
                    try
                    {
                        if (epoch != generation || disposed) return;
                        if (device.ConnectionStatus != BluetoothConnectionStatus.Connected) continue;
                        var services = await device.GetGattServicesForUuidAsync(ServiceId, BluetoothCacheMode.Uncached).AsTask(timeout.Token);
                        if (services.Status != GattCommunicationStatus.Success)
                        { status($"Bluetooth 서비스: {services.Status}"); continue; }
                        service = services.Services.FirstOrDefault();
                        foreach (var extra in services.Services.Skip(1)) extra.Dispose();
                        if (service == null) continue;
                        var chars = await service.GetCharacteristicsForUuidAsync(CharacteristicId, BluetoothCacheMode.Uncached).AsTask(timeout.Token);
                        if (chars.Status != GattCommunicationStatus.Success || chars.Characteristics.Count == 0)
                        { status($"Bluetooth 배터리 characteristic: {chars.Status}"); continue; }
                        if (epoch != generation || disposed) return;
                        var characteristic = chars.Characteristics[0];
                        var token = new object();
                        Windows.Foundation.TypedEventHandler<BluetoothLEDevice, object> disconnect = (d, _) =>
                        { if (d.ConnectionStatus != BluetoothConnectionStatus.Connected) post(() => { if (epoch == generation && connections.TryGetValue(info.Id, out var active) && ReferenceEquals(active.Token, token)) Remove(info.Id); }); };
                        c = new(token, device, service, characteristic, disconnect);
                        known.Add(info.Id);
                        connections[info.Id] = c; retained = true;
                        device.ConnectionStatusChanged += disconnect;
                    }
                    finally { if (!retained) { service?.Dispose(); device.Dispose(); } }
                }
                if (epoch != generation || disposed || !connections.ContainsKey(info.Id)) continue;
                var result = await c.Characteristic.ReadValueAsync(BluetoothCacheMode.Uncached).AsTask(timeout.Token);
                if (epoch != generation || disposed || !connections.ContainsKey(info.Id)) continue;
                if (result.Status == GattCommunicationStatus.Success) changed("ble:" + info.Id, Bytes(result.Value));
                else status($"Bluetooth 읽기: {result.Status}");
            }
            catch (UnauthorizedAccessException) { status("Bluetooth 권한 필요"); Remove(info.Id); }
            catch (OperationCanceledException) { status("Bluetooth 조회 시간 초과 · 다음 주기에 재시도"); Remove(info.Id); }
            catch (Exception ex) { status($"Bluetooth: {ex.Message}"); Remove(info.Id); }
            finally { Remove(info.Id, false); }
        }
    }
    private void Remove(string id, bool invalidate = true)
    {
        if (!connections.Remove(id, out var c)) return;
        c.Device.ConnectionStatusChanged -= c.Disconnect;
        c.Service.Dispose(); c.Device.Dispose(); if (invalidate) changed("ble:" + id, null);
    }
    public void Reset() { generation++; foreach (var id in connections.Keys.ToArray()) Remove(id); foreach (var id in known) changed("ble:" + id, null); known.Clear(); }
    public void Dispose() { disposed = true; watcher.Stop(); Reset(); }
}
