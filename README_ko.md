# NocFree-and-rust

[English](README.md)

NocFree & ANSI 키보드의 ZMK 동작을 nRF52833용 `no_std` Rust 펌웨어로
옮긴 프로젝트입니다. 원본 프로젝트는
[`NocFreeKB/NocFree-and-zmk`](https://github.com/NocFreeKB/NocFree-and-zmk)이며,
이 저장소는 독립적인 Rust 포팅입니다.

이 펌웨어는 **84키 NocFree & ANSI 모델만 지원**합니다. 다른 NocFree 모델과
ISO 배열은 지원하지 않습니다.

2026-08-23 현재 양쪽 최신 Rust 이미지가 실제 장치에 설치돼 있고 USB, BLE,
84개 물리 키, 물리 모드/전원 스위치, 단축키, NocFree Link 키 변경, 양쪽 DFU와
역할별 순정 원복을 통과했습니다. 다음 작업자는 [HANDOFF.md](HANDOFF.md)를 먼저
읽으십시오.

순정 firmware 비교, 추가 기능 우선순위와 배터리 보정 절차는
[ROADMAP_ko.md](ROADMAP_ko.md)에 정리돼 있습니다.

> [!IMPORTANT]
> 배터리 환산은 역분석한 순정 V2.3.0 알고리즘을 따르지만 전체 방전 주기와
> 소비전력 측정은 아직 남아 있습니다. 파란 페어링 LED와 빨간 저전압 LED
> 점멸은 구현했으나 충전/완충을 비롯한 나머지 순정 LED 상태는 미완성입니다.
> 공장 USB 동글/2.4G 기능도 구현·검증되지 않았습니다.
>
> 2026-08-24 실기에서 빠른 좌우 교차 입력, 책상 거리 split 재연결, 백라이트
> 수렴 문제를 다시 열었습니다. 이후 절대 백라이트 상태 동기화와 재연결 진단은
> 실기 검증을 통과했으며 입력 순서와 재연결 tuning은 미해결입니다.
> [PROGRESS.md](PROGRESS.md)를 참고하십시오.

## 역할

- **왼쪽 (`central`)**: 왼쪽 37키, 오른쪽 split 수신, 전체 84키 키맵,
  USB/BLE HID, BLE 멀티 페어링 슬롯 3개, NocFree Link를 담당합니다.
- **오른쪽 (`right`)**: 오른쪽 47키를 스캔해 암호화 BLE split으로 왼쪽에
  전달합니다. 오른쪽 USB는 HID가 아니라 독립 CDC 복구용입니다.

양쪽은 PCA9555 세 개(`0x20`, `0x22`, `0x24`)를 SDA P0.11/SCL P1.09에서
100 kHz로 읽습니다. active-low 입력, 5 ms debounce, 1M BLE PHY를 사용합니다.
부트로더나 다른 펌웨어가 I2C 전송 중 MCU를 재시작해도 외부 확장칩이 걸린 채
남지 않도록 TWIM 초기화 전에 최대 9회의 bus-clear clock과 STOP을 보냅니다.

## 빌드

저장소를 clone한 뒤 저장소 루트에서 Windows PowerShell을 여십시오. Rust MSVC
toolchain과 Python 3이 필요합니다. 아래 명령은 추가 빌드 의존성을 설치하고 양쪽
UF2를 생성합니다.

```powershell
$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

& rustup target add thumbv7em-none-eabihf
$targetExit = $LASTEXITCODE
if ($targetExit -ne 0) { throw ('rustup target add failed with exit code {0}' -f $targetExit) }

& rustup component add llvm-tools-preview
$componentExit = $LASTEXITCODE
if ($componentExit -ne 0) { throw ('rustup component add failed with exit code {0}' -f $componentExit) }

& python -m pip install 'adafruit-nrfutil==0.5.3.post16'
$pipExit = $LASTEXITCODE
if ($pipExit -ne 0) { throw ('dependency installation failed with exit code {0}' -f $pipExit) }

& pwsh -NoProfile -ExecutionPolicy Bypass -File '.\tools\build-release.ps1'
$buildExit = $LASTEXITCODE
if ($buildExit -ne 0) { throw ('build failed with exit code {0}' -f $buildExit) }
```

스크립트는 포맷, Windows 호스트 테스트, host/ARM Clippy, 양쪽 release 빌드,
BIN/UF2/serial-DFU ZIP 생성, 주소/family/vector/round-trip 및 ZIP 내부 BIN
일치를 모두 검사합니다. 최신 결과는 Rust 64개,
Python/계약/아티팩트 17개 테스트 통과입니다.

빌드가 성공하면 반드시 각 반쪽에 맞는 UF2만 사용하십시오.

- **왼쪽/central:** [`firmware/NocFree_And_Rust_ZMK_Based_ANSI_Left.uf2`](firmware/NocFree_And_Rust_ZMK_Based_ANSI_Left.uf2)
- **오른쪽/peripheral:** [`firmware/NocFree_And_Rust_ZMK_Based_ANSI_Right.uf2`](firmware/NocFree_And_Rust_ZMK_Based_ANSI_Right.uf2)

이 두 파일은 저장소에도 커밋되어 있어 직접 빌드하지 않고 내려받을 수 있습니다.
flash하기 전에 [RECOVERY.md](RECOVERY.md)를 먼저 읽으십시오.

키보드에서 실행되는 코드는 전부 Rust `no_std`입니다. `tools/*.py`와 Python
패키지인 `adafruit-nrfutil`은 PC에서 산출물을 만들고 검사할 때만 사용되며
키보드에는 들어가지 않습니다.

## 최신 산출물

| 파일 | 크기(bytes) | SHA-256 |
|---|---:|---|
| `firmware/NocFree_Rust_Left.bin` | 75,004 | `C84F695683E581981EA6406C20064791B24702825941BFC064D05E8AFA104EFB` |
| [`firmware/NocFree_And_Rust_ZMK_Based_ANSI_Left.uf2`](firmware/NocFree_And_Rust_ZMK_Based_ANSI_Left.uf2) | 150,016 | `5113421A1BAF1E9C5EE461F41506F5CAED4F3A561CD39E2F7D62C4FF9C8FF875` |
| `firmware/NocFree_Rust_Left_DFU.zip` | 75,880 | `366CF388A9F419E65AAAE7BAB14E9150A91605E25B27AEB89E1AD963EA74112B` |
| `firmware/NocFree_Rust_Right.bin` | 46,836 | `0E748A2F21B9F74FF5D72D052225A7F5E6423C23D9E687F72C1C2BA69B63630E` |
| [`firmware/NocFree_And_Rust_ZMK_Based_ANSI_Right.uf2`](firmware/NocFree_And_Rust_ZMK_Based_ANSI_Right.uf2) | 93,696 | `AA21211CE20625ED80EE3307DC2EF147D1EEAF373BE94AECEA24A75BCEEAEDA5` |
| `firmware/NocFree_Rust_Right_DFU.zip` | 47,718 | `B9EC950D2BBD6465582BB20499E86D90E2CDBC319AD4CF5AC904FFBFEF8BCE84` |

UF2는 앱 시작 `0x27000`부터 왼쪽 `0x394ff`, 오른쪽 `0x326ff`까지만
기록합니다. SoftDevice, 저장소, 공장 파일시스템과 UF2 부트로더는 보존합니다.

## Split 재연결 진단

각 반쪽은 최근 split 이벤트 32개를 RAM에 보관합니다. 해당 반쪽 USB를 연결한
뒤 Windows PowerShell에서 다음처럼 키보드 상태를 바꾸지 않고 읽을 수 있습니다.

```powershell
& '.\tools\read-split-diagnostics.ps1' -Role Left
& '.\tools\read-split-diagnostics.ps1' -Role Right
```

왼쪽 로그는 scan, 광고 identity/RSSI, 연결, 보안, GATT 탐색, split-ready,
해제 사유, 시도 횟수와 실제 연결 파라미터를 구분합니다. 오른쪽 로그는 광고
모드/간격, 연결/보안, 해제 사유와 미연결 상태 키 입력을 포함합니다. 진단은
115200 baud를 사용하며 기존 1200-baud DFU는 복구 전용으로 그대로 유지됩니다.

## NocFree Link 키 변경

왼쪽은 `link.nocfree.com`이 인식하는 VID/PID `2886:8029`, 제품명
`NocFree & ANSI`, WinUSB vendor bulk interface를 제공합니다.

- 8개 레이어 × 84개 물리 키
- 16개 hotkey 슬롯과 실제 HID chord 실행
- 키맵/hotkey CRC 포함 flash 영구 저장
- 단일 키 변경, 재부팅 후 보존, hotkey 생성/삭제, 기본값 복구 실기 통과

ZMK Studio 프로토콜은 구현하지 않았고, 요청된 두 경로 중 NocFree Link를
선택했습니다. quick text는 조회 시 빈 슬롯으로 응답해 웹앱 timeout을 막지만
저장·실행은 구현하지 않았습니다. Link의 배터리 조회도 현재 `0xff`(사용할 수
없음)를 반환하므로, 배터리 확인은 아래 `Fn+I`를 사용해야 합니다.

## 구현 상태

| 상태 | 기능 | 현재 범위와 제한 |
|---|---|---|
| 완료 | 84키 ANSI 입력 | 왼쪽 37키와 오른쪽 47키, 양쪽 Fn, 비문자 키와 한국어 Windows 특수키까지 실기 확인 |
| 완료 | USB/BLE HID | 왼쪽 USB HID, BLE HID, CCCD 즉시 저장·복원, USB↔BLE 전환과 같은 이미지에서의 BLE 자동 재연결 |
| 회귀 / 진단 완료 | 양쪽 split | 암호화 BLE 전달과 P2 단계별 진단은 동작하지만 책상 거리 재연결이 간헐적으로 민감하고 빠른 좌우 교차 입력 순서가 바뀔 수 있음 |
| 완료 | BLE 멀티 페어링 | 호스트 bond 슬롯 3개와 선택 상태 영구 저장. Windows 11과 Android 두 호스트로 슬롯 1/2 페어링 확인; 세 번째 호스트와 다른 OS는 미검증 |
| 완료 | 백라이트 | 왼쪽 기준 version 포함 절대 상태가 enabled·밝기·timeout·generation을 모든 변경과 재연결 때 오른쪽에 동기화하며 30초 소등과 첫 키 wake도 유지 |
| 완료 | 물리 스위치 | 왼쪽 Wired/Bluetooth 선택과 2.4G 위치의 안전한 무출력, 오른쪽 물리 전원 스위치 동작 |
| 완료 | NocFree Link 키맵 | 8×84 키, hotkey 16개, 실행·삭제·기본값 복구와 CRC 포함 flash 저장 |
| 완료 | 복구 | 양쪽 독립 CDC 1200-baud DFU, Fn DFU 단축키, Rust↔순정 V2.3.0 왕복 |
| 부분 | 배터리 | 양쪽 모두 순정 V2.3.0에서 복구한 ADC/divider 환산, 75/25 전압 필터, 2.31–3.30 V 퍼센트 곡선, 60초 주기 측정과 `Fn+I` 출력을 사용. 완충된 양쪽에서 100%를 확인했으며 전체 방전 주기와 DMM 검증은 남음 |
| 부분 | NocFree Link 호환 | 키맵과 hotkey는 동작. Link 배터리 표시는 미지원이며 quick text는 빈 조회만 지원하고 저장·삭제·실행은 미지원 |
| 부분 | 순정 전원 관리 | 배터리 divider는 측정할 때만 켜고 idle 키 스캔은 왼쪽 `P0.31`/오른쪽 `P0.05` PCA9555 interrupt와 250 ms 안전 스캔을 사용. 배터리 사용 중 NocFree 키 무입력 5분 후 왼쪽 central이 System OFF에 진입하며 USB 전원은 System OFF만 막고 30초 백라이트 timeout에는 영향을 주지 않음. 오른쪽은 자동 재연결을 위해 System ON idle을 유지하고 최초 split 연결 뒤 단절 광고 주기를 250 ms에서 1초로 늦춤. 충전 상태·sleep 전류·순정 수준 사용 시간은 아직 계측 검증하지 않음 |
| 미구현 | 공장 USB 동글 / 2.4 GHz 통신 | 동글 페어링과 입력은 동작하거나 검증된 상태가 아님. 공장 USB receiver, 왼쪽 외부 nRF24L01, 오른쪽/별도 numpad의 ESB 연결은 사용하지 않으며 현재 split은 BLE 전용 |
| 부분 | 물리 스위치 옆 LED | 왼쪽 파란 LED는 페어링부터 bond 완료 또는 프로필 선택까지 점멸. 양쪽 빨간 charger 공유 선은 open-drain 방식으로 10% 이하에서 0.5초 간격 점멸. 파란 LED는 실기 통과했고 빨간 LED는 자동 테스트만 통과했으며 양쪽이 완충이라 실물 점멸은 확인하지 못함. 충전/완충 등 나머지 순정 표시는 남음 |
| 미구현 | 기타 설정 경로 | ZMK Studio와 공장 firmware updater의 완전한 호환은 제공하지 않음 |

## 전원 및 깨우기 동작

- NocFree 키 무입력 30초 후 USB 전원 연결 여부와 관계없이 양쪽 백라이트가 꺼집니다. 다음 키 입력으로 백라이트가 켜지며 해당 입력도 정상적으로 전달됩니다.
- 배터리 사용 중 NocFree 키 무입력 5분 후 왼쪽 central이 nRF52 System OFF에 진입합니다. 진입 전에 양쪽 백라이트 소등을 요청하며, USB 전원이 연결되어 있거나 BLE 페어링 중이면 System OFF에 들어가지 않습니다.
- 잠든 왼쪽은 USB를 연결하거나 키보드가 다시 연결될 때까지 왼쪽 키 하나를 길게 눌러 깨울 수 있습니다. System OFF wake는 MCU reset을 동반하므로 아주 짧게 누른 wake 키 문자는 부팅 중 소모될 수 있습니다.
- 오른쪽은 BLE 전용 split 자동 재연결을 위해 저활동 System ON을 유지합니다. 최초 split 연결 뒤 단절 광고 주기를 250 ms에서 1초로 늦추지만, 오른쪽 System OFF나 실제 소비전류 감소량은 구현·검증됐다고 주장하지 않습니다.

## 전체 기본 단축키

왼쪽 Fn과 오른쪽 Fn은 같은 레이어를 사용합니다. 아래 표는 NocFree Link에서
키를 바꾸기 전의 firmware 기본값입니다.

| 키 | 누르는 방법 | 동작 |
|---|---|---|
| `Fn+Esc` | 즉시 | 왼쪽 애플리케이션 재시작. DFU가 아님 |
| `Fn+F1` / `Fn+F2` | 즉시 | 화면 밝기 내림 / 올림 |
| `Fn+F3` | 즉시 | macOS: Mission Control, Windows: Task View |
| `Fn+F4` | 즉시 | macOS: Spotlight, Windows: Search |
| `Fn+F5` / `Fn+F6` | 즉시 | 양쪽 백라이트 밝기 20% 내림 / 올림 |
| `Fn+F7` / `F8` / `F9` | 즉시 | 이전 곡 / 재생·일시정지 / 다음 곡 |
| `Fn+F10` / `F11` / `F12` | 즉시 | 음소거 / 볼륨 내림 / 볼륨 올림 |
| `Fn+1` / `Fn+2` / `Fn+3` | 짧게 | BLE 페어링 슬롯 1 / 2 / 3 선택 |
| `Fn+1` / `Fn+2` / `Fn+3` | 1초 홀드 | 해당 슬롯의 bond를 삭제하고 새 호스트 페어링 시작 |
| `Fn+5` | 3초 홀드 | **왼쪽** UF2 부트로더 진입. 짧게 누르면 무동작 |
| `Fn+0` | 3초 홀드 | **오른쪽** UF2 부트로더 진입. 짧게 누르면 무동작 |
| `Fn+Tab` | 즉시 | 양쪽 백라이트 켜기 / 끄기 |
| `Fn+I` | 3초 홀드 | `L {왼쪽 퍼센트} R {오른쪽 퍼센트}`를 현재 출력으로 입력. 짧게 누르면 무동작 |
| `Fn+Delete` | 3초 홀드 | 호환용 **오른쪽** DFU 별칭. 새 조작은 `Fn+0` 권장 |
| `Fn+M` | 짧게 / 1초 홀드 | 짧게 M 입력 / 홀드하면 macOS 모드 영구 저장 |
| `Fn+N` | 짧게 / 1초 홀드 | 짧게 N 입력 / 홀드하면 Windows 모드 영구 저장 |

표에 없는 `Fn+키`는 별도 기능이 없는 transparent 동작이라 원래 키가
입력됩니다. 특히 기준 ZMK의 `Fn+6`(활성 프로필 삭제)과 `Fn+7`(USB/BLE
toggle)은 Rust에서 같은 위치에 구현하지 않았습니다. 슬롯 삭제·페어링은
대상 `Fn+1/2/3`을 1초 홀드하고, 출력 선택은 왼쪽 물리 스위치를 사용하십시오.
`Fn+U`와 `Fn+B`는 출력 전환 기능이 없으며 각각 원래 문자로 동작합니다.
DFU는 오입력 방지를 위해 기준 ZMK의 즉시 실행 대신 3초 홀드로
변경했습니다.

왼쪽 DFU와 split이 끊긴 오른쪽 DFU는 각 반쪽의 CDC 1200-baud touch를
사용합니다. NocFree &에는 외부 리셋 버튼이 없습니다. 확인되지 않은 PCB
패드를 쇼트하지 마십시오. 정확한 절차는 [RECOVERY.md](RECOVERY.md)에 있습니다.

## 물리 스위치

왼쪽 3단 스위치는 P0.15/P0.17 active-low 입력으로 동작합니다. 위치 의미는
[NocFree & 공식 설명서](https://www.nocfree.com/pages/nocfree-and-manual)를
기준으로 확인했습니다.

- 위 **2.4G**: 아직 2.4G transport가 없어 USB/BLE 출력을 모두 안전하게 차단
- 가운데 **Wired**: USB HID 출력 선택
- 아래 **Bluetooth**: BLE HID 출력 선택

두 감지 핀이 동시에 low인 전환 순간에는 이전 출력을 유지하며, 20 ms 동안
같은 위치가 확인된 뒤에만 모드를 바꿉니다. Wired에서 왼쪽 USB가 없으면
HID 출력은 없지만 스위치가 전원을 끄지는 않으므로 MCU와 스캐너는 배터리를
소모합니다.

오른쪽 스위치는 펌웨어 입력이 아니라 배터리 전원선의 물리 ON/OFF입니다. 위가
OFF, 아래가 ON입니다. 오른쪽 USB가 연결되면 VBUS가 배터리 스위치를 우회하므로
OFF에서도 보드가 켜지는 것이 하드웨어상 정상입니다.

## 실기 검증

| 요구사항 | 최신 이미지의 증거 |
|---|---|
| USB 양쪽 입력/순서 | 이전 테스트는 통과했지만 2026-08-24 `jam -> ajm`을 재현해 순서 문제를 다시 열었음 |
| 84개 물리 키 | 문자 50개 + 비문자 33개 자동 sweep + Print Screen 수동 확인 |
| 한국어 Windows 특수키 | Right Alt의 `KanaMode` 매핑과 Print Screen OS 처리 확인 |
| BLE | 새 페어링 `freshcapture` → Wired `freshwired2` → 장치 삭제·재페어링 없는 BLE `reconnectcapture` 통과 |
| BLE 멀티 페어링 | 슬롯 1은 Windows 11, 슬롯 2는 Android에서 페어링·연결 확인 |
| 왼쪽 물리 모드 | Wired `wiredswitchok`, BLE `bleswitchok`/`bleagainok`, 2.4G 무출력 `24silentok` 통과 |
| 오른쪽 물리 전원 | 오른쪽 USB 분리 후 ON `jkluiop`, OFF 무입력, ON 복귀 `jkluiop` 통과 |
| Link | A→B, Shift+B hotkey, 재부팅 보존, 삭제/기본 A 복구 통과 |
| 양쪽 순정 원복 | Rust→순정 V2.3.0→serial DFU→같은 Rust→입력 통과 |
| 오른쪽 DFU 단축키 | `Fn+Delete`→UF2 확인→최신 Rust→전원 재인가 없이 `jkluiop` 통과 |
| 미디어 | mute/unmute, volume down/up 통과 |
| 배터리 | 순정 V2.3.0 환산식과 필터 복구 후 완충된 양쪽에서 `Fn+I` 3초 홀드로 `L 100 R 100` 출력; 방전 동작은 장기 실기 확인이 남음 |
| 상태 LED | `Fn+3` 홀드 후 손을 떼도 왼쪽 파란 LED가 계속 점멸하고, `Fn+1` 짧게 눌러 Windows bond 슬롯으로 복귀하면 소등됨을 확인. 빨간 저전압 점멸은 양쪽 모두 10% 초과라 실물 확인하지 못함 |
| interrupt 기반 idle 스캔 | 3초 idle 뒤 양쪽 모두 첫 키를 즉시 감지하고, 길게 누르기 반복과 release가 정상이며 좌우 혼합 입력 순서를 보존함. interrupt 유실 대비 250 ms 안전 스캔 유지 |
| 백라이트 동기화/자동 소등 | 왼쪽을 수동 OFF로 둔 채 오른쪽을 재부팅해 상태를 어긋내도 자동 수렴, 토글 10회 동기 유지, 30초 양쪽 자동 소등과 오른쪽 첫 키의 양쪽 wake 확인 |
| Split 재연결 진단 | 약 30cm에서 `-75/-74 dBm`, 첫 MTU 교환 실패와 다음 연결·보안·GATT 성공, HCI 해제 사유 `0x08`, 미연결 오른쪽 키를 기록. 양쪽 1200-baud 복구 뒤 입력 복귀도 확인 |
| System OFF와 split 복귀 | 10초 진단 이미지에서 USB 연결 중 System OFF 차단, 배터리 상태 왼쪽 central System OFF, sleep 전 양쪽 백라이트 소등, 왼쪽 USB 또는 왼쪽 키 홀드 wake, BLE 복귀와 오른쪽 split 입력을 확인. 배포 이미지는 같은 경로에서 timeout만 5분으로 변경. 짧은 wake 키 탭은 reset 부팅 중 소모될 수 있으므로 해당 문자도 입력하려면 재연결까지 왼쪽 키를 유지해야 함 |
| 새 DFU 단축키 | `Fn+5` 왼쪽 DFU, 짧은 `Fn+5` 무동작, `Fn+0` 3초 오른쪽 DFU와 최신 이미지 복귀 확인 |
| 물리 스위치 전용 출력 | `Fn+U`, `Fn+B`를 차례로 눌러 출력 전환 없이 `ub` 입력 확인 |

BLE 호스트 실기 검증은 **Windows 11과 Android에서만** 수행했습니다. macOS,
iOS, Linux 및 세 번째 호스트는 확인하지 않았습니다. `NocFree 1`/`2`/`3`은
선택된 페어링 슬롯을 나타내는 광고 이름이며 서로 다른 Bluetooth identity가
아닙니다. 따라서 같은 Windows 11 PC에서는 기존 장치 이름이 선택 슬롯에 따라
바뀌어 보일 수 있고, 빈 슬롯의 새 페어링은 Android 같은 다른 호스트에서
확인해야 합니다.

최신 이미지 검증에서는 슬롯 1의 기존 bond와 Windows 장치를 각각 한 번
삭제하고 새로 페어링했습니다. 이후 같은 bond로 Bluetooth→Wired→Bluetooth를
전환했으며 Windows 장치 삭제나 재페어링 없이 HID 입력이 즉시 복구됐습니다.
펌웨어는 Windows의 CCCD write를 받는 즉시 시스템 속성을 RAM/flash에 저장하고,
재연결 보안이 완료되면 알림 전송 전에 복원합니다.

## 코드 읽는 순서

1. `src/keymap.rs`, `src/link_keymap.rs`
2. `src/scanner.rs`, `src/pca9555.rs`, `src/hardware_scanner.rs`
3. `src/report.rs`
4. `src/bin/right.rs`, `src/split_ble.rs`
5. `src/bin/central.rs`, `src/link_usb.rs`, `src/link_protocol.rs`
6. `src/platform.rs`, `src/bond_store.rs`, `src/bond_record.rs`

`vendor/nrf-softdevice`와 macro 폴더는 upstream 커밋
`b0ac850c0a5a05b8a5aef4f752b48115755b8542`의 로컬 사본입니다. 1M PHY와
보안 CCCD 변경 이유는 각 `README.nocfree.md`에 기록돼 있습니다.
