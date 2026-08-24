param(
    [Parameter(Mandatory = $true)]
    [ValidateSet('Left', 'Right')]
    [string]$Role
)

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

$deviceId = if ($Role -eq 'Left') { 'VID_2886&PID_8029' } else { 'VID_1D50&PID_615E' }
$devices = Get-CimInstance Win32_PnPEntity | Where-Object {
    $_.ConfigManagerErrorCode -eq 0 -and
    $_.PNPDeviceID -like ('*{0}*' -f $deviceId) -and
    $_.Name -match '\(COM\d+\)'
}
if (@($devices).Count -ne 1) {
    throw ('Expected one connected {0} diagnostic port, found {1}' -f $Role, @($devices).Count)
}

$match = [regex]::Match([string]$devices[0].Name, '\((COM\d+)\)')
if (-not $match.Success) {
    throw ('Could not parse COM port from {0}' -f $devices[0].Name)
}
$portName = $match.Groups[1].Value

$eventNames = @{
    1 = 'scan-start'
    2 = 'advertisement-found'
    3 = 'scan-error'
    4 = 'connect-start'
    5 = 'connected'
    6 = 'connect-error'
    7 = 'connection-parameters'
    8 = 'security-start'
    9 = 'security-ok'
    10 = 'security-error'
    11 = 'gatt-start'
    12 = 'gatt-ok'
    13 = 'gatt-error'
    14 = 'split-ready'
    15 = 'disconnected'
    16 = 'advertising'
    17 = 'advertising-error'
    18 = 'key-while-disconnected'
}

function Convert-Address {
    param([UInt64]$Data)

    $bytes = for ($index = 0; $index -lt 6; $index += 1) {
        '{0:X2}' -f (($Data -shr (8 * $index)) -band 0xFF)
    }
    $flags = ($Data -shr 48) -band 0xFF
    return ('identity={0} flags=0x{1:X2}' -f ($bytes -join ':'), $flags)
}

function Convert-ConnectionParameters {
    param([UInt64]$Data)

    $minimum = $Data -band 0xFFFF
    $maximum = ($Data -shr 16) -band 0xFFFF
    $latency = ($Data -shr 32) -band 0xFFFF
    $timeout = ($Data -shr 48) -band 0xFFFF
    return ('interval={0}-{1} units ({2:N2}-{3:N2} ms), latency={4}, timeout={5} units ({6:N0} ms)' -f
        $minimum, $maximum, ($minimum * 1.25), ($maximum * 1.25), $latency, $timeout, ($timeout * 10))
}

function Format-DiagnosticRecord {
    param([string]$Line)

    $fields = $Line.Split(',')
    if ($fields.Count -ne 5) {
        throw ('Malformed diagnostic record: {0}' -f $Line)
    }
    $timestamp = [UInt32]$fields[0]
    $event = [int]$fields[1]
    $arg = [int]$fields[2]
    $value = [UInt16]$fields[3]
    $data = [Convert]::ToUInt64($fields[4], 16)
    $name = if ($eventNames.ContainsKey($event)) { $eventNames[$event] } else { 'unknown' }

    $scanErrors = @{ 1 = 'timeout'; 2 = 'SoftDevice error' }
    $connectErrors = @{
        1 = 'timeout'
        2 = 'no address'
        3 = 'no free connection'
        4 = 'MTU exchange'
        5 = 'SoftDevice error'
    }
    $securityErrors = if ($Role -eq 'Left') {
        @{ 1 = 'encryption start'; 2 = 'pairing start'; 3 = 'security timeout' }
    }
    else {
        @{ 1 = 'security timeout'; 2 = 'disconnected before security result' }
    }
    $gattErrors = @{
        1 = 'disconnected during discovery'
        2 = 'service not found'
        3 = 'service incomplete'
        4 = 'GATT discovery'
        5 = 'SoftDevice error'
        6 = 'state notification setup'
        7 = 'battery notification setup'
    }
    $advertisingErrors = @{ 1 = 'timeout'; 2 = 'no free connection'; 3 = 'SoftDevice error' }

    $detail = switch ($event) {
        1 { 'attempt={0}' -f $value }
        2 { 'attempt={0}, RSSI={1} dBm, {2}' -f $value, $arg, (Convert-Address $data) }
        3 { '{0}, elapsed={1} ms' -f $scanErrors[$arg], $value }
        4 { 'attempt={0}, requested {1}' -f $value, (Convert-ConnectionParameters $data) }
        5 { 'elapsed={0} ms, {1}' -f $value, (Convert-Address $data) }
        6 { '{0}, elapsed={1} ms' -f $connectErrors[$arg], $value }
        7 { Convert-ConnectionParameters $data }
        8 { 'attempt={0}' -f $value }
        9 { 'elapsed={0} ms' -f $value }
        10 { '{0}, elapsed={1} ms' -f $securityErrors[$arg], $value }
        11 { 'attempt={0}' -f $value }
        12 { 'elapsed={0} ms' -f $value }
        13 { '{0}, elapsed={1} ms' -f $gattErrors[$arg], $value }
        14 { 'attempt={0}, total={1} ms' -f $data, $value }
        15 { 'HCI-reason=0x{0:X2}' -f ($arg -band 0xFF) }
        16 {
            $mode = if ($arg -eq 0) { 'fast' } else { 'idle' }
            'attempt={0}, mode={1}, interval={2} units ({3:N0} ms)' -f $data, $mode, $value, ($value * 0.625)
        }
        17 { '{0}, elapsed={1} ms' -f $advertisingErrors[$arg], $value }
        18 { 'pressed={0}, state=0x{1:X16}' -f $value, $data }
        default { 'arg={0}, value={1}, data=0x{2:X16}' -f $arg, $value, $data }
    }
    return ('[{0,10} ms] {1}: {2}' -f $timestamp, $name, $detail)
}

$serial = [System.IO.Ports.SerialPort]::new()
$serial.PortName = $portName
$serial.BaudRate = 115200
$serial.Parity = [System.IO.Ports.Parity]::None
$serial.DataBits = 8
$serial.StopBits = [System.IO.Ports.StopBits]::One
$serial.Handshake = [System.IO.Ports.Handshake]::None
$serial.DtrEnable = $true
$serial.ReadTimeout = 2000
$serial.NewLine = "`n"

try {
    $serial.Open()
    $header = $serial.ReadLine().Trim()
    $headerFields = $header.Split(',')
    if ($headerFields.Count -ne 4 -or $headerFields[0] -ne 'NFDIAG1') {
        throw ('Unexpected diagnostic header: {0}' -f $header)
    }
    $count = [int]$headerFields[2]
    $dropped = [UInt32]$headerFields[3]
    Write-Output ('NocFree {0} split diagnostics on {1}: {2} records, {3} overwritten' -f
        $Role, $portName, $count, $dropped)
    for ($index = 0; $index -lt $count; $index += 1) {
        Write-Output (Format-DiagnosticRecord $serial.ReadLine().Trim())
    }
}
finally {
    if ($serial.IsOpen) {
        $serial.Close()
    }
    $serial.Dispose()
}
