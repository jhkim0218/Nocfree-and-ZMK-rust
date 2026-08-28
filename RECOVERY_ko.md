# DFU 및 역할별 순정 원복

[English](RECOVERY.md) · [한국어](RECOVERY_ko.md) · [日本語](RECOVERY_ja.md)

NocFree &에는 외부 리셋 버튼이 없습니다. 확인되지 않은 PCB 패드를 쇼트하거나
전원 재연결을 reset pin double-tap으로 가정하면 안 됩니다. Rust 펌웨어는
역할별 Fn 단축키와 별개로 양쪽과 Experimental 동글 각각의 1200-baud CDC 복구 경로를 유지합니다.

## 보호 영역

| 범위 | 용도 | 정책 |
|---|---|---|
| `0x00000..0x26fff` | MBR + S140 7.3.0 | 보존 |
| `0x27000..0x64fff` | Rust 애플리케이션 | 기록 가능 |
| `0x65000..0x67fff` | BLE host profiles | 저장 데이터 |
| `0x68000..0x68fff` | 예약 | 보존 |
| `0x69000..0x69fff` | 전용 동글 bond | 저장 데이터 |
| `0x6a000..0x6afff` | 선택 profile/settings | 저장 데이터 |
| `0x6b000..0x6bfff` | split bond | 저장 데이터 |
| `0x6c000..0x6cfff` | Link keymap/hotkeys | 저장 데이터 |
| `0x6d000..0x73fff` | 공장 파일시스템 | 보존 |
| `0x74000..0x7ffff` | Adafruit UF2 bootloader/metadata | 보존 |

빌드와 UF2 왕복 검사는 앱이 `0x65000`을 넘으면 실패합니다. Left, Right, Dongle을
각 역할과 배열에 정확히 맞춰 사용하십시오.

## 진입 경로

- 왼쪽 Rust CDC 1200 baud touch: 독립적인 왼쪽 UF2 진입
- 오른쪽 Rust CDC 1200 baud touch: 독립적인 오른쪽 UF2 진입
- 동글 Rust CDC 1200 baud touch: 독립적인 동글 UF2 진입
- 동글 Rust CDC 2400 baud touch: 동글 bond 삭제 후 재시작
- `Fn+5` 3초 홀드: 왼쪽 UF2 진입
- `Fn+0` 3초 홀드: split이 정상일 때 오른쪽 UF2 진입
- `Fn+Esc`: 왼쪽 앱 재시작이며 DFU가 아님
- 처리되지 않은 panic: GPREGRET `0x57` 기록 후 UF2 진입

왼쪽 1200-baud 복구 때는 물리 스위치를 가운데 **Wired**에 둡니다. 포트가 너무
빨리 사라져 열기/닫기에서 없는 장치 오류가 나도 이미 reset된 경우가 있으므로,
반복하기 전에 UF2 볼륨과 bootloader CDC를 확인합니다.

Windows COM 번호는 바뀌므로 `DEVPKEY_Device_Parent`로 역할을 확인합니다.
Rust는 `RUST-LEFT`/`RUST-RIGHT`/`RUST-DONGLE`이며 동글 parent는
`USB\VID_239A&PID_80D8\RUST-DONGLE`입니다. UF2 CDC는 `VID_239A&PID_0029`, 순정 serial DFU는
`VID_239A&PID_002A`이며 칩 serial은 왼쪽 `52CF50988BD1E6EE`, 오른쪽
`D82A03513BB02626`입니다. macOS는 `stty -f /dev/cu.usbmodemXXXX 1200`, Linux는
`stty -F /dev/ttyACM0 1200` 형태를 사용하지만 실제 장치명과 역할을 먼저 확인해야 합니다.

## UF2 및 serial DFU

`INFO_UF2.TXT`에서 NocFree &, S140 7.3.0, NocFree & Board-ID를 확인합니다.
양쪽 volume 이름이 같으므로 사라진 CDC와 반대쪽 앱 존재 여부도 함께 확인합니다.
그 뒤 왼쪽은 `firmware/NocFree_And_Rust_ZMK_Based_ANSI_Left.uf2`, 오른쪽은
`firmware/NocFree_And_Rust_ZMK_Based_ANSI_Right.uf2`만 복사합니다.
Experimental ANSI 동글은
`firmware/experimental/NocFree_And_Rust_ZMK_Based_ANSI_Experimental_Dongle.uf2`를 사용합니다.
ANSI 동글 bootloader와 앱 복구는 실물에서 확인했습니다. ISO/JIS/KR은 같은 배열 이름의
Dongle UF2와 대응 키보드에서 검증해야 합니다.

serial DFU는 확인된 `PID_002A` 포트에서만 실행합니다. `adafruit-nrfutil`은 과거
`No data received on serial port` 실패를 출력하고도 exit 0을 반환한 적이 있으므로,
실패 문구 없음, `Device programmed.`, 목표 Rust parent 복귀를 모두 확인해야 합니다.

## 검증된 순정 왕복

2026-08-21 양쪽이 다음을 통과했습니다.

1. 대상 Rust parent/CDC와 반대쪽 존재 확인
2. 대상만 1200 touch
3. NocFree & / S140 UF2 bootloader 확인
4. 역할별 순정 V2.3.0 UF2 SHA-256 확인 후 복사
5. 역할별 순정 parent와 칩 serial 확인
6. 순정 CDC 1200 touch
7. 같은 칩의 `PID_002A` 확인
8. 역할별 Rust DFU ZIP 전송
9. `RUST-LEFT`/`RUST-RIGHT`와 실제 입력 확인

순정 왼쪽 SHA-256은 `A3FF612B94E9CE0C12BEFD9FF19ECDE5D6E4DB964C0345E432B71ED7A2C5BC91`,
오른쪽은 `E1F851906B3E35117B8A8AAC09E5C8273D75921F64D1D3B1496E60A53D3E1C66`입니다.

## 검증된 동글 serial-DFU 왕복

2026-08-27 순정 동글의 application-only 복구를 확인했습니다. 앱 기준선은
`NocFree_Dongle`, `VID_2886&PID_8029`, serial `E19D2CEA0B437049`, CDC `MI_00`,
HID `MI_02`입니다. 앱 CDC를 1200 baud로 열고 닫으면 UF2 디스크가 아니라
`VID_239A&PID_002A`, 제품명 `NocFree &`, `nRF Serial`인 serial 전용 bootloader로
진입합니다. 시험 중 COM6에서 COM14로 바뀌었으므로 COM 번호를 고정하면 안 됩니다.

사용한 공식 V2.3.1 application-only 패키지는 저장소 밖의
`D:\study\nocfree\official_v2_3_1\dongle.zip`이며 SHA-256은
`02C1EE2BB420E374E51AC6B0C0EE7A422796DFDFF0AC2707CA28096A564C0567`입니다.
manifest는 `softdevice_req=0x0123`이고 bootloader/SoftDevice/UICR를 포함하지 않습니다.
사용자 승인 뒤 COM14에 115200 baud로 전송해 `Device programmed.`를 확인했고,
원래 제품명·VID/PID·serial·CDC·HID가 모두 돌아왔습니다.

양쪽 키보드가 Rust firmware라 순정 ESB 입력은 확인할 수 없었습니다. 이번 결과는
동글 복구와 USB identity를 증명하며 순정 무선 프로토콜 호환을 증명하지 않습니다.

예상하지 않은 반쪽이 사라지거나 역할/배열/hash가 다르거나, 목표 parent가 돌아오지
않거나, 키가 고착되거나, 발열 또는 앱/bootloader 모두 미탐지 상태가 되면 즉시
중단하고 다른 반쪽을 건드리지 않습니다. 자동 빌드는 실물 flash를 허가하지 않으며
반드시 사용자의 명시적 승인 뒤 진행해야 합니다.
