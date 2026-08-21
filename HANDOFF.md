# NocFree Rust 펌웨어 작업 인계서

마지막 갱신: 2026-08-21 (Asia/Seoul)

이 문서는 다른 AI나 작업자가 대화 기록 없이 현재 작업을 이어가기 위한 기준
문서입니다. 이전에 통과한 펌웨어와 현재 작업 중인 Link 호환 펌웨어의 검증
상태를 반드시 구분하십시오.

## 1. 현재 목표

`NocFree-and-zmk`의 NocFree & ANSI 동작을 Rust 펌웨어로 옮기고 다음을 실제
장치에서 확인하는 것이 완료 조건입니다.

1. 기존 ZMK 단축키와 양쪽 84키 물리 배열
2. USB와 Bluetooth에서 왼쪽·오른쪽 입력 및 교차 입력 순서
3. `link.nocfree.com` 또는 ZMK Studio를 통한 키 변경과 재부팅 후 보존
4. 양쪽 DFU 진입, 역할별 순정 펌웨어 복귀, 같은 Rust 펌웨어 재설치

현재 구현은 두 키 변경 경로 중 **NocFree Link 호환**을 선택했습니다. ZMK
Studio 프로토콜은 구현하지 않았습니다.

## 2. 저장소

### Rust 구현

- 경로: `D:\study\nocfree\NocFree-and-rust`
- Git branch: `main`
- remote: 없음
- 마지막 commit: `7e361e5 feat: add NocFree Link keymap protocol foundation`
- 그 이전 commit: `8523f20 feat: add NocFree Rust firmware`
- 최신 Link/복구 안전장치 변경과 산출물은 아직 commit하지 않은 working tree에
  있습니다. `git status`를 먼저 확인하고 변경을 버리지 마십시오.

### ZMK 기준

- 경로: `D:\study\nocfree\NocFree-and-zmk`
- remote: `https://github.com/jhkim0218/NocFree-and-zmk.git`
- branch: `main`
- 기준 commit: `e5e2f470795e92609f7ee6e810470fa6976557d1`
- 마지막 확인 상태: `main...origin/main`, clean

## 3. 반쪽 역할

| 역할 | 책임 | 정상 Rust USB 식별자 |
|---|---|---|
| 왼쪽 / `central` | 왼쪽 37키, 전체 84키 키맵, 오른쪽 split 수신, USB HID, BLE HID, Link, BLE 프로필 | 최신 목표 `USB\VID_2886&PID_8029\RUST-LEFT` |
| 오른쪽 / `right` | 오른쪽 47키 스캔, 암호화 BLE split 송신, 독립 CDC 복구 | `USB\VID_1D50&PID_615E\RUST-RIGHT` |

오른쪽은 정상 상태에서도 호스트 입력용 HID를 제공하지 않습니다. 오른쪽 입력은
BLE split으로 왼쪽에 전달되고 왼쪽 HID를 통해 PC에 입력됩니다. COM 번호로
반쪽을 추측하지 말고 반드시 USB 부모 Instance ID를 확인하십시오.

## 4. 현재 실제 장치 상태와 사고 원인

2026-08-21 마지막 PnP 확인 결과:

- 왼쪽: USB/CDC/UF2 모두 열거되지 않음
- 오른쪽: `USB\VID_1D50&PID_615E\RUST-RIGHT`, COM18, 상태 OK
- UF2 볼륨: 없음
- Windows의 BLE 캐시는 왼쪽이 현재 실행 중이라는 증거가 아님
- J-Link, CMSIS-DAP, ST-Link 등 SWD probe는 연결되어 있지 않았음

왼쪽에는 Link vendor interface를 처음 추가한 중간 UF2가 설치됐습니다. 기존
USB 구성은 keyboard HID + consumer HID + CDC 2개 interface로 이미 4개를
사용했고 Link가 다섯 번째였습니다. `embassy-usb` 기본 interface 한도 4를
넘어 USB builder에서 panic했으며 당시 이미지는 `panic-halt`를 사용했습니다.
따라서 scanner, CDC 1200-baud recovery, split, BLE host task가 시작되기 전에
멈춥니다.

이 문제의 소스 수정은 끝났습니다.

- `.cargo/config.toml`: `EMBASSY_USB_MAX_INTERFACE_COUNT = "5"`
- `panic-halt` 제거
- `src/platform.rs`: 미처리 panic 시 SoftDevice SVC로 GPREGRET `0x57`을
  설정하고 system reset하여 UF2로 들어가는 fail-safe panic handler
- 최신 양쪽 이미지는 자동 빌드와 테스트를 통과함

하지만 수정 이미지를 왼쪽에 넣으려면 현재 왼쪽을 한 번 물리적으로 부트로더에
진입시켜야 합니다.

### 현재 왼쪽 복구에 관한 금지 사항

- NocFree &에는 외부 reset 버튼이 없습니다.
- 현재 설치된 Rust 키맵에는 공장 펌웨어의 `Option+3` 복구 단축키가 없습니다.
- panic이 scanner/CDC보다 먼저라 키 조합과 1200-baud touch를 사용할 수
  없습니다.
- 전원 재연결은 Adafruit bootloader가 검사하는 reset-pin double tap과 같다고
  입증되지 않았습니다.
- NocFree가 지시하지 않은 PCB 패드를 임의로 쇼트하면 안 됩니다.

안전한 다음 단계는 NocFree가 지정한 reset/recovery test point를 정확히
확인하거나 J-Link/nRF52 DK 같은 SWD probe를 준비하는 것입니다. 사용자가
하드웨어를 조작해야 할 때는 반드시 12절의 Windows 알림을 띄우고 작업을
멈춘 뒤 사용자의 다음 메시지를 기다립니다.

## 5. 최신 구현 내용

### 입력과 split

- PCA9555 세 개: `0x20`, `0x22`, `0x24`
- I2C: SDA P0.11, SCL P1.09, 100 kHz
- active-low, press/release 5 ms debounce
- 왼쪽 37키 + 오른쪽 47키 = 84키
- 오른쪽 split은 별도 Just Works bond로 암호화
- 왼쪽/오른쪽 변화는 하나의 FIFO로 합쳐 교차 입력 순서를 보존
- split interval 7.5 ms, latency 30, supervision timeout 4초
- 모든 BLE PHY는 vendor patch로 1M만 허용
- USB/BLE 경로 전환 때 이전 경로에 release를 보내 stuck key 방지

### Link 동적 키맵

- 왼쪽 USB VID/PID: `0x2886:0x8029`
- product: `NocFree & ANSI`
- serial: `RUST-LEFT`
- vendor class `0xFF` bulk IN/OUT + MS OS 2.0 WINUSB descriptor
- Link frame: `[FF FE op len payload FE FF]`
- 8 layers × 84 physical keys
- Link UI matrix: 6 rows × 21 columns; 비어 있는 좌표는 transparent
- 기본값은 기준 ZMK keymap과 Fn layer를 보존
- SET/GET key, GET/SET layer row, clear layer/all, system/version/battery
- 16개 hotkey 슬롯의 SET/GET/CLEAR/DELETE, flash 저장 및 HID chord 실행
- 동적 binding은 `ReportEngine::apply_snapshot_with`가 실제 입력마다 조회
- page 7 `0x6c000`에 CRC가 포함된 version-2 record로 저장

Link의 quick text 16개는 GET 요청에 빈 슬롯으로 즉시 응답하여 웹앱 timeout을
막지만 SET/CLEAR/DELETE와 문자열 실행은 아직 구현하지 않았습니다. 목표의
일반 키 변경과 hotkey는 구현됐지만 실제 웹앱 연결·변경·재부팅 보존은 왼쪽
복구 후 실기 검증해야 합니다.

## 6. 기본 키 바인딩

- `Fn+1` .. `Fn+5`: BLE 호스트 프로필 1..5 선택
- `Fn+0`: 현재 BLE 프로필 삭제
- `Fn+U`: USB 출력 강제 선택
- `Fn+B`: BLE 출력 강제 선택
- `Fn+F10/F11/F12`: 음소거/볼륨 내림/볼륨 올림
- `Fn+Esc`: 왼쪽 애플리케이션 재시작; DFU가 아님
- `Fn+Delete`: 정상 split을 통해 오른쪽 UF2 부트로더 진입
- 양쪽 독립 복구: 해당 반쪽 CDC의 1200-baud touch

현재 중간 이미지에서 멈춘 왼쪽에는 위 키/CDC 경로가 동작하지 않습니다.

## 7. flash 메모리

- `0x00000..0x26fff`: MBR + S140 7.3.0, 보존
- `0x27000..0x64fff`: Rust application
- `0x65000..0x69fff`: BLE host profiles
- `0x6a000..0x6afff`: 선택 profile/settings
- `0x6b000..0x6bfff`: split bond
- `0x6c000..0x6cfff`: Link keymap/hotkeys
- `0x6d000..0x73fff`: factory filesystem, 보존
- `0x74000..0x7ffff`: UF2 bootloader/metadata, 보존

`memory.x`의 앱 FLASH는 `0x27000`, 길이 `0x3e000`; RAM은 `0x20010000`,
길이 `0x10000`입니다.

## 8. 최신 산출물

`tools/build-release.ps1`을 통과한 현재 파일입니다.

| 파일 | 크기(bytes) | SHA-256 |
|---|---:|---|
| `firmware/NocFree_Rust_Left.bin` | 59,228 | `301B8AA4B3A3F20A8C97E0076EB4921969EB4DF5FD87BA167CD893CAE9C78CF4` |
| `firmware/NocFree_Rust_Left.uf2` | 118,784 | `368CDD50F457680561CD0D0360F092A756675472F5A2297A9531B7DF734B7707` |
| `firmware/NocFree_Rust_Left_DFU.zip` | 60,104 | `4A70FE90CD0B00069ACB0745A308A4C2E7B10057B8CC7A7C0586C9D3DC28F0DE` |
| `firmware/NocFree_Rust_Right.bin` | 36,172 | `B60DFB644BF3920FC4DE1285FD8DA9260E2B8CB69F3DD0F989A5AC163A38A329` |
| `firmware/NocFree_Rust_Right.uf2` | 72,704 | `A75EEAAD39B0B10ACF4B38B61EEBA361B44FE9E39A13EB8DB29423547F472E3C` |
| `firmware/NocFree_Rust_Right_DFU.zip` | 37,054 | `4D19A3DED7BBDB47CD448ADECCF50265D81DC09748492DDAFBB4C57EA3DA1535` |

DFU ZIP은 `adafruit-nrfutil 0.5.3.post16`, DFU version 0.5, device type 82,
SoftDevice requirement `0xFFFE`로 생성합니다. 자동 테스트가 각 ZIP 내부 BIN과
같은 역할의 최신 `.bin`이 바이트 단위로 일치하는지 확인합니다.

키보드에서 실행되는 코드는 전부 Rust `no_std`입니다. `tools/*.py`와 Python으로
작성된 `adafruit-nrfutil`은 PC에서 BIN을 UF2/DFU ZIP으로 포장하고 검사할 때만
사용되며 Python 코드는 키보드에 플래시되지 않습니다.

역할별 순정 파일:

| 역할 | 파일 | SHA-256 |
|---|---|---|
| 왼쪽 | `D:\study\nocfree\NocFree_and_V2.3.0_Left_ANSI.uf2` | `A3FF612B94E9CE0C12BEFD9FF19ECDE5D6E4DB964C0345E432B71ED7A2C5BC91` |
| 오른쪽 | `D:\study\nocfree\NocFree_and_V2.3.0_Right_ANSI.uf2` | `E1F851906B3E35117B8A8AAC09E5C8273D75921F64D1D3B1496E60A53D3E1C66` |

두 순정 UF2는 nRF52833 family `0x621e937a`, 앱 시작 `0x27000`으로 검증됐습니다.
다른 역할의 파일을 복사하면 안 됩니다.

## 9. 자동 검증

PowerShell:

```powershell
$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
Set-Location -LiteralPath 'D:\study\nocfree\NocFree-and-rust'
& pwsh -NoProfile -File '.\tools\build-release.ps1'
$buildExit = $LASTEXITCODE
if ($buildExit -ne 0) { throw ('build failed with exit code {0}' -f $buildExit) }
```

마지막 전체 결과:

- Rust host tests: 43/43
- Python tests: 14/14
- Rust fmt: 통과
- host lib + central/right ARM clippy `-D warnings`: 통과
- central/right release build: 통과
- ELF→BIN→UF2, 주소/family/vector/round-trip: 통과
- DFU ZIP 생성 및 내부 BIN/manifest 검증: 통과

## 10. 이전 체크포인트의 실제 하드웨어 증거

다음은 commit `7e361e5` 이전/당시의 정상 Rust 이미지에서 실제 통과한 결과이며,
최신 Link 이미지의 통과 증거로 사용하면 안 됩니다.

- 양쪽 UF2 bootloader: NocFree & / S140 7.3.0
- USB 전체 84키: 문자 50개 + 비문자 34개, 양쪽 Fn
- 오른쪽→왼쪽 USB 입력: `yuiopY`
- 교차 순서 수정 후: `jamjamjamjam`, `jajaj...`
- BLE 교차 입력: `blejamjamjam`
- USB→BLE→USB: `finalusbjamfinalblejamfinalusbback`
- 미디어: mute/unmute, volume down/up
- 왼쪽과 오른쪽 각각 Rust→DFU→역할별 순정→serial DFU→Rust 복귀
- `Fn+Esc` 후 `resetleftok`
- `Fn+Delete` 오른쪽 DFU 후 재설치, `rightbackjam`

BLE 전체 84키 sweep, profile 2..5 각각의 pairing/삭제, 장시간 완전 전원 cycle은
당시에도 별도로 실시하지 않았습니다.

## 11. 남은 순서

1. NocFree가 지정한 왼쪽 recovery/reset point 또는 SWD probe 확보
2. Windows 알림 후 왼쪽을 UF2로 한 번 진입
3. `INFO_UF2.TXT`에서 NocFree & / S140 7.3.0 확인
4. 최신 **Left** UF2 설치
5. 목표 USB ID, keyboard/consumer/CDC/Link interface 확인
6. 왼쪽 1200-baud DFU 왕복과 panic fail-safe 경로 확인
7. Chrome/Edge `link.nocfree.com`에서 연결, 단일 키 변경, hotkey 변경,
   재부팅 후 보존, 기본값 복구 확인
8. 왼쪽 USB/BLE 키와 오른쪽 split 교차 입력 확인
9. 왼쪽이 완전히 정상일 때만 최신 **Right** UF2 설치
10. 오른쪽 입력, `Fn+Delete`, 독립 1200-baud DFU, 역할별 순정 원복 확인
11. 문서의 실기 상태 갱신 후 commit

사용자가 물리 테스트를 해야 하는 지점마다 한 단계만 알리고 멈춰야 합니다.
몇 초 동안 키 입력을 감시하거나 자동으로 다음 단계로 넘어가면 안 됩니다.

## 12. Windows 사용자 알림

사용자 조작이 필요하면 다음 helper를 직접 실행합니다.

```powershell
$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
& pwsh -NoProfile -File 'D:\study\nocfree\NocFree-and-rust\tools\notify-user.ps1' 'Codex에서 키보드 확인이 필요합니다.'
$notifyExit = $LASTEXITCODE
if ($notifyExit -ne 0) { throw ('notify failed with exit code {0}' -f $notifyExit) }
```

이 helper는 system sound와 640×220 TopMost Windows Forms 창을 띄웁니다.
사용자가 `새 알림 보임`으로 확인했습니다. 알림을 보낸 뒤에는 사용자의 다음
입력을 기다리고 추가 polling을 하지 않습니다.

## 13. PowerShell 규칙

모든 PowerShell 실행은 다음으로 시작합니다.

```powershell
$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
```

- `$LASTEXITCODE`는 native process 직후에만 저장·확인
- native process 결과를 pipeline으로 넘기기 전에 exit code 확인
- `foreach` statement를 pipeline에 직접 연결하지 않기
- `$var:` 대신 `'{0}: {1}' -f $name, $value`
- COM child ID만 보고 반쪽 판별하지 말고 USB parent property 확인
- 왼쪽/오른쪽 역할과 UF2 이름을 복사 직전에 다시 대조

## 14. 코드 읽기 순서

1. `src/keymap.rs`, `src/link_keymap.rs`: 기본 84키, 동적 레이어, hotkey
2. `src/link_protocol.rs`, `src/link_usb.rs`: Link wire protocol와 USB class
3. `src/report.rs`: 동적 action을 HID/command로 변환
4. `src/scanner.rs`, `src/pca9555.rs`, `src/hardware_scanner.rs`
5. `src/bin/central.rs`: 왼쪽 USB/BLE/split/Link 통합
6. `src/bin/right.rs`, `src/split_protocol.rs`, `src/split_ble.rs`
7. `src/platform.rs`, `src/bond_store.rs`, `src/bond_record.rs`
8. `tools/build-release.ps1`, `tools/test_*.py`, `RECOVERY.md`

`vendor/nrf-softdevice`와 macro 폴더는 upstream commit
`b0ac850c0a5a05b8a5aef4f752b48115755b8542`의 로컬 사본입니다. 1M PHY와
보안 CCCD 수정 이유는 각 `README.nocfree.md`에 있습니다.

## 15. 현재 완료 판정

**미완료입니다.** 최신 Link 이미지의 소스·산출물 자동 검증은 통과했지만,
왼쪽이 중간 이미지에서 soft-brick되어 최신 이미지를 설치하지 못했습니다.
따라서 최신 이미지의 물리 84키, USB/BLE, Link 웹앱 키 변경, 양쪽 DFU/순정
원복을 아직 완료로 표시할 수 없습니다.
