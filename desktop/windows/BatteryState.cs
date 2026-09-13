using System.Buffers.Binary;

namespace NocFreeBattery;

internal sealed record Snapshot(byte Flags, ulong Id, ushort Sequence, byte Left, byte Right,
    ushort LeftAge, ushort RightAge, byte Output)
{
    public static Snapshot Parse(ReadOnlySpan<byte> b)
    {
        if (b.Length != 20 || b[0] != 1 || (b[1] & 0xf0) != 0 || b[19] != 0 ||
            (b[12] > 100 && b[12] != 255) || (b[13] > 100 && b[13] != 255) ||
            (b[18] > 2 && b[18] != 255)) throw new FormatException("Unsupported or malformed battery snapshot");
        var snapshot = new Snapshot(b[1], BinaryPrimitives.ReadUInt64LittleEndian(b[2..]),
            BinaryPrimitives.ReadUInt16LittleEndian(b[10..]), b[12], b[13],
            BinaryPrimitives.ReadUInt16LittleEndian(b[14..]), BinaryPrimitives.ReadUInt16LittleEndian(b[16..]), b[18]);
        foreach (bool right in new[] { false, true })
        {
            bool flagged = (snapshot.Flags & (right ? 8 : 4)) != 0;
            if (flagged ? !snapshot.Valid(right) : (right ? snapshot.Right : snapshot.Left) != 255)
                throw new FormatException("Contradictory battery state");
        }
        if (snapshot.Id == 0 && (snapshot.Flags != 0 || snapshot.Output != 255 ||
            snapshot.LeftAge != 65535 || snapshot.RightAge != 65535))
            throw new FormatException("Invalid empty receiver");
        return snapshot;
    }
    public bool Valid(bool right) => (Flags & (right ? 2 : 1)) != 0 &&
        (Flags & (right ? 8 : 4)) != 0 && (right ? Right : Left) <= 100 && (right ? RightAge : LeftAge) <= 180;
}

internal sealed class BatteryState
{
    internal sealed record Entry(Snapshot Value, double Received, int Priority);
    private readonly Dictionary<string, Entry> paths = new();
    public void Remove(string path) => paths.Remove(path);
    public void Clear() => paths.Clear();
    public bool Accept(string path, ReadOnlySpan<byte> bytes, double now, int priority)
    {
        Snapshot next;
        try { next = Snapshot.Parse(bytes); } catch (FormatException) { return false; }
        if (next.Id == 0) { Remove(path); return true; }
        if (paths.TryGetValue(path, out var old) && old.Value.Id == next.Id)
        {
            int delta = (ushort)(next.Sequence - old.Value.Sequence);
            if (delta >= 32768) return false;
            if (delta == 0)
            {
                // A repeated GET_REPORT cannot rejuvenate a measurement or revive invalid flags.
                next = next with {
                    Flags = (byte)(next.Flags & old.Value.Flags),
                    Left = old.Value.Left, Right = old.Value.Right,
                    LeftAge = Older(next.LeftAge, old.Value.LeftAge, now - old.Received),
                    RightAge = Older(next.RightAge, old.Value.RightAge, now - old.Received)
                };
            }
        }
        paths[path] = new(next, now, priority);
        return true;
    }
    private static ushort Older(ushort a, ushort b, double elapsed) =>
        a == 65535 || b == 65535 ? (ushort)65535 : (ushort)Math.Min(65534, Math.Max(a, b + Math.Ceiling(Math.Max(0, elapsed))));
    public static bool Fresh(Entry e, bool right, double now) => e.Value.Valid(right) &&
        (right ? e.Value.RightAge : e.Value.LeftAge) + Math.Max(0, now - e.Received) <= 180;
    public Entry[] Devices(double now) => paths.Values.GroupBy(e => e.Value.Id)
        .Select(g => g.OrderByDescending(e => Fresh(e, false, now) || Fresh(e, true, now))
            .ThenBy(e => e.Priority).First()).ToArray();
    public static string Side(Entry e, bool right, double now)
    {
        if ((e.Value.Flags & (right ? 2 : 1)) == 0) return "연결 끊김";
        if (Fresh(e, right, now)) return $"{(right ? e.Value.Right : e.Value.Left)}%";
        return (right ? e.Value.RightAge : e.Value.LeftAge) == 65535 ? "알 수 없음" : "업데이트 지연";
    }
}
