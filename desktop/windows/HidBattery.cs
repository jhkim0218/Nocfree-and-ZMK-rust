using Microsoft.Win32.SafeHandles;
using System.ComponentModel;
using System.Runtime.InteropServices;

namespace NocFreeBattery;

internal static class HidBattery
{
    [StructLayout(LayoutKind.Sequential)] private struct InterfaceData { public uint Size; public Guid Class; public uint Flags; public nuint Reserved; }
    [StructLayout(LayoutKind.Sequential)] private struct Attributes { public int Size; public ushort Vid, Pid, Version; }
    [StructLayout(LayoutKind.Sequential)] private struct Caps
    {
        public ushort Usage, Page, InputLength, OutputLength, FeatureLength;
        [MarshalAs(UnmanagedType.ByValArray, SizeConst = 17)] public ushort[] Reserved;
        public ushort LinkCount, InputButtons, InputValues, InputIndices, OutputButtons, OutputValues, OutputIndices, FeatureButtons, FeatureValues, FeatureIndices;
    }
    [DllImport("hid.dll")] private static extern void HidD_GetHidGuid(out Guid guid);
    [DllImport("setupapi.dll", SetLastError = true)] private static extern nint SetupDiGetClassDevsW(ref Guid guid, nint enumerator, nint window, uint flags);
    [DllImport("setupapi.dll", SetLastError = true)] private static extern bool SetupDiEnumDeviceInterfaces(nint set, nint info, ref Guid guid, uint index, ref InterfaceData data);
    [DllImport("setupapi.dll", SetLastError = true)] private static extern bool SetupDiGetDeviceInterfaceDetailW(nint set, ref InterfaceData data, nint detail, uint size, out uint required, nint info);
    [DllImport("setupapi.dll")] private static extern bool SetupDiDestroyDeviceInfoList(nint set);
    [DllImport("kernel32.dll", CharSet = CharSet.Unicode, SetLastError = true)] private static extern SafeFileHandle CreateFileW(string path, uint access, uint share, nint security, uint disposition, uint flags, nint template);
    [DllImport("hid.dll", SetLastError = true)] private static extern bool HidD_GetAttributes(SafeFileHandle file, ref Attributes attrs);
    [DllImport("hid.dll")] private static extern bool HidD_GetPreparsedData(SafeFileHandle file, out nint data);
    [DllImport("hid.dll")] private static extern bool HidD_FreePreparsedData(nint data);
    [DllImport("hid.dll")] private static extern int HidP_GetCaps(nint data, out Caps caps);
    [DllImport("hid.dll", SetLastError = true)] private static extern bool HidD_GetFeature(SafeFileHandle file, [In, Out] byte[] bytes, int size);

    public static List<(string Path, byte[] Bytes, int Priority)> Read()
    {
        var results = new List<(string, byte[], int)>();
        HidD_GetHidGuid(out var guid);
        var set = SetupDiGetClassDevsW(ref guid, 0, 0, 0x12);
        if (set == -1) throw new Win32Exception();
        try
        {
            for (uint i = 0; ; i++)
            {
                var data = new InterfaceData { Size = (uint)Marshal.SizeOf<InterfaceData>() };
                if (!SetupDiEnumDeviceInterfaces(set, 0, ref guid, i, ref data))
                {
                    if (Marshal.GetLastWin32Error() == 259) break;
                    throw new Win32Exception();
                }
                SetupDiGetDeviceInterfaceDetailW(set, ref data, 0, 0, out uint needed, 0);
                var detail = Marshal.AllocHGlobal((int)needed);
                try
                {
                    Marshal.WriteInt32(detail, nint.Size == 8 ? 8 : 6);
                    if (!SetupDiGetDeviceInterfaceDetailW(set, ref data, detail, needed, out _, 0)) continue;
                    string path = Marshal.PtrToStringUni(detail + 4)!;
                    bool left = path.Contains("vid_2886&pid_8029", StringComparison.OrdinalIgnoreCase);
                    bool dongle = path.Contains("vid_239a&pid_80d8", StringComparison.OrdinalIgnoreCase);
                    if (!left && !dongle) continue;
                    // Zero access, shared handle for capability inspection; never read keyboard input.
                    using var file = CreateFileW(path, 0, 3, 0, 3, 0, 0);
                    if (file.IsInvalid) continue;
                    var attrs = new Attributes { Size = Marshal.SizeOf<Attributes>() };
                    if (!HidD_GetAttributes(file, ref attrs) ||
                        !((attrs.Vid == 0x2886 && attrs.Pid == 0x8029) || (attrs.Vid == 0x239a && attrs.Pid == 0x80d8))) continue;
                    if (!HidD_GetPreparsedData(file, out var parsed)) continue;
                    Caps caps;
                    try { if (HidP_GetCaps(parsed, out caps) < 0) continue; }
                    finally { HidD_FreePreparsedData(parsed); }
                    if (caps.Page != 0xff60 || caps.Usage != 1 || caps.FeatureLength != 21) continue;
                    var bytes = new byte[21]; bytes[0] = 1;
                    if (!HidD_GetFeature(file, bytes, bytes.Length)) throw new Win32Exception(Marshal.GetLastWin32Error(), "Battery HID feature read failed");
                    if (bytes[0] != 1) throw new FormatException("Unexpected HID battery report ID");
                    results.Add(("usb:" + path, bytes[1..], left ? 0 : 1));
                }
                finally { Marshal.FreeHGlobal(detail); }
            }
        }
        finally { SetupDiDestroyDeviceInfoList(set); }
        return results;
    }
}
