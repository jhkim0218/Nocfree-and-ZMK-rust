# DFU 및 원복 절차

Rust UF2는 다음 영역만 사용합니다.

- `0x00000..0x26fff`: MBR + S140 7.3.0 — 보존
- `0x27000..0x64fff`: Rust 애플리케이션 — 기록
- `0x65000..0x6bfff`: BLE 프로필과 split bond 저장
- `0x6c000..0x6cfff`: Link 동적 키맵과 hotkey 저장
- `0x6d000..0x73fff`: 공장 파일시스템 — 보존
- `0x74000..0x7ffff`: Adafruit UF2 부트로더와 메타데이터 — 보존

## 2026-08-21 현재 왼쪽 복구 상태

현재 왼쪽에는 Link USB 인터페이스 수 설정이 빠진 중간 Rust 이미지가 설치돼
있습니다. 이 이미지는 USB builder에서 panic하므로 키 스캐너, CDC 1200-baud
복구, split 처리가 시작되지 않습니다.

- NocFree &에는 외부 reset 버튼이 없습니다.
- 현재 Rust 키맵에는 공장 펌웨어의 `Option+3` 복구 단축키가 없습니다.
- 전원 재연결은 Adafruit 부트로더가 요구하는 reset-pin double tap과 같다고
  입증되지 않았습니다.
- 현재 왼쪽은 USB/CDC/UF2로 열거되지 않으므로 소프트웨어만으로 진입시킬 수
  없습니다.
- 확인되지 않은 PCB 패드를 임의로 쇼트하지 마십시오.

왼쪽을 한 번 복구하려면 NocFree가 지정한 reset/recovery test point를 정확히
확인하거나 J-Link/nRF52 DK 같은 SWD 프로브가 필요합니다. 복구 후 설치할 최신
왼쪽 이미지에는 USB interface-count 수정과 panic→UF2 안전장치가 모두 들어
있습니다.

## 반드시 지킬 순서

1. 플래시할 반쪽의 UF2 드라이브에서 `INFO_UF2.TXT`를 확인합니다.
2. Board-ID가 NocFree이고 부트로더가 Adafruit nRF52 UF2인지 확인합니다.
3. **왼쪽 UF2만 먼저** 플래시합니다.
4. 왼쪽이 USB HID와 CDC로 열거되고 키 입력이 되는지 확인합니다.
5. 왼쪽 CDC를 1200 baud로 열어 UF2 드라이브가 다시 나타나는지 확인합니다.
6. 준비한 왼쪽 공장 UF2를 복사해 실제 원복까지 확인합니다.
7. 왼쪽 Rust UF2를 다시 플래시한 뒤에만 오른쪽으로 진행합니다.
8. 오른쪽도 Rust UF2 → CDC 1200 baud → 공장 UF2 원복을 같은 방식으로
   검증합니다.

## DFU 진입 경로

- **왼쪽**: USB CDC 1200-baud touch. `Fn+Esc`는 일반 재시작일 뿐입니다.
- **오른쪽**: USB CDC 1200-baud touch가 독립 복구 경로입니다.
- **오른쪽 보조 경로**: 두 반쪽의 split이 정상일 때 `Fn+Delete`.

Windows에서는 대상 COM 포트를 정확히 확인한 뒤, 1200 baud로 한 번 열었다
닫는 도구를 사용합니다. COM 번호만 보고 판단하지 말고 USB serial number로
왼쪽과 오른쪽을 구분해야 합니다.

## 이 작업 공간에서 검증한 역할 식별자

COM 번호는 다시 연결할 때 바뀔 수 있으므로 아래 고정 식별자를 사용합니다.

| 역할/상태 | USB 부모 Instance ID |
|---|---|
| 최신 Rust 왼쪽 목표 | `USB\VID_2886&PID_8029\RUST-LEFT` |
| Rust 오른쪽 | `USB\VID_1D50&PID_615E\RUST-RIGHT` |
| 순정 왼쪽 | `USB\VID_2886&PID_8029\52CF50988BD1E6EE` |
| 순정 오른쪽 | `USB\VID_239A&PID_80D8\D82A03513BB02626` |
| 부트로더 CDC 왼쪽 | `USB\VID_239A&PID_002A\52CF50988BD1E6EE` |
| 부트로더 CDC 오른쪽 | `USB\VID_239A&PID_002A\D82A03513BB02626` |

사용자가 준비한 역할별 순정 이미지의 검증된 SHA-256은 다음과 같습니다.

- `NocFree_and_V2.3.0_Left_ANSI.uf2`:
  `A3FF612B94E9CE0C12BEFD9FF19ECDE5D6E4DB964C0345E432B71ED7A2C5BC91`
- `NocFree_and_V2.3.0_Right_ANSI.uf2`:
  `E1F851906B3E35117B8A8AAC09E5C8273D75921F64D1D3B1496E60A53D3E1C66`

2026-08-21에 이전 체크포인트 Rust 이미지로 양쪽 모두 다음 전체 사이클을 실제
통과했습니다. 최신 Link 호환 이미지는 아직 같은 실기 사이클을 통과하지
않았습니다.

1. Rust CDC 1200-baud DFU로 UF2 드라이브 진입
2. 역할에 맞는 순정 UF2 복사와 위 순정 부모 ID 확인
3. 순정 CDC 1200-baud touch로 부트로더 CDC(`PID_002A`) 진입
4. 역할에 맞는 Rust application-only serial DFU 패키지 업로드
5. `RUST-LEFT`/`RUST-RIGHT` 복귀와 좌우 입력 확인

순정 펌웨어에서 1200-baud touch 후에는 이 PC에서 UF2 대용량 저장 장치가
마운트되지 않고 부트로더 CDC만 열거됐습니다. 이 경우 설치된 공식
`adafruit-nrfutil` 0.5.3.post16을 사용합니다.

```powershell
$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
& adafruit-nrfutil dfu serial --package 'D:\study\nocfree\NocFree-and-rust\firmware\NocFree_Rust_Right_DFU.zip' --port COM13 --baudrate 115200
$dfuExit = $LASTEXITCODE
if ($dfuExit -ne 0) { throw ('serial DFU failed with exit code {0}' -f $dfuExit) }
```

COM 번호는 예시일 뿐입니다. 위 표의 부모 ID로 역할을 확인한 현재 포트를
사용하십시오. 패키지는 애플리케이션만 기록하며 검증된 SHA-256은 다음과
같습니다.

- `NocFree_Rust_Left_DFU.zip`:
  `4A70FE90CD0B00069ACB0745A308A4C2E7B10057B8CC7A7C0586C9D3DC28F0DE`
- `NocFree_Rust_Right_DFU.zip`:
  `4D19A3DED7BBDB47CD448ADECCF50265D81DC09748492DDAFBB4C57EA3DA1535`

## 즉시 중단 조건

- 예상한 반쪽이 아닌 장치가 사라지거나 UF2 드라이브로 전환됨
- 플래시 후 USB CDC가 전혀 열거되지 않음
- 키가 계속 눌린 상태로 보고됨
- 기기가 뜨거워짐
- UF2 드라이브가 다시 나타나지 않음

이 중 하나라도 발생하면 다른 반쪽을 플래시하지 않습니다. UF2 드라이브가
남아 있다면 해당 역할의 공장 UF2를 즉시 복사해 원복합니다.
