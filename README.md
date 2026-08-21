# NocFree-and-rust

NocFree & ANSI 키보드의 ZMK 동작을 nRF52833용 `no_std` Rust 펌웨어로
옮긴 프로젝트입니다. 기준은
[`jhkim0218/NocFree-and-zmk`](https://github.com/jhkim0218/NocFree-and-zmk.git)
커밋 `e5e2f470795e92609f7ee6e810470fa6976557d1`입니다.

2026-08-21 현재 양쪽 최신 Rust 이미지가 실제 장치에 설치돼 있고 USB, BLE,
84개 물리 키, 물리 모드/전원 스위치, 단축키, NocFree Link 키 변경, 양쪽 DFU와
역할별 순정 원복을 통과했습니다. 다음 작업자는 [HANDOFF.md](HANDOFF.md)를 먼저
읽으십시오.

## 역할

- **왼쪽 (`central`)**: 왼쪽 37키, 오른쪽 split 수신, 전체 84키 키맵,
  USB/BLE HID, 3개 BLE 프로필, NocFree Link를 담당합니다.
- **오른쪽 (`right`)**: 오른쪽 47키를 스캔해 암호화 BLE split으로 왼쪽에
  전달합니다. 오른쪽 USB는 HID가 아니라 독립 CDC 복구용입니다.

양쪽은 PCA9555 세 개(`0x20`, `0x22`, `0x24`)를 SDA P0.11/SCL P1.09에서
100 kHz로 읽습니다. active-low 입력, 5 ms debounce, 1M BLE PHY를 사용합니다.
부트로더나 다른 펌웨어가 I2C 전송 중 MCU를 재시작해도 외부 확장칩이 걸린 채
남지 않도록 TWIM 초기화 전에 최대 9회의 bus-clear clock과 STOP을 보냅니다.

## 빌드

Windows PowerShell에서 다음을 실행합니다. Rust, Python 3,
`adafruit-nrfutil` 0.5.3.post16이 필요합니다.

```powershell
$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
Set-Location -LiteralPath 'D:\study\nocfree\NocFree-and-rust'
& pwsh -NoProfile -ExecutionPolicy Bypass -File '.\tools\build-release.ps1'
$buildExit = $LASTEXITCODE
if ($buildExit -ne 0) { throw ('build failed with exit code {0}' -f $buildExit) }
```

스크립트는 포맷, Windows 호스트 테스트, host/ARM Clippy, 양쪽 release 빌드,
BIN/UF2/serial-DFU ZIP 생성, 주소/family/vector/round-trip 및 ZIP 내부 BIN
일치를 모두 검사합니다. 마지막 결과는 Rust 52개, Python/계약/아티팩트 16개
테스트 통과입니다.

키보드에서 실행되는 코드는 전부 Rust `no_std`입니다. `tools/*.py`와 Python
패키지인 `adafruit-nrfutil`은 PC에서 산출물을 만들고 검사할 때만 사용되며
키보드에는 들어가지 않습니다.

## 최신 산출물

| 파일 | 크기(bytes) | SHA-256 |
|---|---:|---|
| `firmware/NocFree_Rust_Left.bin` | 65,900 | `2159C4435C7FCFCAD35E4A8526D8A2E4789F38D190DE75EACB4F151A94361F6D` |
| `firmware/NocFree_Rust_Left.uf2` | 132,096 | `8EBEEF9396139E54C69ED5335E39876D8075F434E30FF047B2A95A6FA46F8421` |
| `firmware/NocFree_Rust_Left_DFU.zip` | 66,776 | `F412EC2A668CAAFB425CF8D753897339F2F86A8389944AA770A806733DF0CEB0` |
| `firmware/NocFree_Rust_Right.bin` | 39,076 | `760F55A6CBBC8DCD019C2C2B8F83C0B5083515DA3461A0A250344CC9DC188054` |
| `firmware/NocFree_Rust_Right.uf2` | 78,336 | `A6BFA71F382E5F853D7176D60C694346CD348B31FD6B5E13F8F5CEAEFB89E47A` |
| `firmware/NocFree_Rust_Right_DFU.zip` | 39,958 | `F7D5CD9007B8E563E92F88C1B57960B5E621C254987350BA8DE56026149F9BDB` |

UF2는 앱 시작 `0x27000`부터 왼쪽 `0x371ff`, 오른쪽 `0x308ff`까지만
기록합니다. SoftDevice, 저장소, 공장 파일시스템과 UF2 부트로더는 보존합니다.

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
| 완료 | USB/BLE HID | 왼쪽 USB HID, BLE HID, USB↔BLE 전환과 같은 이미지에서의 BLE 자동 재연결 |
| 완료 | 양쪽 split | 오른쪽 입력과 배터리 값을 암호화 BLE split으로 왼쪽에 전달하고, 링크 단절 뒤 자동 복구 |
| 완료 | BLE 프로필 3개 | 프로필별 bond와 선택 상태 영구 저장, `NocFree 1`/`2`/`3` 이름으로 광고 |
| 완료 | 백라이트 | 양쪽 흰색 백라이트 동기 제어, 토글과 20% 단위 밝기 조절 |
| 완료 | 물리 스위치 | 왼쪽 Wired/Bluetooth 선택과 2.4G 위치의 안전한 무출력, 오른쪽 물리 전원 스위치 동작 |
| 완료 | NocFree Link 키맵 | 8×84 키, hotkey 16개, 실행·삭제·기본값 복구와 CRC 포함 flash 저장 |
| 완료 | 복구 | 양쪽 독립 CDC 1200-baud DFU, Fn DFU 단축키, Rust↔순정 V2.3.0 왕복 |
| 부분 | 배터리 | 양쪽 ADC 측정과 `Fn+I` 출력은 동작. 현재 3.45–4.20 V 선형 환산이라 완충/방전 실측에 따른 퍼센트 곡선 보정이 남음 |
| 부분 | NocFree Link 호환 | 키맵과 hotkey는 동작. Link 배터리 표시는 미지원이며 quick text는 빈 조회만 지원하고 저장·삭제·실행은 미지원 |
| 부분 | 순정 전원 관리 | 배터리 divider는 측정할 때만 켬. 순정과 같은 idle 전류·deep sleep·충전/저전압 임계값은 아직 계측 검증하지 않음 |
| 미구현 | 2.4 GHz 통신 | 공장 USB receiver, 왼쪽 외부 nRF24L01, 오른쪽/별도 numpad의 ESB 연결은 사용하지 않으며 현재 split은 BLE 전용 |
| 미구현 | 순정 상태 표시등 | 빨간 충전/저전압 LED와 파란 상태 LED의 순정 동작 재현 |
| 미구현 | 기타 설정 경로 | ZMK Studio와 공장 firmware updater의 완전한 호환은 제공하지 않음 |

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
| `Fn+1` / `Fn+2` / `Fn+3` | 짧게 | BLE 프로필 1 / 2 / 3 선택 |
| `Fn+1` / `Fn+2` / `Fn+3` | 1초 홀드 | 해당 프로필의 bond를 삭제하고 페어링 시작 |
| `Fn+5` | 3초 홀드 | **왼쪽** UF2 부트로더 진입. 짧게 누르면 무동작 |
| `Fn+0` | 3초 홀드 | **오른쪽** UF2 부트로더 진입. 짧게 누르면 무동작 |
| `Fn+Tab` | 즉시 | 양쪽 백라이트 켜기 / 끄기 |
| `Fn+U` | 즉시 | USB 출력 강제 선택 |
| `Fn+I` | 3초 홀드 | `L {왼쪽 퍼센트} R {오른쪽 퍼센트}`를 현재 출력으로 입력. 짧게 누르면 무동작 |
| `Fn+Delete` | 3초 홀드 | 호환용 **오른쪽** DFU 별칭. 새 조작은 `Fn+0` 권장 |
| `Fn+B` | 즉시 | BLE 출력 강제 선택 |
| `Fn+M` | 짧게 / 1초 홀드 | 짧게 M 입력 / 홀드하면 macOS 모드 영구 저장 |
| `Fn+N` | 짧게 / 1초 홀드 | 짧게 N 입력 / 홀드하면 Windows 모드 영구 저장 |

표에 없는 `Fn+키`는 별도 기능이 없는 transparent 동작이라 원래 키가
입력됩니다. 특히 기준 ZMK의 `Fn+6`(활성 프로필 삭제)과 `Fn+7`(USB/BLE
toggle)은 Rust에서 같은 위치에 구현하지 않았습니다. 프로필 삭제·페어링은
대상 `Fn+1/2/3`을 1초 홀드하고, 출력 선택은 물리 스위치 또는 `Fn+U`/`Fn+B`를
사용하십시오. DFU는 오입력 방지를 위해 기준 ZMK의 즉시 실행 대신 3초 홀드로
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
같은 위치가 확인된 뒤에만 모드를 바꿉니다. `Fn+U`/`Fn+B` 강제 선택은 다음
물리 위치 변경 전까지 계속 사용할 수 있습니다. Wired에서 왼쪽 USB가 없으면
HID 출력은 없지만 스위치가 전원을 끄지는 않으므로 MCU와 스캐너는 배터리를
소모합니다.

오른쪽 스위치는 펌웨어 입력이 아니라 배터리 전원선의 물리 ON/OFF입니다. 위가
OFF, 아래가 ON입니다. 오른쪽 USB가 연결되면 VBUS가 배터리 스위치를 우회하므로
OFF에서도 보드가 켜지는 것이 하드웨어상 정상입니다.

## 실기 검증

| 요구사항 | 최신 이미지의 증거 |
|---|---|
| USB 양쪽 입력/순서 | `asdfjkljamjamjam`, 전체 문자 배열, 교차 `jam` 반복 통과 |
| 84개 물리 키 | 문자 50개 + 비문자 33개 자동 sweep + Print Screen 수동 확인 |
| 한국어 Windows 특수키 | Right Alt의 `KanaMode` 매핑과 Print Screen OS 처리 확인 |
| BLE | `blejamjamjam`, USB↔BLE 전환, 재부팅 후 자동 재연결 통과 |
| 왼쪽 물리 모드 | Wired `wiredswitchok`, BLE `bleswitchok`/`bleagainok`, 2.4G 무출력 `24silentok` 통과 |
| 오른쪽 물리 전원 | 오른쪽 USB 분리 후 ON `jkluiop`, OFF 무입력, ON 복귀 `jkluiop` 통과 |
| Link | A→B, Shift+B hotkey, 재부팅 보존, 삭제/기본 A 복구 통과 |
| 양쪽 순정 원복 | Rust→순정 V2.3.0→serial DFU→같은 Rust→입력 통과 |
| 오른쪽 DFU 단축키 | `Fn+Delete`→UF2 확인→최신 Rust→전원 재인가 없이 `jkluiop` 통과 |
| 미디어 | mute/unmute, volume down/up 통과 |
| 배터리 | `Fn+I` 3초 홀드로 `L 24 R 11` 출력; 완충 기준 퍼센트 보정은 아직 필요 |
| 새 DFU 단축키 | `Fn+5` 왼쪽 DFU, 짧은 `Fn+5` 무동작, `Fn+0` 3초 오른쪽 DFU와 최신 이미지 복귀 확인 |

새 UF2 설치 직후 첫 BLE 전환에서는 Windows의 이전 세션 때문에 장치를 한 차례
삭제/재등록해야 했습니다. 같은 이미지의 다음 Wired→BLE 전환은 Windows 조작
없이 `bleagainok`로 자동 재연결됐고, 기존 `Fn+Esc` 재부팅 시험도 자동 재연결을
통과했습니다.

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
