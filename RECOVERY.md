# DFU 및 역할별 순정 원복

NocFree &에는 외부 리셋 버튼이 없습니다. 현재 Rust 키맵에는 공장 펌웨어의
`Option+3` 복구 조합도 없습니다. 확인되지 않은 PCB 패드를 임의로 쇼트하거나
전원 재연결을 reset-pin double tap과 같다고 가정하면 안 됩니다.

## 보호 영역

- `0x00000..0x26fff`: MBR + S140 7.3.0 — 보존
- `0x27000..0x64fff`: Rust 애플리케이션 — 기록 가능
- `0x65000..0x69fff`: BLE host profiles
- `0x6a000..0x6afff`: selected profile/settings
- `0x6b000..0x6bfff`: split bond
- `0x6c000..0x6cfff`: Link keymap/hotkeys
- `0x6d000..0x73fff`: 공장 파일시스템 — 보존
- `0x74000..0x7ffff`: Adafruit UF2 부트로더/메타데이터 — 보존

최신 Rust UF2 실제 범위는 왼쪽 `0x27000..0x3acff`, 오른쪽
`0x27000..0x329ff`입니다. 빌드 테스트가 보호 영역 침범 여부를 검사합니다.

## 역할 식별자

COM 번호는 바뀌므로 고정값으로 사용하지 마십시오. 포트의
`DEVPKEY_Device_Parent`를 아래 부모 ID와 비교합니다.

| 역할/상태 | USB 부모 Instance ID |
|---|---|
| Rust 왼쪽 | `USB\VID_2886&PID_8029\RUST-LEFT` |
| Rust 오른쪽 | `USB\VID_1D50&PID_615E\RUST-RIGHT` |
| 순정 왼쪽 | `USB\VID_2886&PID_8029\52CF50988BD1E6EE` |
| 순정 오른쪽 | `USB\VID_239A&PID_80D8\D82A03513BB02626` |
| Rust UF2 bootloader CDC 왼쪽 | `USB\VID_239A&PID_0029\52CF50988BD1E6EE` |
| Rust UF2 bootloader CDC 오른쪽 | `USB\VID_239A&PID_0029\D82A03513BB02626` |
| 순정 serial-DFU CDC 왼쪽 | `USB\VID_239A&PID_002A\52CF50988BD1E6EE` |
| 순정 serial-DFU CDC 오른쪽 | `USB\VID_239A&PID_002A\D82A03513BB02626` |

## DFU 진입 경로

- **왼쪽 독립 경로**: 왼쪽 Rust CDC를 1200 baud로 touch
- **오른쪽 독립 경로**: 오른쪽 Rust CDC를 1200 baud로 touch
- **왼쪽 단축키**: `Fn+5` 3초 홀드
- **오른쪽 단축키**: split이 정상일 때 `Fn+0` 3초 홀드
- `Fn+Esc`는 왼쪽 애플리케이션 재시작이며 DFU가 아님
- 미처리 panic은 GPREGRET `0x57`을 설정하고 UF2로 재시작

1200-baud touch 직후 포트가 사라져 `.NET SerialPort.Open()`이나 `Close()`가
`없는 장치` 예외를 내는 것은 실제 reset이 먼저 완료된 경우가 있습니다. 예외만
보고 재시도하지 말고 현재 부모 ID, UF2 볼륨, bootloader CDC를 먼저 확인합니다.
왼쪽 1200-baud 복구를 검증할 때는 물리 스위치를 가운데 **Wired**에 둡니다.
P3.1 실기에서는 Bluetooth 위치의 두 시도가 앱으로 돌아왔고, Wired 위치에서
`PID_0029` CDC와 UF2 볼륨이 확인된 뒤 같은 이미지를 복구했습니다.

## 최신 Rust UF2/DFU

| 역할 | UF2 SHA-256 | DFU ZIP SHA-256 |
|---|---|---|
| 왼쪽 | `031F4FE1299F439153A405358B52E5C92E7D3E68B5F0DB803918FE1798699DF3` | `DF597EA1650313A2917B4E2956EDDF801B0D165C470D7B6C1B817D137F947FFB` |
| 오른쪽 | `453BC8AAB3A762C9A9CBC6A7E6D4159B97FEAE55DE5D79CF1E3C0C075FCBDACE` | `A181DCA39D8E6E7CCDFAAB3C3DB7DF72FF109A8FBB1E6FC091BC5D38BF34106F` |

UF2 볼륨에서는 `INFO_UF2.TXT`가 다음을 포함하는지 확인합니다.

```text
UF2 Bootloader 0.9.2-39-g0147d71
Model: NocFree &
Board-ID: NocFree &
SoftDevice: S140 7.3.0
```

그 뒤 역할에 맞는 UF2만 복사합니다. 양쪽 UF2 드라이브는 모델 문자열이 같으므로
어떤 CDC를 reset했는지, 반대쪽 앱이 계속 존재하는지를 함께 가드해야 합니다.

## 순정 V2.3.0 원복 파일

| 역할 | 파일 | SHA-256 |
|---|---|---|
| 왼쪽 | `D:\study\nocfree\NocFree_and_V2.3.0_Left_ANSI.uf2` | `A3FF612B94E9CE0C12BEFD9FF19ECDE5D6E4DB964C0345E432B71ED7A2C5BC91` |
| 오른쪽 | `D:\study\nocfree\NocFree_and_V2.3.0_Right_ANSI.uf2` | `E1F851906B3E35117B8A8AAC09E5C8273D75921F64D1D3B1496E60A53D3E1C66` |

두 파일은 nRF52833 family `0x621e937a`, 앱 시작 `0x27000`으로 검증됐습니다.
역할을 바꿔 복사하면 안 됩니다.

## 검증된 전체 원복 순서

2026-08-21 양쪽 모두 다음 사이클을 실제 통과했습니다.

1. 대상 Rust 부모 ID와 CDC parent 확인
2. 반대쪽 Rust 부모 ID가 존재하는지 가드
3. 대상 CDC 1200 touch
4. NocFree & / S140 7.3.0 UF2 볼륨 확인
5. 역할별 순정 V2.3.0 UF2 SHA-256 확인 후 복사
6. 순정 부모 ID와 칩 시리얼 확인
7. 순정 CDC 1200 touch
8. 정확한 칩 시리얼의 `PID_002A` bootloader CDC 확인
9. 그 bootloader COM에 역할별 Rust DFU ZIP 전송
10. `RUST-LEFT`/`RUST-RIGHT` 복귀와 실제 입력 확인

순정 펌웨어에서 1200 touch한 뒤 이 PC에서는 UF2 대용량 저장 장치 대신
bootloader CDC만 열거됐습니다. 이때만 다음처럼 serial DFU를 실행합니다.

```powershell
$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
$package = 'D:\study\nocfree\NocFree-and-rust\firmware\NocFree_Rust_Right_DFU.zip'
$bootloaderPort = 'COM13' # 예시: 반드시 위 부모 ID로 당시에 다시 찾을 것
& adafruit-nrfutil dfu serial --package $package --port $bootloaderPort --baudrate 115200
$dfuExit = $LASTEXITCODE
if ($dfuExit -ne 0) { throw ('serial DFU failed with exit code {0}' -f $dfuExit) }
```

### `adafruit-nrfutil` 주의

Rust 앱 COM에 `--touch 1200`을 붙여 바로 전송하려 한 시험에서는 도구가
`No data received on serial port`로 실패했는데도 exit code 0을 반환했습니다.
따라서 다음을 지킵니다.

- Rust 앱에서는 1200 touch 후 UF2를 직접 복사하는 경로를 우선 사용
- serial DFU는 정확한 `PID_002A` bootloader COM을 찾은 뒤 실행
- `$LASTEXITCODE`뿐 아니라 `Device programmed.` 출력과 최종 Rust 부모 ID 확인
- 실패 출력이 있으면 성공으로 기록하지 않음

## `Fn+Delete` 복구

최신 이미지에서 다음을 실기 검증했습니다.

1. 사용자가 `Fn+Delete`
2. 오른쪽 `RUST-RIGHT` 소멸
3. `F:\INFO_UF2.TXT`의 NocFree & / S140 확인
4. 최신 오른쪽 UF2 복사
5. 오른쪽 앱과 split 자동 복귀
6. 케이블 전원 재인가 없이 오른쪽 `jkluiop` 입력

순정 왕복 중에는 외부 PCA9555/I2C 버스가 전원 유지 상태에서 한 번 걸렸고
오른쪽 케이블 재연결로 풀렸습니다. 이후 양쪽 펌웨어에 TWIM 생성 전 표준 I2C
bus clear(최대 9 clock + STOP)를 추가했습니다. 최신 재플래시와 `Fn+Delete`
복귀에서는 물리 전원 재인가가 필요하지 않았습니다.

## 즉시 중단 조건

- 예상한 반쪽이 아닌 앱이 사라짐
- 두 개 이상의 NocFree UF2 볼륨이 보여 역할을 확정할 수 없음
- 역할과 다른 UF2/DFU 파일 또는 SHA-256 불일치
- flash 후 목표 Rust 부모 ID가 돌아오지 않음
- 키가 계속 눌린 상태로 보고됨
- 기기가 뜨거워짐
- 부트로더와 앱 어느 쪽에서도 장치를 찾을 수 없음

어느 하나라도 발생하면 다른 반쪽을 건드리지 않습니다. 사용자의 물리 조작이
필요하면 `tools/notify-user.ps1`로 Windows 알림을 띄운 뒤 polling하지 말고
사용자의 다음 메시지를 기다립니다.
