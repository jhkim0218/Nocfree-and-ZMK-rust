$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

$message = if ($args.Count -gt 0) { [string]$args[0] } else { 'Codex에서 확인이 필요합니다.' }

Add-Type -AssemblyName System.Windows.Forms
Add-Type -AssemblyName System.Drawing

$form = [System.Windows.Forms.Form]::new()
$form.Text = 'Codex 확인 요청'
$form.ClientSize = [System.Drawing.Size]::new(640, 220)
$form.StartPosition = [System.Windows.Forms.FormStartPosition]::CenterScreen
$form.TopMost = $true
$form.ShowInTaskbar = $true

$label = [System.Windows.Forms.Label]::new()
$label.AutoSize = $false
$label.Bounds = [System.Drawing.Rectangle]::new(25, 25, 590, 110)
$label.Font = [System.Drawing.Font]::new('Segoe UI', 14)
$label.TextAlign = [System.Drawing.ContentAlignment]::MiddleCenter
$label.Text = $message
$form.Controls.Add($label)

$button = [System.Windows.Forms.Button]::new()
$button.Bounds = [System.Drawing.Rectangle]::new(250, 155, 140, 42)
$button.Text = '확인'
$button.DialogResult = [System.Windows.Forms.DialogResult]::OK
$form.AcceptButton = $button
$form.Controls.Add($button)

$timer = [System.Windows.Forms.Timer]::new()
$timer.Interval = 120000
$timer.Add_Tick({ $form.Close() })
$form.Add_Shown({
    [System.Media.SystemSounds]::Exclamation.Play()
    $form.Activate()
    $form.BringToFront()
    $timer.Start()
})

[void]$form.ShowDialog()
$timer.Dispose()
$form.Dispose()

Write-Output ('Notification dismissed: {0}' -f $message)
