using System.Diagnostics;
using Microsoft.Win32;

namespace NocFreeBattery;

internal static class Program
{
    [STAThread]
    private static int Main(string[] args)
    {
        if (args.Contains("--self-test")) { SelfTest.Run(); return 0; }
        ApplicationConfiguration.Initialize();
        using var app = new BatteryApp();
        Application.Run(app);
        return 0;
    }
}

internal sealed class BatteryApp : ApplicationContext
{
    private readonly BatteryState state = new();
    private readonly NotifyIcon tray;
    private readonly ContextMenuStrip menu = new();
    private readonly System.Windows.Forms.Timer timer = new() { Interval = 1000 };
    private readonly DeviceWindow window;
    private readonly BleBattery ble;
    private readonly HashSet<string> usbPaths = new();
    private readonly Stopwatch clock = Stopwatch.StartNew();
    private double lastPoll = -30, lastMenuPoll = -2;
    private bool refreshing, refreshAgain, bleRefreshing, suspended, closing;
    private int generation;
    private string status = "장치 검색 중";
    private double Now => clock.Elapsed.TotalSeconds;
    public BatteryApp()
    {
        window = new DeviceWindow(() => { if (!closing) _ = Refresh(); });
        _ = window.Handle;
        ble = new BleBattery(Post, (path, bytes) => {
            if (bytes == null) state.Remove(path);
            else if (!state.Accept(path, bytes, Now, 2)) status = "Bluetooth 데이터 미지원 또는 오래된 응답";
            Paint();
        }, text => { status = text; Paint(); }, () => _ = Refresh());
        tray = new NotifyIcon { Icon = SystemIcons.Application, Text = "NocFree Battery", ContextMenuStrip = menu, Visible = true };
        menu.Opening += (_, _) => { Paint(); if (Now - lastMenuPoll >= 2) { lastMenuPoll = Now; _ = Refresh(); } };
        timer.Tick += (_, _) => { Paint(); if (!suspended && Now - lastPoll >= 30) _ = Refresh(); };
        SystemEvents.PowerModeChanged += PowerChanged;
        timer.Start();
        Post(() => _ = Refresh());
    }
    private void Post(Action action) { if (!closing && !window.IsDisposed) window.BeginInvoke(action); }
    private void PowerChanged(object sender, PowerModeChangedEventArgs e) => Post(() => {
        if (e.Mode == PowerModes.Suspend || e.Mode == PowerModes.Resume)
        {
            generation++; state.Clear(); usbPaths.Clear(); ble.Reset();
            suspended = e.Mode == PowerModes.Suspend;
            status = suspended ? "절전 중" : "측정 대기";
            Paint(); if (!suspended) _ = Refresh();
        }
    });
    private async Task Refresh()
    {
        if (closing || suspended) return;
        if (refreshing) { refreshAgain = true; return; }
        refreshing = true; lastPoll = Now;
        int epoch = generation;
        try
        {
            status = "";
            try
            {
                var usb = await Task.Run(HidBattery.Read);
                if (epoch != generation || closing) return;
                var present = usb.Select(d => d.Path).ToHashSet();
                foreach (var path in usbPaths.Where(p => !present.Contains(p))) state.Remove(path);
                usbPaths.Clear();
                foreach (var item in usb)
                {
                    usbPaths.Add(item.Path);
                    if (!state.Accept(item.Path, item.Bytes, Now, item.Priority)) status = "USB 데이터 미지원 또는 오래된 응답";
                    else if (item.Bytes.AsSpan(2, 8).IndexOfAnyExcept((byte)0) < 0) status = "동글 연결됨 · 키보드 연결 끊김";
                }
            }
            catch (Exception ex) { foreach (var path in usbPaths) state.Remove(path); usbPaths.Clear(); status = $"USB: {ex.Message}"; }
            Paint();
            if (!bleRefreshing) _ = RefreshBle();
        }
        catch (Exception ex) { status = $"장치 검색: {ex.Message}"; }
        finally
        {
            refreshing = false;
            if (!closing) { Paint(); if (refreshAgain) { refreshAgain = false; _ = Refresh(); } }
        }
    }
    private async Task RefreshBle()
    {
        bleRefreshing = true;
        try { await ble.Refresh(); }
        catch (Exception ex) { if (!closing) status = $"Bluetooth 검색: {ex.Message}"; }
        finally { bleRefreshing = false; if (!closing) Paint(); }
    }
    private void Paint()
    {
        if (closing) return;
        menu.Items.Clear();
        var devices = state.Devices(Now);
        var tooltip = "NocFree · --";
        for (int i = 0; i < devices.Length; i++)
        {
            var e = devices[i];
            string output = e.Value.Output switch { 0 => "유선", 1 => "Bluetooth", 2 => "동글", _ => "출력 알 수 없음" };
            menu.Items.Add($"NocFree{(devices.Length > 1 ? $" {i + 1}" : "")} · {output}");
            string left = BatteryState.Side(e, false, Now), right = BatteryState.Side(e, true, Now);
            menu.Items.Add($"왼쪽   {left}"); menu.Items.Add($"오른쪽 {right}");
            var ages = new List<double>();
            if (BatteryState.Fresh(e, false, Now)) ages.Add(e.Value.LeftAge + Now - e.Received);
            if (BatteryState.Fresh(e, true, Now)) ages.Add(e.Value.RightAge + Now - e.Received);
            if (ages.Count != 0) menu.Items.Add($"마지막 측정 {Math.Floor(ages.Max())}초 전");
            menu.Items.Add(new ToolStripSeparator());
            if (i == 0) tooltip = $"NocFree · L {left} / R {right}";
        }
        if (devices.Length == 0) menu.Items.Add("연결 장치 없음 · 배터리 펌웨어 필요");
        if (status.Length != 0) menu.Items.Add(status);
        menu.Items.Add("새로 고침", null, (_, _) => _ = Refresh());
        var startup = new ToolStripMenuItem("로그인 시 실행") { Checked = StartupEnabled() };
        startup.Click += (_, _) => {
            try
            {
                using var key = Registry.CurrentUser.CreateSubKey(@"Software\Microsoft\Windows\CurrentVersion\Run");
                if (StartupEnabled()) key.DeleteValue("NocFreeBattery", false);
                else key.SetValue("NocFreeBattery", $"\"{Environment.ProcessPath}\"");
            }
            catch (Exception ex) { status = $"로그인 설정: {ex.Message}"; }
            Paint();
        };
        menu.Items.Add(startup);
        menu.Items.Add("종료", null, (_, _) => ExitThread());
        tray.Text = tooltip.Length > 63 ? tooltip[..63] : tooltip;
    }
    private static bool StartupEnabled()
    {
        using var key = Registry.CurrentUser.OpenSubKey(@"Software\Microsoft\Windows\CurrentVersion\Run");
        return key?.GetValue("NocFreeBattery") is string;
    }
    protected override void ExitThreadCore()
    {
        closing = true; timer.Stop(); timer.Dispose();
        SystemEvents.PowerModeChanged -= PowerChanged;
        ble.Dispose(); tray.Visible = false; tray.Dispose(); menu.Dispose(); window.Dispose();
        base.ExitThreadCore();
    }
    private sealed class DeviceWindow(Action changed) : Form
    {
        protected override void SetVisibleCore(bool value) => base.SetVisibleCore(false);
        protected override void WndProc(ref Message m)
        {
            if (m.Msg == 0x219) changed(); // WM_DEVICECHANGE: re-enumerate battery collections.
            base.WndProc(ref m);
        }
    }
}
