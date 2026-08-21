# NocFree-and-rust

NocFree & ANSI 키보드의 ZMK 동작을 nRF52833용 `no_std` Rust 펌웨어로
옮긴 프로젝트입니다. 기준은
[`jhkim0218/NocFree-and-zmk`](https://github.com/jhkim0218/NocFree-and-zmk.git)
커밋 `e5e2f470795e92609f7ee6e810470fa6976557d1`입니다.

2026-08-21 현재 양쪽 최신 Rust 이미지가 실제 장치에 설치돼 있고 USB, BLE,
84개 물리 키, 단축키, NocFree Link 키 변경, 양쪽 DFU와 역할별 순정 원복을
통과했습니다. 다음 작업자는 [HANDOFF.md](HANDOFF.md)를 먼저 읽으십시오.

## 역할

- **왼쪽 (`central`)**: 왼쪽 37키, 오른쪽 split 수신, 전체 84키 키맵,
  USB/BLE HID, 5개 BLE 프로필, NocFree Link를 담당합니다.
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
일치를 모두 검사합니다. 마지막 결과는 Rust 43개, Python/계약/아티팩트 16개
테스트 통과입니다.

키보드에서 실행되는 코드는 전부 Rust `no_std`입니다. `tools/*.py`와 Python
패키지인 `adafruit-nrfutil`은 PC에서 산출물을 만들고 검사할 때만 사용되며
키보드에는 들어가지 않습니다.

## 최신 산출물

| 파일 | 크기(bytes) | SHA-256 |
|---|---:|---|
| `firmware/NocFree_Rust_Left.bin` | 60,012 | `DFF1A747F26F30FC125B2F5A0FC18E6B645AF1E26ED3C48E5D56D9C8B8CB3A07` |
| `firmware/NocFree_Rust_Left.uf2` | 120,320 | `C856810A82FCAD01306562BDBD76B1AC4F862679A23D256FEB8F51939E0B632B` |
| `firmware/NocFree_Rust_Left_DFU.zip` | 60,888 | `58FF28C4DF6FC36F9471EB801B68297CB6D5D59593D160754ADFED682FDC5922` |
| `firmware/NocFree_Rust_Right.bin` | 36,948 | `ACC150B6FB721197B5121EEA914E160C5F1B60A19C14F9C2F681AA15BAEBC104` |
| `firmware/NocFree_Rust_Right.uf2` | 74,240 | `5591FABA2CA68A148DF1010C92A8ECE0E42624AE30B81066932B98CE71697C3C` |
| `firmware/NocFree_Rust_Right_DFU.zip` | 37,830 | `99F9260CA6779137AF9C8F8E568744EC9E9FECA501D453053A532E56BCE99B9D` |

UF2는 앱 시작 `0x27000`부터 왼쪽 `0x35aff`, 오른쪽 `0x300ff`까지만
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
저장·실행은 구현하지 않았습니다.

## 기본 단축키

- `Fn+1` .. `Fn+5`: BLE 프로필 1..5 선택
- `Fn+0`: 현재 BLE 프로필 삭제
- `Fn+U`: USB 출력 강제 선택
- `Fn+B`: BLE 출력 강제 선택
- `Fn+F10/F11/F12`: 음소거/볼륨 내림/볼륨 올림
- `Fn+Esc`: 왼쪽 애플리케이션 재시작; DFU가 아님
- `Fn+Delete`: split을 통해 오른쪽 UF2 부트로더 진입

왼쪽 DFU와 split이 끊긴 오른쪽 DFU는 각 반쪽의 CDC 1200-baud touch를
사용합니다. NocFree &에는 외부 리셋 버튼이 없습니다. 확인되지 않은 PCB
패드를 쇼트하지 마십시오. 정확한 절차는 [RECOVERY.md](RECOVERY.md)에 있습니다.

## 실기 검증

| 요구사항 | 최신 이미지의 증거 |
|---|---|
| USB 양쪽 입력/순서 | `asdfjkljamjamjam`, 전체 문자 배열, 교차 `jam` 반복 통과 |
| 84개 물리 키 | 문자 50개 + 비문자 33개 자동 sweep + Print Screen 수동 확인 |
| 한국어 Windows 특수키 | Right Alt의 `KanaMode` 매핑과 Print Screen OS 처리 확인 |
| BLE | `blejamjamjam`, USB↔BLE 전환, 재부팅 후 자동 재연결 통과 |
| Link | A→B, Shift+B hotkey, 재부팅 보존, 삭제/기본 A 복구 통과 |
| 양쪽 순정 원복 | Rust→순정 V2.3.0→serial DFU→같은 Rust→입력 통과 |
| 오른쪽 DFU 단축키 | `Fn+Delete`→UF2 확인→최신 Rust→전원 재인가 없이 `jkluiop` 통과 |
| 미디어 | mute/unmute, volume down/up 통과 |

첫 BLE 전환 직후 Windows의 이전 세션 때문에 한 차례 연결 해제/재연결이
필요했지만, 이후 출력 전환과 `Fn+Esc` 재부팅에서는 Windows 조작 없이 자동
재연결됐습니다.

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
