$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

Add-Type -AssemblyName System.Windows.Forms
Add-Type -AssemblyName System.Drawing

$keys = [System.Windows.Forms.Keys]
$steps = @(
    @{ Label = 'Esc'; Codes = @($keys::Escape) }
    @{ Label = 'F1'; Codes = @($keys::F1) }
    @{ Label = 'F2'; Codes = @($keys::F2) }
    @{ Label = 'F3'; Codes = @($keys::F3) }
    @{ Label = 'F4'; Codes = @($keys::F4) }
    @{ Label = 'F5'; Codes = @($keys::F5) }
    @{ Label = 'F6'; Codes = @($keys::F6) }
    @{ Label = 'F7'; Codes = @($keys::F7) }
    @{ Label = 'F8'; Codes = @($keys::F8) }
    @{ Label = 'F9'; Codes = @($keys::F9) }
    @{ Label = 'F10'; Codes = @($keys::F10) }
    @{ Label = 'F11'; Codes = @($keys::F11) }
    @{ Label = 'F12'; Codes = @($keys::F12) }
    @{ Label = 'Print Screen'; Codes = @($keys::PrintScreen, $keys::Snapshot) }
    @{ Label = 'Home'; Codes = @($keys::Home) }
    @{ Label = 'Backspace'; Codes = @($keys::Back) }
    @{ Label = 'Page Up'; Codes = @($keys::PageUp, $keys::Prior) }
    @{ Label = 'Tab'; Codes = @($keys::Tab) }
    @{ Label = 'Caps Lock'; Codes = @($keys::CapsLock, $keys::Capital) }
    @{ Label = 'Delete'; Codes = @($keys::Delete) }
    @{ Label = '왼쪽 Shift'; Codes = @($keys::ShiftKey, $keys::LShiftKey) }
    @{ Label = '오른쪽 Shift'; Codes = @($keys::ShiftKey, $keys::RShiftKey) }
    @{ Label = '위쪽 화살표'; Codes = @($keys::Up) }
    @{ Label = 'Page Down'; Codes = @($keys::PageDown, $keys::Next) }
    @{ Label = '왼쪽 Ctrl'; Codes = @($keys::ControlKey, $keys::LControlKey) }
    @{ Label = '왼쪽 Alt'; Codes = @($keys::Menu, $keys::LMenu) }
    @{ Label = '왼쪽 Windows'; Codes = @($keys::LWin) }
    @{ Label = '왼쪽 Space'; Codes = @($keys::Space) }
    @{ Label = '오른쪽 Space'; Codes = @($keys::Space) }
    @{ Label = '오른쪽 Windows'; Codes = @($keys::RWin) }
    @{ Label = '오른쪽 Alt'; Codes = @($keys::Menu, $keys::RMenu) }
    @{ Label = '왼쪽 화살표'; Codes = @($keys::Left) }
    @{ Label = '아래쪽 화살표'; Codes = @($keys::Down) }
    @{ Label = '오른쪽 화살표'; Codes = @($keys::Right) }
)

$script:index = 0
$script:held = [System.Collections.Generic.HashSet[System.Windows.Forms.Keys]]::new()

$form = [System.Windows.Forms.Form]::new()
$form.Text = 'NocFree 비문자 키 테스트'
$form.ClientSize = [System.Drawing.Size]::new(640, 300)
$form.StartPosition = [System.Windows.Forms.FormStartPosition]::CenterScreen
$form.TopMost = $true
$form.KeyPreview = $true

$title = [System.Windows.Forms.Label]::new()
$title.AutoSize = $false
$title.Bounds = [System.Drawing.Rectangle]::new(20, 20, 600, 40)
$title.Font = [System.Drawing.Font]::new('Segoe UI', 16, [System.Drawing.FontStyle]::Bold)
$title.TextAlign = [System.Drawing.ContentAlignment]::MiddleCenter
$form.Controls.Add($title)

$instruction = [System.Windows.Forms.Label]::new()
$instruction.AutoSize = $false
$instruction.Bounds = [System.Drawing.Rectangle]::new(20, 75, 600, 90)
$instruction.Font = [System.Drawing.Font]::new('Segoe UI', 28, [System.Drawing.FontStyle]::Bold)
$instruction.TextAlign = [System.Drawing.ContentAlignment]::MiddleCenter
$form.Controls.Add($instruction)

$status = [System.Windows.Forms.Label]::new()
$status.AutoSize = $false
$status.Bounds = [System.Drawing.Rectangle]::new(20, 180, 600, 45)
$status.Font = [System.Drawing.Font]::new('Segoe UI', 12)
$status.TextAlign = [System.Drawing.ContentAlignment]::MiddleCenter
$form.Controls.Add($status)

$closeButton = [System.Windows.Forms.Button]::new()
$closeButton.Bounds = [System.Drawing.Rectangle]::new(250, 240, 140, 40)
$closeButton.Text = '완료 후 닫기'
$closeButton.Enabled = $false
$closeButton.Add_Click({ $form.Close() })
$form.Controls.Add($closeButton)

function Update-Instruction {
    if ($script:index -ge $steps.Count) {
        $title.Text = ('통과: {0}/{0}' -f $steps.Count)
        $instruction.Text = '모든 비문자 키 통과'
        $instruction.ForeColor = [System.Drawing.Color]::DarkGreen
        $status.Text = '완료 후 닫기를 누르고 Codex에 nontextok를 입력하세요.'
        $closeButton.Enabled = $true
        return
    }

    $title.Text = ('진행: {0}/{1}' -f $script:index, $steps.Count)
    $instruction.Text = [string]$steps[$script:index].Label
    $instruction.ForeColor = [System.Drawing.Color]::Black
    $status.Text = '표시된 물리 키를 한 번 눌렀다가 떼세요.'
}

$form.Add_PreviewKeyDown({
    param($sender, $event)
    $event.IsInputKey = $true
})

$form.Add_KeyDown({
    param($sender, $event)
    $event.Handled = $true
    $event.SuppressKeyPress = $true
    if (-not $script:held.Add($event.KeyCode)) { return }
    if ($script:index -ge $steps.Count) { return }

    $expected = $steps[$script:index]
    if ($event.KeyCode -notin $expected.Codes) {
        $status.Text = ('다른 키 감지: {0}. 다시 {1} 키를 누르세요.' -f $event.KeyCode, $expected.Label)
        $status.ForeColor = [System.Drawing.Color]::DarkRed
        return
    }

    $status.ForeColor = [System.Drawing.Color]::Black
    $script:index += 1
    Update-Instruction
})

$form.Add_KeyUp({
    param($sender, $event)
    $event.Handled = $true
    $event.SuppressKeyPress = $true
    [void]$script:held.Remove($event.KeyCode)
})

Update-Instruction
[void]$form.ShowDialog()
$form.Dispose()

if ($script:index -ne $steps.Count) {
    throw ('Key sweep closed before completion: {0}/{1}' -f $script:index, $steps.Count)
}

Write-Output ('Key sweep completed: {0}/{0}' -f $steps.Count)
