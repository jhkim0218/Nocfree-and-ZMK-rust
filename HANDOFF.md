# NocFree Rust 펌웨어 작업 인계서

마지막 갱신: 2026-08-21 (Asia/Seoul)

이 문서는 다른 AI나 작업자가 별도 대화 기록 없이 현재 작업을 이어가기 위한
단일 기준 문서입니다. 핵심 목표인 USB/BLE 입력과 양쪽 순정 원복은 실제
하드웨어에서 통과했습니다. 아래의 검증 범위와 미실시 항목을 구분해 인용해야
합니다.

## 1. 최종 목표와 범위

- 기준 저장소 `NocFree-and-zmk`의 동작을 nRF52833용 `no_std` Rust
  펌웨어로 옮긴다.
- 왼쪽과 오른쪽을 모두 Rust 펌웨어로 구동한다.
- USB와 Bluetooth에서 양쪽 키 입력을 실제 확인한다.
- Rust 펌웨어를 플래시한 뒤에도 각 반쪽이 DFU로 진입하고 역할에 맞는 순정
  UF2로 복귀할 수 있음을 실제 확인한다.

여기서 "모든 기능"은 기준 ZMK 설정에 들어 있던 기능을 뜻합니다. 기준 설정이
명시적으로 제외한 numpad, 공장 ESB/2.4 GHz 모드, 배터리 표시, 백라이트,
상태 LED, 물리 모드 스위치, Studio, deep sleep, gaming/low-latency 모드는
Rust 이식 범위에도 포함하지 않았습니다.

## 2. 저장소와 작업 폴더

### 기준 ZMK 저장소

- 경로: `D:\study\nocfree\NocFree-and-zmk`
- 원격 fetch/push: `https://github.com/jhkim0218/NocFree-and-zmk.git`
- 브랜치: `main`
- 기준 커밋: `e5e2f470795e92609f7ee6e810470fa6976557d1`
- 2026-08-21 확인 당시 상태: `main...origin/main`, 변경 파일 없음

### Rust 구현

- 경로: `D:\study\nocfree\NocFree-and-rust`
- 아직 Git 저장소가 아닙니다. 사용자의 명시적 요청 없이 초기화하거나 push하지
  마십시오.
- 주요 문서: `README.md`, `RECOVERY.md`, 이 파일 `HANDOFF.md`

## 3. 왼쪽과 오른쪽을 절대 혼동하지 말 것

| 역할 | Rust USB 부모 Instance ID | 현재 포트 | 책임 |
|---|---|---|---|
| **왼쪽 / central** | `USB\VID_1D50&PID_615E\RUST-LEFT` | COM17 | 왼쪽 37키, 전체 키맵, 오른쪽 상태 수신, USB HID, BLE HID, BLE 호스트 5개 프로필 |
| **오른쪽 / right** | `USB\VID_1D50&PID_615E\RUST-RIGHT` | COM18 | 오른쪽 47키 스캔, 암호화된 BLE split 송신, 독립 CDC 복구 |

COM17/COM18은 2026-08-21 마지막 확인 값일 뿐이며 재연결하면 바뀔 수 있습니다.
항상 포트의 `DEVPKEY_Device_Parent`를 읽어 위 고정 부모 ID로 역할을 확정한 뒤
작업하십시오. 오른쪽은 정상 상태에서도 호스트 입력용 HID가 없고 CDC만
열거됩니다. 실제 입력은 오른쪽에서 왼쪽으로 split 전송된 뒤 왼쪽 HID를 통해
호스트에 전달됩니다.

순정 장치 식별자는 다음과 같습니다.

| 역할 | 순정 USB 부모 Instance ID | 칩 serial |
|---|---|---|
| 왼쪽 | `USB\VID_2886&PID_8029\52CF50988BD1E6EE` | `52CF50988BD1E6EE` |
| 오른쪽 | `USB\VID_239A&PID_80D8\D82A03513BB02626` | `D82A03513BB02626` |

## 4. 현재 플래시된 최종 이미지

아래 네 파일을 2026-08-21에 다시 해시 확인했습니다. 두 UF2가 현재 각 반쪽에
플래시되어 있습니다.

| 파일 | 크기(bytes) | SHA-256 |
|---|---:|---|
| `firmware/NocFree_Rust_Left.bin` | 47,220 | `ADFDF7D5A46BC12250C933001B99EA72BED128C7115D8078A5896B61CF9B1607` |
| `firmware/NocFree_Rust_Left.uf2` | 94,720 | `7053E74EA87313A106AE15C02724816A0A69D6D4F3E6B722D852DDEA6A725992` |
| `firmware/NocFree_Rust_Right.bin` | 30,652 | `F11C5CC67C7B3395B42470078B8A055D71E2EF86F07E2C551350264CE2D2E174` |
| `firmware/NocFree_Rust_Right.uf2` | 61,440 | `480A9DB7ECC6146CCB54E37FB6F586AEEC48D5DD39EDF2F4FA2E91BACC0F38BA` |

순정 펌웨어에서 UF2 대용량 저장 장치가 나타나지 않을 때 쓰는 application-only
serial DFU 패키지도 보관되어 있습니다.

| 파일 | 크기(bytes) | SHA-256 |
|---|---:|---|
| `firmware/NocFree_Rust_Left_DFU.zip` | 48,096 | `CD8B19398BF13BF2E7AF5852D09ED31A60BEFFAE35942201942068D2BD0CA7F4` |
| `firmware/NocFree_Rust_Right_DFU.zip` | 31,534 | `0906C30D444075A8BD1BF86FE6C8EAD0C2BAD4FEB2BF3356A97D85356458C5A3` |

현재 장치 상태도 다시 확인했습니다.

- 왼쪽: `RUST-LEFT`, USB keyboard HID + consumer HID + CDC, 상태 OK
- 오른쪽: `RUST-RIGHT`, CDC만 열거, 상태 OK
- 현재 UF2 드라이브: 없음(두 반쪽 모두 애플리케이션 실행 중)

## 5. 구현된 기능

- PCA9555 세 개(`0x20`, `0x22`, `0x24`)를 SDA P0.11, SCL P1.09,
  100 kHz I2C로 읽습니다.
- active-low 입력, 5 ms press/release debounce, 10 ms idle/3 ms active
  scan, 절대 시각 스케줄링과 I2C backoff를 사용합니다.
- PCA9555 polarity `0x0000`, config `0xffff`를 기록 후 다시 읽어 검증하고,
  불일치하면 입력을 내보내지 않습니다.
- 왼쪽은 USB 16-byte NKRO keyboard report, 2-byte consumer report와 BLE
  HOGP를 제공합니다.
- 오른쪽 47키 상태는 별도의 Just Works bond를 사용하는 암호화 BLE split으로
  왼쪽에 전달됩니다.
- 호스트 BLE 이름은 `NocFree Rust`, 오른쪽 split 광고 이름은
  `NocFree Rust Right`입니다.
- 호스트용 BLE bond 5개와 split bond는 서로 다른 flash page에 저장됩니다.
- USB/BLE 출력 전환 때 이전 경로에 release를 보내 stuck key를 방지합니다.
- 스캔 및 출력 경로에 bounded queue가 있으며 가득 차면 가장 오래된 frame을
  버리고 최신 상태와 release를 보존합니다. 경로/연결 변경 때 queue를
  동기화해 오래된 입력이 재생되지 않게 했습니다.
- 왼쪽 로컬과 오른쪽 split 변경은 하나의 FIFO로 병합해 별도 queue의
  left-biased 선택으로 입력 순서가 뒤집히지 않게 했습니다.
- split 연결은 ZMK 기본값과 같은 7.5 ms interval(`6 × 1.25 ms`), latency 30,
  supervision timeout 4초를 사용합니다. 라이브러리 기본 50~250 ms를 그대로
  쓰면 빠른 교차 타이핑에서 오른쪽 입력이 다음 왼쪽 입력 뒤로 밀렸습니다.
- 모든 BLE PHY는 로컬 vendor patch로 1M만 사용합니다.
- CDC를 1200 baud로 열면 SoftDevice SVC로 GPREGRET `0x57`을 설정한 뒤
  UF2 부트로더로 재시작합니다.
- 내부 RC LF clock을 사용하며 DCDC와 radio power 증가는 사용하지 않습니다.

중요 수정: `src/platform.rs`의 SoftDevice 설정은
`central_sec_count: 1`이어야 합니다. 처음에는 0이어서 왼쪽 central이
오른쪽과 암호화된 SMP pairing을 할 수 없었고 오른쪽 입력이 전달되지
않았습니다. 1로 변경하고 양쪽을 다시 플래시한 뒤 오른쪽 입력이 정상화됐으며,
이 값은 repository contract test로 고정했습니다.

### 플래시 메모리 배치

- `0x00000..0x26fff`: MBR + S140 7.3.0, 보존
- `0x27000..0x64fff`: Rust 애플리케이션
- `0x65000..0x69fff`: BLE 호스트 프로필
- `0x6a000..0x6afff`: 선택된 프로필/설정
- `0x6b000..0x6bfff`: split bond
- `0x6c000..0x6cfff`: 미사용
- `0x6d000..0x73fff`: 공장 파일시스템, 보존
- `0x74000..0x7ffff`: UF2 부트로더/메타데이터, 보존

`memory.x`의 앱 범위는 FLASH `0x27000`, 길이 `0x3e000`; RAM은
`0x20010000`, 길이 `0x10000`입니다.

## 6. 키 바인딩

- `Fn+1` .. `Fn+5`: BLE 호스트 프로필 1..5 선택
- `Fn+0`: 현재 선택된 BLE 프로필 삭제
- `Fn+U`: USB 출력 강제 선택
- `Fn+B`: BLE 출력 강제 선택
- `Fn+F10/F11/F12`: 음소거/볼륨 내림/볼륨 올림
- `Fn+Esc`: **왼쪽 애플리케이션 재시작**. DFU가 아닙니다.
- `Fn+Delete`: 정상 split 링크를 통해 **오른쪽** UF2 부트로더 진입
- 왼쪽과 오른쪽 모두: 각 반쪽의 CDC 1200-baud touch로 독립 DFU 진입

전체 84키 물리 위치와 Fn 레이어는 `src/keymap.rs`를 기준으로 확인하십시오.

## 7. 자동 검증과 빌드

PowerShell에서 다음 한 명령으로 전체 검증과 release UF2 생성을 실행합니다.

```powershell
$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
Set-Location -LiteralPath 'D:\study\nocfree\NocFree-and-rust'
.\tools\build-release.ps1
```

마지막 전체 실행 결과:

- Rust 단위 테스트: 30/30 통과
- Python 테스트: 12/12 통과(3 UF2 + 9 repository contract)
- host library, central, right `clippy -D warnings`: 통과
- central/right release ARM 빌드: 통과
- ELF → BIN → UF2 생성과 주소/family/vector/round-trip 검증: 통과

스크립트는 포맷 검사, 테스트, clippy, 양쪽 release 빌드, `llvm-objcopy`,
UF2 생성/검증 중 하나라도 실패하면 중단합니다. `cargo size`는 설치되어 있지
않으므로 크기가 필요하면 Rust toolchain의 `llvm-size.exe`를 사용하십시오.

## 8. 실제 하드웨어 검증 결과

### 통과한 항목

- 최종 왼쪽/오른쪽 UF2 설치, 부팅, 역할별 USB 열거
- 양쪽 UF2 부트로더 0.9.2-39-g0147d71, Board-ID `NocFree &`,
  SoftDevice S140 7.3.0 확인
- 최종 이미지에서 오른쪽 키 입력이 왼쪽으로 전달되어 USB HID로 출력됨
- USB 교차 반쪽 조합 실제 확인: 오른쪽 단독으로 `yuiop`, 왼쪽 Shift를 누른
  채 오른쪽 Y를 눌러 대문자 `Y`; 사용자가 정확히 `yuiopY`를 확인함
- split interval/FIFO 수정 후 오른쪽 `j` → 왼쪽 `a` → 오른쪽 `m`을 빠르게
  반복한 `jamjamjamjam`이 정확한 순서로 입력됐고 오른쪽에서 시작하는
  `jajaj…` 교차 반복도 순서 역전 없이 확인됨
- USB에서 문자 50개와 `tools/key-sweep.ps1`의 비문자 34개를 합친 전체
  84개 물리 위치 통과. 양쪽 Fn으로 미디어 명령도 확인함
- Windows `NocFree Rust` pairing과 BLE HOGP 입력 통과. `blejamjamjam`으로
  좌우 교차 순서를 확인하고 `Fn+B`/`Fn+U`로 BLE/USB 전환 확인
- 음소거/해제, 볼륨 내림/올림을 실제 확인함
- 현재 정확한 왼쪽 이미지에서 DFU → 왼쪽 순정 UF2 → 순정 왼쪽 부모 ID →
  Rust serial DFU → `RUST-LEFT` 복귀 전체 사이클 통과
- 현재 정확한 오른쪽 이미지에서도 같은 순서로 오른쪽 순정 부모 ID와
  `RUST-RIGHT` 복귀 전체 사이클 통과
- 두 복구 사이클 뒤 `finalusbjamfinalblejamfinalusbback`으로 한 줄에서
  USB → BLE → USB와 좌우 교차 입력을 확인함
- `Fn+Esc`로 왼쪽 애플리케이션을 재시작한 뒤 `resetleftok` 입력 통과
- `Fn+Delete`로 오른쪽 UF2 부트로더 `NocFree & / S140 7.3.0`에 진입하고,
  같은 오른쪽 UF2(해시 `480A…8BA`) 재설치 후 `rightbackjam` 입력 통과

### 실제로 실시하지 않은 추가 항목

- BLE에서 USB와 동일한 전체 84키 sweep을 한 번 더 반복하지는 않음. BLE는
  좌우 교차 문자열과 출력 전환으로 검증함
- `Fn+2`..`Fn+5` 각 프로필의 실제 페어링과 `Fn+0` bond 삭제는 미실시
- 완전한 양쪽 전원 차단 후 장시간 BLE 재연결 시험은 미실시

## 9. 이후 변경 시 재검증 순서

펌웨어 소스를 변경하면 `tools/build-release.ps1`을 먼저 통과시키고 생성된
artifact 해시를 새로 기록하십시오. 하드웨어 검증은 왼쪽을 먼저 설치·복구해
정상 상태로 되돌린 뒤 오른쪽을 진행합니다. USB 84키, BLE 좌우 교차 입력,
USB/BLE 전환, 양쪽 DFU/순정 원복을 다시 확인해야 현재와 같은 완료 판정을
유지할 수 있습니다.

## 10. 순정 원복 파일

| 역할 | 파일 | 크기(bytes) | SHA-256 |
|---|---|---:|---|
| 왼쪽 | `D:\study\nocfree\NocFree_and_V2.3.0_Left_ANSI.uf2` | 295,936 | `A3FF612B94E9CE0C12BEFD9FF19ECDE5D6E4DB964C0345E432B71ED7A2C5BC91` |
| 오른쪽 | `D:\study\nocfree\NocFree_and_V2.3.0_Right_ANSI.uf2` | 162,304 | `E1F851906B3E35117B8A8AAC09E5C8273D75921F64D1D3B1496E60A53D3E1C66` |

두 파일 모두 standard UF2, nRF52833 family ID `0x621e937a`, 앱 시작 주소
`0x27000`임을 확인했습니다. 역할이 다른 UF2는 절대 복사하지 마십시오.
상세 안전 조건은 `RECOVERY.md`가 기준입니다.

DFU 때는 다음 원칙을 반드시 지킵니다.

- COM 번호가 아니라 USB 부모 ID로 대상 반쪽을 먼저 검증합니다.
- CDC를 1200 baud, DTR true로 열고 닫은 뒤 모든 볼륨에서
  `INFO_UF2.TXT`를 검색합니다.
- 장치가 즉시 재부팅하면서 `Close()`가 "없는 장치" 오류를 낼 수 있습니다.
  이 오류는 올바른 NocFree UF2 볼륨이 실제 나타난 경우에만 성공으로
  간주합니다. UF2 볼륨이 없으면 실패입니다.
- `INFO_UF2.TXT`에서 NocFree Board-ID와 S140 7.3.0을 확인한 뒤 복사합니다.
- 순정 펌웨어의 1200-baud touch 뒤 UF2 볼륨 없이 부트로더 CDC
  `USB\VID_239A&PID_002A\<칩 serial>`만 나타나면 공식 `adafruit-nrfutil`
  0.5.3.post16과 위 역할별 `_DFU.zip`을 115200 baud로 업로드합니다.
- 예상과 다른 반쪽이 사라지거나 UF2로 바뀌면 즉시 중단합니다.
- 발열, CDC 미열거, stuck key, UF2 볼륨 미등장 시 다른 반쪽으로 진행하지
  않습니다.

## 11. PowerShell 작업 규칙

사용자가 이전 PowerShell 구문/식별 실수의 재발을 특히 경계하고 있습니다.
모든 PowerShell 실행은 다음 두 줄로 시작합니다.

```powershell
$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
```

- `$LASTEXITCODE`는 `git`, `cargo`, `python`, 실행 파일 같은 **native process를
  실행한 직후에만** 검사합니다. `Get-Content` 같은 PowerShell cmdlet 뒤에는
  검사하지 않습니다.
- native process 출력을 pipeline으로 넘기기 전에 exit code를 먼저 저장하고
  검사합니다.
- `foreach (...) { ... }` 문 자체를 바로 pipeline에 연결하지 말고 결과를 먼저
  배열 변수에 받은 뒤 그 배열을 pipeline에 전달합니다.
- 문자열에 변수 뒤 콜론을 직접 붙이는 `$var:` 형태를 쓰지 말고
  `'{0}: {1}' -f $name, $value`처럼 `-f` formatting을 사용합니다.
- hex와 `X8` formatting 전에 명시적으로 정수형을 확정합니다.
- COM 하위 Instance ID에는 serial이 없을 수 있으므로 하위 ID 문자열만 보고
  왼쪽/오른쪽을 판별하지 않습니다. 반드시 부모 property를 읽습니다.
- recursive delete는 로컬 정책에 막혔습니다. `tools/__pycache__`는 ignore되어
  있으므로 제거를 억지로 시도하지 마십시오.

## 12. 사용자 확인이 필요할 때

사용자는 다른 화면을 보고 있는 경우가 많아 물리 키 입력이나 케이블 조작이
필요하면 이 PC에서 알림을 원합니다. Codex 자체의 확실한 내장 알림 설정은
확인되지 않았습니다. `tools/notify-user.ps1`은 system sound를 재생하고
640×220 TopMost Windows Forms 창을 taskbar에 최대 120초 동안 표시합니다.
`msg.exe`와 NotifyIcon은 이 PC에서 보이지 않거나 신뢰할 수 없어 폐기했습니다.

```powershell
$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
& pwsh -NoProfile -File 'D:\study\nocfree\NocFree-and-rust\tools\notify-user.ps1' 'Codex에서 키보드 확인이 필요합니다.'
$notifyExit = $LASTEXITCODE
if ($notifyExit -ne 0) { throw ('notify-user.ps1 failed with exit code {0}' -f $notifyExit) }
```

물리 확인이 필요하면 먼저 이 알림을 한 번 보내고 작업을 멈춘 뒤 사용자의
반응을 기다립니다. helper 실행 성공과 사용자가 실제로 메시지를 봤는지는
구분해야 합니다. 최초 Windows 메시지는 사용자가 보였다고 확인했으며,
새 TopMost 방식도 사용자가 `새 알림 보임`으로 확인했습니다. 별도 hidden
`Start-Process`로 실행하면 창이 보이지 않았으므로 `pwsh -File`로 직접
실행하십시오.

## 13. 코드 읽는 순서

1. `src/keymap.rs`: 84개 물리 입력과 Fn 동작
2. `src/scanner.rs`, `src/pca9555.rs`, `src/hardware_scanner.rs`: 입력 디코딩,
   I2C, debounce
3. `src/report.rs`, `src/output_router.rs`, `src/usb_descriptor.rs`: USB/BLE
   report와 출력 전환
4. `src/bin/right.rs`, `src/split_protocol.rs`, `src/split_ble.rs`: 오른쪽
   scanner와 암호화 split
5. `src/bin/central.rs`: 왼쪽 scanner, split, USB/BLE 통합
6. `src/platform.rs`, `src/ble_hid.rs`, `src/bond_store.rs`: SoftDevice, HOGP,
   bond/DFU
7. `tools/build-release.ps1`, `tools/nocfree_uf2.py`, `tools/test_*.py`: 빌드와
   contract/UF2 검증

`vendor/nrf-softdevice`와 `vendor/nrf-softdevice-macro`는 upstream 커밋
`b0ac850c0a5a05b8a5aef4f752b48115755b8542`의 로컬 사본입니다. 각 폴더의
`README.nocfree.md`에 변경 이유가 있습니다. vendor patch를 upstream으로
무심코 교체하지 마십시오.

## 14. 알려진 작업 환경 문제

- 기준 ZMK 저장소의 `tests/run.sh`는 Windows checkout의 CRLF
  (`core.autocrlf=true`) 때문에 그대로는 실패했습니다.
- 개별 Python board/keymap/role/flash 기능 테스트는 통과했습니다. hygiene는
  CRLF/cp949 문제로 실패했고 현재 source build artifact가 없어 일부 artifact
  테스트는 skip됐습니다. 기준 저장소는 수정하지 않았습니다.
- 임시 LF clone/WSL 방식은 로컬 정책에 막혀 수행하지 못했습니다. 이 문제는
  Rust 펌웨어의 현재 자동 테스트 통과 여부와는 별개입니다.
- PowerShell WinRT로 BLE 광고를 직접 진단하려던 시도는 event/API 제약으로
  실패했습니다. 임시 `ble_scan_temp.cs/.exe/.pdb`는 남아 있지 않습니다.
  대신 Windows에 실제 pairing하고 BLE HOGP 키 입력으로 검증했습니다.

## 15. 완료 판정

현재 상태는 **핵심 목표 완료**입니다. 최종 정확한 이미지에서 USB 전체 84키,
BLE 좌우 교차 입력, USB/BLE 전환, 양쪽 DFU → 역할별 순정 펌웨어 → 같은 Rust
펌웨어 복귀를 실제 통과했습니다. 따라서 "flash 후 양쪽 모두 DFU를 통해
원상복귀하고 다시 Rust로 돌아올 수 있다"고 결론 내릴 수 있습니다. 단, BLE
전체 84키 반복 sweep과 프로필 2~5/삭제, 장시간 전원 cycle은 실시하지 않은
추가 검증으로 명시해야 합니다.
