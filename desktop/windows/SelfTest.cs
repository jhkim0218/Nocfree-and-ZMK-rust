using System.Buffers.Binary;
namespace NocFreeBattery;
internal static class SelfTest
{
    private static byte[] Packet(ushort seq = 1, byte flags = 15, byte left = 80, byte right = 65, ushort age = 0, ulong id = 1)
    {
        var b = new byte[20]; b[0] = 1; b[1] = flags;
        BinaryPrimitives.WriteUInt64LittleEndian(b.AsSpan(2), id);
        BinaryPrimitives.WriteUInt16LittleEndian(b.AsSpan(10), seq);
        b[12] = left; b[13] = right;
        BinaryPrimitives.WriteUInt16LittleEndian(b.AsSpan(14), age);
        BinaryPrimitives.WriteUInt16LittleEndian(b.AsSpan(16), age);
        b[18] = id == 0 ? (byte)255 : (byte)0;
        return b;
    }
    private static void Check(bool ok, string intent) { if (!ok) throw new Exception(intent); }
    public static void Run()
    {
        var directory = new DirectoryInfo(AppContext.BaseDirectory);
        string? fixtures = null;
        while (directory != null)
        {
            var candidate = Path.Combine(directory.FullName, "desktop", "protocol", "vectors.json");
            if (File.Exists(candidate)) { fixtures = candidate; break; }
            directory = directory.Parent;
        }
        Check(fixtures != null, "shared protocol vectors must exist");
        using var vectors = System.Text.Json.JsonDocument.Parse(File.ReadAllText(fixtures!));
        int count = 0;
        foreach (var vector in vectors.RootElement.EnumerateArray())
        {
            bool accepted;
            try { Snapshot.Parse(Convert.FromHexString(vector.GetProperty("hex").GetString()!)); accepted = true; }
            catch (FormatException) { accepted = false; }
            Check(accepted == vector.GetProperty("valid").GetBoolean(), $"shared vector {vector.GetProperty("name").GetString()}");
            count++;
        }
        Check(count > 0, "shared vectors cannot be empty");
        var state = new BatteryState();
        Check(state.Accept("usb", Packet(left: 0, age: 180), 0, 0), "valid packet accepted");
        Check(BatteryState.Side(state.Devices(0)[0], false, 0) == "0%", "zero is real battery, and age 180 valid");
        Check(!BatteryState.Fresh(state.Devices(1)[0], false, 1), "older than 180 stale");
        foreach (var bad in new[] { Array.Empty<byte>(), new byte[21], Packet(left: 101), Packet(flags: 128) })
            Check(!state.Accept("usb", bad, 100, 0), "malformed cannot refresh");
        foreach (int offset in new[] { 0, 19, 18 }) { var b = Packet(); b[offset] = 99; Check(!state.Accept("usb", b, 100, 0), "version/reserved/output rejected"); }
        state.Clear();
        state.Accept("usb", Packet(seq: 65535), 0, 0);
        Check(state.Accept("usb", Packet(seq: 0, left: 70), 1, 0), "sequence wrap accepted");
        Check(!state.Accept("usb", Packet(seq: 65535, left: 80), 2, 0), "read after newer notification rejected");
        Check(state.Devices(2)[0].Value.Left == 70, "old read cannot replace new measurement");
        state.Accept("usb", Packet(seq: 0, left: 99), 100, 0);
        Check(!BatteryState.Fresh(state.Devices(182)[0], false, 182), "duplicate polling cannot rejuvenate age");
        state.Accept("usb", Packet(seq: 0, flags: 1, left: 255, right: 255), 101, 0);
        state.Accept("usb", Packet(seq: 0), 102, 0);
        Check(!BatteryState.Fresh(state.Devices(102)[0], false, 102), "same sequence cannot revive invalid flags");
        state.Accept("ble", Packet(seq: 2), 103, 2);
        Check(state.Devices(103).Length == 1 && state.Devices(103)[0].Priority == 2, "fresh duplicate path beats stale USB");
        state.Accept("usb", Packet(seq: 3), 104, 0);
        Check(state.Devices(104)[0].Priority == 0, "fresh USB preferred");
        state.Accept("other", Packet(id: 2), 104, 1);
        Check(state.Devices(104).Length == 2, "different keyboard stays separate");
        state.Accept("usb", Packet(id: 3), 105, 0);
        Check(state.Devices(105).Length == 3, "new keyboard identity resets sequence baseline");
        state.Remove("ble"); state.Remove("other");
        state.Accept("usb", Packet(id: 0, flags: 0, left: 255, right: 255, age: 65535), 106, 0);
        Check(state.Devices(106).Length == 0, "empty receiver clears previous keyboard");
        state.Accept("usb", Packet(), 107, 0); state.Clear();
        Check(state.Devices(107).Length == 0, "suspend clears cache");
        state.Accept("usb", Packet(seq: 0), 108, 0);
        Check(state.Devices(108).Length == 1, "resume permits fresh baseline");
        state.Remove("usb"); Check(state.Devices(108).Length == 0, "disconnect immediate invalidation");
        state.Accept("usb", Packet(flags: 3, left: 255, right: 255, age: 65534), 0, 0);
        state.Accept("usb", Packet(age: 0), 10000, 0);
        Check(state.Devices(10000)[0].Value.LeftAge == 65534, "age saturates without wrapping");
        Console.WriteLine("WINDOWS_BATTERY_TESTS_PASS");
    }
}
