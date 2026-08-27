# NocFree-and-rust

[English](README.md) · [日本語](README_ja.md)

> [!CAUTION]
> `develop` 브랜치는 개발 중인 작업을 포함합니다. 펌웨어 산출물은 자동 검사를
> 통과하지만 **실물 하드웨어에서 검증되지 않았습니다**.

nRF52833 기반 NocFree & 키보드를 위한 독립 `no_std` Rust 펌웨어입니다. 원본
[`NocFreeKB/NocFree-and-zmk`](https://github.com/NocFreeKB/NocFree-and-zmk)의
동작을 포팅했으며 NocFree 공식 펌웨어가 아닙니다.

> [!IMPORTANT]
> - 펌웨어 **기본값은 Windows 모드**입니다. macOS 모드는 `Fn+M`, Windows
>   복귀는 `Fn+N`을 각각 1초 동안 누릅니다.
> - 키보드 좌우와 배열에 맞는 파일만 플래시하십시오. 서로 다른 빌드나 배열의
>   왼쪽·오른쪽 파일을 섞으면 안 됩니다.
> - NocFree &에는 외부 리셋 버튼이 없습니다. 플래시 전에 [RECOVERY_ko.md](RECOVERY_ko.md)를
>   읽고 양쪽 DFU 및 순정 V2.3.0 복구 방법을 확인하십시오.
> - 공장 USB 동글/2.4 GHz 입력은 구현되지 않았습니다. 왼쪽 스위치의 2.4G
>   위치에서는 의도적으로 아무 입력도 출력하지 않습니다.

| 배열 | 현재 상태 | 실물 검증 |
|---|---|---|
| ANSI | 기본 빌드, 5 ms 입력 순서·linear 백라이트 후보 | 이전 3 ms 펌웨어는 검증했지만 현재 5 ms 빌드는 실기 미검증 |
| ISO | Experimental | 해당 실물에서 미검증 |
| JIS | Experimental | 해당 실물에서 미검증 |
| KR | Experimental | 해당 실물에서 미검증 |

## 먼저 확인할 내용

키보드는 역할이 고정된 좌우 분리형 구조입니다.

- **왼쪽(`central`)**은 37키와 오른쪽 입력을 모아 전체 키맵을 처리하고 USB 또는
  Bluetooth HID를 출력합니다.
- **오른쪽(`right`)**은 47키를 스캔해 암호화 BLE split으로 왼쪽에 보냅니다.
  오른쪽 USB는 복구·진단용이며 키보드 HID를 출력하지 않습니다.

왼쪽 물리 스위치는 출력 방식을 선택합니다.

| 위치 | 모드 | 동작 |
|---|---|---|
| 위 | 2.4G | 출력 없음. 공장 동글 전송은 미구현 |
| 가운데 | Wired | 왼쪽 USB 포트로 USB HID 출력 |
| 아래 | Bluetooth | 왼쪽에서 Bluetooth HID 출력 |

오른쪽 스위치는 배터리 전원을 물리적으로 제어합니다. 위가 OFF, 아래가 ON입니다.
USB 전원은 스위치를 우회하므로 USB 연결 중에는 OFF여도 오른쪽 보드가 켜집니다.
왼쪽 USB가 없는 Wired 모드는 HID 출력만 없을 뿐 키보드 전원을 끄지 않습니다.

처음 사용할 때는 다음 순서를 권장합니다.

1. [RECOVERY_ko.md](RECOVERY_ko.md)를 읽고 좌우 펌웨어를 구분합니다.
2. 같은 빌드의 UF2 두 개를 빌드하거나 내려받습니다.
3. 한쪽씩 플래시한 뒤 Wired USB에서 양쪽 입력을 먼저 확인합니다.
4. Bluetooth, 물리 스위치, DFU 단축키와 빠른 좌우 교차 입력을 확인합니다.

## 빌드와 펌웨어

Rust, Python 3, libclang이 포함된 C/C++ 도구와 패키징 도구를 설치합니다.

```text
rustup target add thumbv7em-none-eabihf
rustup component add llvm-tools-preview
python3 -m pip install adafruit-nrfutil==0.5.3.post16
```

Windows에서는 설치된 명령에 따라 `python3` 대신 `python`을 사용할 수 있습니다.
macOS에서 libclang을 찾지 못하면 Homebrew LLVM을 설치하고, Debian/Ubuntu에서는
`clang`과 `libclang-dev`를 설치합니다. 자동 탐색이 실패할 때만 `LIBCLANG_PATH`를
설정하십시오.

Windows, macOS 또는 Linux에서 ANSI를 빌드하고 검사합니다.

```text
python3 -B tools/build_release.py --layout ANSI
```

Experimental 배열은 `ISO`, `JIS`, `KR`을 지정하고, 네 배열 전체는 다음과 같이
빌드합니다.

```text
python3 -B tools/build_release.py --all-layouts
```

Windows PowerShell 래퍼도 계속 사용할 수 있습니다.

```powershell
& pwsh -NoProfile -ExecutionPolicy Bypass -File '.\tools\build-release.ps1' -Layout ANSI
```

빌더는 형식 검사, host/ARM 검사, 좌우 빌드, BIN/UF2/serial-DFU 생성과 주소·내용
검증을 수행합니다. ANSI 산출물은 `firmware`, Experimental 배열은
`firmware/experimental`에 저장됩니다.

저장소에 포함된 ANSI UF2:

- [왼쪽/central UF2](firmware/NocFree_And_Rust_ZMK_Based_ANSI_Left.uf2)
- [오른쪽/peripheral UF2](firmware/NocFree_And_Rust_ZMK_Based_ANSI_Right.uf2)

위 기본 파일은 눈에 보이는 linear 20% 백라이트 단계를 사용합니다. 체감 보정 비교용
펌웨어는 다음과 같이 빌드합니다.

```text
python3 -B tools/build_release.py --layout ANSI --backlight-curve perceptual
```

재현 가능한 B 펌웨어는 별도 파일로 저장됩니다.

- [Perceptual 왼쪽 UF2](firmware/experimental/NocFree_And_Rust_ZMK_Based_ANSI_Perceptual_Backlight_Experimental_Left.uf2)
- [Perceptual 오른쪽 UF2](firmware/experimental/NocFree_And_Rust_ZMK_Based_ANSI_Perceptual_Backlight_Experimental_Right.uf2)

두 A/B 펌웨어 모두 실물 미검증 상태입니다. 먼저 linear 좌우 한 쌍을 함께 검사한
다음 perceptual 좌우 한 쌍을 검사하십시오. 좌우에 서로 다른 곡선을 섞으면 안 됩니다.

키보드에서 실행되는 코드는 모두 Rust `no_std`입니다. Python과
`adafruit-nrfutil`은 PC에서 산출물을 만들 때만 사용하며 키보드에는 들어가지 않습니다.

## 미구현 및 미검증 항목

| 영역 | 남은 작업 |
|---|---|
| 현재 ANSI 후보 | 5 ms 입력 순서, 최신 공용 PCA9555 스캔과 linear/perceptual 백라이트 두 쌍을 플래시해 회귀 검증해야 함 |
| ISO/JIS/KR | 소프트웨어 빌드는 통과하지만 각 배열의 실물 키보드 검증이 필요함 |
| Split 신뢰성 | 장시간 시계 drift, 재연결 직후 입력, 실제 BLE jitter, 책상 거리 복구, +8 dBm 거리·전류 비교 |
| 배터리 | 완전 방전 주기, DMM 비교, 동작/idle/System OFF 전류와 실제 사용 시간 측정 |
| 상태 LED | 방전된 장치에서 빨간 저전압 표시 확인, 순정과 같은 충전/완충 표시 구현 |
| 공장 2.4 GHz/동글 | USB 수신기, 외부 nRF24L01, ESB, 동글 페어링과 별도 숫자패드 통신 미구현 |
| NocFree Link 부가 기능 | 배터리 표시는 unavailable이며 Quick Text 저장·삭제·실행 미구현 |
| 기타 도구 | 공장 updater와 ZMK Studio 호환은 미구현이며 현재 프로젝트 필수 범위가 아님 |
| 플랫폼 범위 | Bluetooth host는 Windows 11과 Android만 검증. macOS, iOS, Linux와 세 번째 host는 미검증 |

순정 펌웨어와의 우선순위 비교 및 배터리 보정 방법은 [ROADMAP_ko.md](ROADMAP_ko.md),
상세 실험 기록은 [PROGRESS_ko.md](PROGRESS_ko.md)에 정리합니다.

## 구현된 기능

- **ANSI 입력:** 왼쪽 37키와 오른쪽 47키, 양쪽 Fn, 미디어·시스템 키, 한국어
  Windows 특수키와 암호화 BLE split 전송.
- **USB/Bluetooth HID:** 물리 스위치 출력 선택, BLE 자동 재연결, CCCD 상태 저장,
  선택 상태가 유지되는 페어링 슬롯 3개.
- **NocFree Link:** 8×84 키맵, 실행 가능한 hotkey 16개, CRC가 있는 flash 저장,
  삭제와 기본값 복구.
- **복구:** 양쪽 독립 1200-baud CDC DFU, 길게 누르는 Fn DFU, UF2 진입과
  Rust ↔ 순정 V2.3.0 복구 경로.
- **백라이트:** 양쪽 on/off와 0/20/40/60/80/100% 동기화, 10 kHz PWM, 눈에
  보이는 기본 linear 단계와 재현 가능한 perceptual A/B 곡선, 30초 자동 소등과
  첫 키 wake. `Fn+F5`는 감소, `Fn+F6`은 증가.
- **전원 동작:** interrupt 기반 idle 스캔과 250 ms 안전 스캔, 측정할 때만 배터리
  divider 활성화, 배터리에서 왼쪽 5분 후 System OFF.
- **배터리 표기:** 순정 V2.3.0 환산·필터 로직과 `Fn+I` 출력. 완충된 양쪽에서
  `L 100 R 100` 확인.
- **상태와 진단:** 지속되는 파란 페어링 표시, 자동 검사된 빨간 저전압 로직,
  USB 양쪽에서 읽을 수 있는 최근 split 이벤트 32개.
- **안전한 flash 범위:** SoftDevice, 영구 저장소, 공장 filesystem과 UF2 bootloader
  영역을 보존.

이전 ANSI 펌웨어는 84키 전체, Wired/Bluetooth, Windows 11·Android 멀티페어링,
물리 스위치, NocFree Link, 백라이트 동기화·소등, 전원 wake, 양쪽 DFU와 순정
복구를 실기에서 통과했습니다. 이 결과가 현재 `develop` 산출물의 실기 검증을
대체하지는 않습니다. 최신 인계 상태는 [HANDOFF.md](HANDOFF.md)를 참고하십시오.

## 기본 단축키

양쪽 Fn은 같은 layer를 사용합니다. 아래 표는 NocFree Link로 바꾸기 전 기본값입니다.

| 키 | 실행 | 동작 |
|---|---|---|
| `Fn+Esc` | 즉시 | 왼쪽 애플리케이션 재시작. DFU는 아님 |
| `Fn+F1` / `Fn+F2` | 즉시 | 화면 밝기 감소 / 증가 |
| `Fn+F3` / `Fn+F4` | 즉시 | Mission Control/Task View, Spotlight/Search |
| `Fn+F5` / `Fn+F6` | 즉시 | 양쪽 키보드 백라이트 20% 감소 / 증가 |
| `Fn+F7` / `Fn+F8` / `Fn+F9` | 즉시 | 이전 곡 / 재생·일시정지 / 다음 곡 |
| `Fn+F10` / `Fn+F11` / `Fn+F12` | 즉시 | 음소거 / 음량 감소 / 증가 |
| `Fn+1` / `Fn+2` / `Fn+3` | 짧게 | Bluetooth 페어링 슬롯 1 / 2 / 3 선택 |
| `Fn+1` / `Fn+2` / `Fn+3` | 1초 | 해당 bond 삭제 후 새 페어링 시작 |
| `Fn+5` | 3초 | **왼쪽** UF2 bootloader 진입 |
| `Fn+0` | 3초 | **오른쪽** UF2 bootloader 진입 |
| `Fn+Tab` | 즉시 | 양쪽 백라이트 토글 |
| `Fn+I` | 3초 | 활성 출력으로 `L {왼쪽 %} R {오른쪽 %}` 입력 |
| `Fn+Delete` | 3초 | 오른쪽 DFU 호환 단축키. 새 작업은 `Fn+0` 권장 |
| `Fn+M` | 짧게 / 1초 | M 입력 / macOS 모드 저장 |
| `Fn+N` | 짧게 / 1초 | N 입력 / Windows 모드 저장 |

표에 없는 Fn 조합은 원래 키를 입력합니다. `Fn+6`, `Fn+7`은 profile 삭제나 출력
전환을 하지 않습니다. 대신 `Fn+1/2/3` 길게 누르기와 물리 스위치를 사용합니다.
`Fn+U`, `Fn+B`도 원래 글자를 입력합니다.

## 기술 참고

### 좌우 입력 순서

오른쪽 snapshot은 원본 시각·순번·재조정 정보를 보내고, 왼쪽은 split 준비 전에
시계 차이를 계산해 60초마다 갱신하며 로컬·원격 이벤트를 한 queue에서 합칩니다.

입력 순서 대기값은 실기 검증한 **3 ms**에서 **8 ms** 후보를 거쳐 현재 절충값
**5 ms**로 변경했습니다. 빌더는 `src/scanner.rs`의 `REORDER_WINDOW_MS`를 조정할
수 있습니다. 작은 값은 지연을 줄이고 큰 값은 BLE 전송 jitter 여유를 늘립니다.
변경 후 양쪽을 함께 다시 빌드·플래시하고 USB와 Bluetooth에서 빠른 L-R-L 및
R-L-R 입력을 검사해야 합니다. 컴파일 성공만으로 안전한 값을 정할 수 없습니다.

### Bluetooth profile과 split 연결

페어링 슬롯 1/2/3은 하나의 BLE identity 아래 저장된 bond이며 서로 다른 키보드
3대가 아닙니다. `NocFree 1/2/3`은 선택한 슬롯의 광고 이름입니다. 현재 host에
연결된 슬롯으로 돌아가면 되며, profile을 바꿀 때마다 Windows 장치를 삭제하는
방식은 정상 절차가 아닙니다.

오른쪽 split은 1M BLE, 암호화, 7.5 ms 연결 주기, latency 30, 4초 supervision
timeout, ATT MTU 23, 단계식 미연결 광고와 +8 dBm TX 출력을 사용합니다. 복구는
개선됐지만 거리·전력·장시간 동작은 위 미검증 목록에 남아 있습니다.

### 전원, 진단과 복구

- NocFree 키 입력이 30초 없으면 USB 연결 중에도 양쪽 백라이트가 꺼집니다.
- 배터리에서 왼쪽은 5분 후 System OFF에 진입합니다. USB 전원과 페어링 중에는
  진입하지 않습니다. USB를 연결하거나 재연결될 때까지 왼쪽 키를 길게 눌러 깨웁니다.
- 오른쪽은 BLE split 자동 재연결을 위해 저활동 System ON을 유지합니다. 오른쪽
  System OFF는 구현하지 않았습니다.
- 키보드 상태를 바꾸지 않고 split 로그를 읽을 수 있습니다.

```powershell
& '.\tools\read-split-diagnostics.ps1' -Role Left
& '.\tools\read-split-diagnostics.ps1' -Role Right
```

진단은 115200 baud, DFU touch는 1200 baud를 사용합니다. 확인되지 않은 PCB pad를
단락하지 말고 [RECOVERY_ko.md](RECOVERY_ko.md)를 따르십시오.

## 관련 문서

- [복구와 순정 펌웨어 복귀](RECOVERY_ko.md)
- [배열별 안내](LAYOUTS_ko.md)
- [미구현 기능과 배터리 보정 계획](ROADMAP_ko.md)
- [상세 개발·검증 기록](PROGRESS_ko.md)
- [후속 작업 인계](HANDOFF.md)

로컬 `vendor/nrf-softdevice`와 `vendor/nrf-softdevice-macro`는 upstream commit
`b0ac850c0a5a05b8a5aef4f752b48115755b8542` 기반입니다. 각
`README.nocfree_ko.md`에 1M PHY와 secure CCCD 수정 이유가 있습니다.
