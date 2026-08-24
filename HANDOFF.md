# NocFree Rust 펌웨어 작업 인계서

마지막 갱신: 2026-08-24 (Asia/Seoul)

이 문서는 다른 AI나 작업자가 대화 기록 없이 현재 상태를 이어가기 위한 기준
문서입니다. 먼저 `git status`, 양쪽 USB 부모 ID, 산출물 해시를 확인하십시오.

## 1. 목표와 판정

`NocFree-and-zmk`의 NocFree & ANSI 동작을 Rust 펌웨어로 옮기고 다음을 실제
장치에서 검증하는 것이 목표였습니다.

1. ZMK 단축키와 양쪽 84키 물리 배열
2. USB/Bluetooth의 좌우 입력과 교차 입력 순서
3. `link.nocfree.com` 또는 ZMK Studio를 통한 키 변경과 재부팅 보존
4. 양쪽 DFU, 역할별 순정 복귀, 같은 Rust 펌웨어 재설치
5. 왼쪽 3단 모드 스위치와 오른쪽 배터리 ON/OFF 스위치

2026-08-23 최신 이미지로 다섯 항목을 모두 실기 검증했습니다. 키 변경 경로는
NocFree Link를 선택했으며 ZMK Studio는 구현하지 않았습니다. Link quick text는
빈 조회 응답만 제공하고 저장/실행은 구현하지 않았습니다. 일반 키 변경과
hotkey는 구현·실기 검증됐습니다.

2026-08-24에 빠른 좌우 입력의 `jam -> ajm`, 약 30 cm에서 오른쪽 재연결 실패
후 가까이 가져가야 연결되는 현상, 좌우 백라이트가 반대로 유지되는 현상이
재현됐습니다. 백라이트는 왼쪽 기준 절대 상태로 바꾼 뒤 실기 재검증을 통과했고,
입력 순서는 P4 timestamp/sequence와 3 ms 전역 대기열로 자동·실기 검증을
통과했습니다. split 재연결의 +8 dBm 거리·전력 통제 비교는 남았습니다. 진행은
`PROGRESS.md`를 따릅니다.

## 2. 저장소

### Rust 구현

- 경로: `D:\study\nocfree\NocFree-and-rust`
- branch: `main`
- remote: `https://github.com/jhkim0218/Nocfree-and-ZMK-rust.git`
- Link 구현 체크포인트: `e936062 feat: integrate NocFree Link keymap protocol`
- 최종 상태는 `git log -1`과 `git status`를 권위 있는 값으로 사용

### ZMK 기준

- 경로: `D:\study\nocfree\NocFree-and-zmk`
- remote: `https://github.com/jhkim0218/NocFree-and-zmk.git`
- branch: `main`
- 기준 commit: `e5e2f470795e92609f7ee6e810470fa6976557d1`

## 3. 현재 장치와 역할

마지막 확인에서 양쪽 최신 Rust 앱이 상태 `OK`로 열거됐습니다.

| 역할 | 책임 | USB 부모 Instance ID |
|---|---|---|
| 왼쪽 / `central` | 왼쪽 37키, 전체 키맵, 오른쪽 split 수신, USB/BLE HID, Link, BLE 프로필 | `USB\VID_2886&PID_8029\RUST-LEFT` |
| 오른쪽 / `right` | 오른쪽 47키, 암호화 BLE split 송신, 독립 CDC 복구 | `USB\VID_1D50&PID_615E\RUST-RIGHT` |

오른쪽은 호스트 입력용 HID가 없습니다. 오른쪽 키는 split으로 왼쪽에 전달된 뒤
왼쪽 HID로 PC에 입력됩니다. COM 번호는 바뀔 수 있으므로 항상 포트의
`DEVPKEY_Device_Parent`로 역할을 확인하십시오. 마지막 번호는 왼쪽 COM19,
오른쪽 COM18이었지만 이를 고정값으로 사용하면 안 됩니다.

현재 출력 모드는 USB입니다. Link 테스트에서 바꾼 A와 hotkey는 모두 기본값으로
복구돼 있습니다.

## 4. 구현 요약

### 물리 입력과 split

- PCA9555: `0x20`, `0x22`, `0x24`
- I2C: SDA P0.11, SCL P1.09, 100 kHz
- active-low, press/release 5 ms debounce
- 왼쪽 37키 + 오른쪽 47키 = 84키
- 시작 설정 실패는 최대 1초 backoff로 계속 재시도
- TWIM 생성 전 open-drain SCL 최대 9회와 STOP으로 I2C bus clear
- 오른쪽 split은 별도 Just Works bond와 flash page 사용
- 오래된 split 키로 보안 연결이 실패하면 그 키만 삭제하고 자동 재페어링
- 양쪽 scanner는 원본 시각과 순번을 붙이고, 오른쪽은 20-byte 암호화 ATT
  snapshot으로 전달. 왼쪽은 연결 전 3회·60초 주기 시계 보정 뒤 양쪽을 같은
  3 ms 대기열에서 원본 시각순으로 처리
- interval 7.5 ms, latency 30, supervision timeout 4초
- vendor patch로 모든 BLE PHY를 1M에 고정
- 백라이트는 왼쪽이 `enabled`/밝기/timeout/generation 절대 상태를 소유하고,
  암호화 GATT write-with-response로 변경과 split 재연결 때 오른쪽에 전체 동기화
- USB/BLE 전환 때 이전 출력에 release 전송

### 물리 스위치

- 왼쪽 P0.15/P0.17은 pull-up active-low 모드 감지 입력
- 위 2.4G: transport 미구현이므로 USB/BLE 출력 모두 비활성
- 가운데 Wired: USB HID 출력
- 아래 Bluetooth: BLE HID 출력
- 두 핀이 동시에 low인 전환 순간에는 이전 모드를 유지하고, 20 ms 안정 위치만 적용
- Wired에서 왼쪽 USB가 없으면 HID 출력은 없지만 MCU 전원은 꺼지지 않음
- 오른쪽 위 OFF/아래 ON은 펌웨어가 읽지 못하는 배터리 전원 스위치
- 오른쪽 USB VBUS는 배터리 스위치를 우회하므로 USB 연결 중에는 OFF도 보드에 전원 공급

### Link 동적 키맵

- VID/PID `0x2886:0x8029`
- product `NocFree & ANSI`, serial `RUST-LEFT`
- vendor class bulk IN/OUT + MS OS 2.0 WINUSB descriptor
- frame `[FF FE op len payload FE FF]`
- 8 layers × 84 physical keys, UI 6 rows × 21 columns
- SET/GET key, layer row, clear layer/all, system/version/battery
- hotkey 16개 SET/GET/CLEAR/DELETE와 실제 HID chord
- `ReportEngine::apply_snapshot_with`가 매 입력에서 동적 binding 조회
- page 7 `0x6c000`에 CRC 포함 version-3 record 저장. version 2는 새 기본 Fn
  레이어 적용을 위해 한 번 무효화되며 BLE/split bond는 보존
- quick text GET은 빈 슬롯, SET/CLEAR/DELETE/실행은 미지원

### 단축키

- `Fn+1`/`Fn+2`/`Fn+3`: 짧게 프로필 선택, 1초 홀드로 해당 프로필 페어링
- `Fn+5`: 3초 홀드로 왼쪽 UF2 부트로더
- `Fn+0`: 3초 홀드로 오른쪽 UF2 부트로더
- `Fn+I`: 3초 홀드로 `L {왼쪽 배터리} R {오른쪽 배터리}` 입력
- `Fn+Tab`, `Fn+F5/F6`: 백라이트 토글/밝기 조절
- `Fn+U`: USB 출력
- `Fn+B`: BLE 출력
- `Fn+F7` .. `Fn+F12`: 미디어 제어
- `Fn+M`/`Fn+N`: 짧게 M/N, 1초 홀드로 macOS/Windows 모드 저장
- `Fn+Esc`: 왼쪽 앱 재시작, DFU 아님
- `Fn+Delete`: 이전 복구 호환용 오른쪽 UF2 부트로더
- 양쪽 독립 복구: 해당 반쪽 CDC 1200-baud touch

## 5. flash 메모리

- `0x00000..0x26fff`: MBR + S140 7.3.0, 보존
- `0x27000..0x64fff`: Rust application
- `0x65000..0x69fff`: BLE host profiles
- `0x6a000..0x6afff`: selected profile/settings
- `0x6b000..0x6bfff`: split bond
- `0x6c000..0x6cfff`: Link keymap/hotkeys
- `0x6d000..0x73fff`: factory filesystem, 보존
- `0x74000..0x7ffff`: UF2 bootloader/metadata, 보존

최신 UF2 실제 범위는 왼쪽 `0x27000..0x3acff`, 오른쪽
`0x27000..0x329ff`입니다.

## 6. 최신 산출물

| 파일 | 크기(bytes) | SHA-256 |
|---|---:|---|
| `firmware/NocFree_Rust_Left.bin` | 80,964 | `22F38C6EA74CB155F19D3AA219CC5E3EF6239F73D9433C1750FBA3983AE436DD` |
| `firmware/NocFree_And_Rust_ZMK_Based_ANSI_Left.uf2` | 162,304 | `031F4FE1299F439153A405358B52E5C92E7D3E68B5F0DB803918FE1798699DF3` |
| `firmware/NocFree_Rust_Left_DFU.zip` | 81,840 | `DF597EA1650313A2917B4E2956EDDF801B0D165C470D7B6C1B817D137F947FFB` |
| `firmware/NocFree_Rust_Right.bin` | 47,508 | `1D6CDBB284AA9008AE0D89CFFB10EB979EC7017B0EEBFB76AE81848139D23BA4` |
| `firmware/NocFree_And_Rust_ZMK_Based_ANSI_Right.uf2` | 95,232 | `453BC8AAB3A762C9A9CBC6A7E6D4159B97FEAE55DE5D79CF1E3C0C075FCBDACE` |
| `firmware/NocFree_Rust_Right_DFU.zip` | 48,390 | `A181DCA39D8E6E7CCDFAAB3C3DB7DF72FF109A8FBB1E6FC091BC5D38BF34106F` |

DFU ZIP은 application-only이며 자동 테스트가 ZIP 내부 BIN과 같은 역할의 최신
`.bin`이 일치하는지 검사합니다. 키보드 코드는 모두 Rust `no_std`이고 Python은
PC 빌드/검사 도구에만 사용됩니다.

역할별 순정 UF2:

| 역할 | 파일 | SHA-256 |
|---|---|---|
| 왼쪽 | `D:\study\nocfree\NocFree_and_V2.3.0_Left_ANSI.uf2` | `A3FF612B94E9CE0C12BEFD9FF19ECDE5D6E4DB964C0345E432B71ED7A2C5BC91` |
| 오른쪽 | `D:\study\nocfree\NocFree_and_V2.3.0_Right_ANSI.uf2` | `E1F851906B3E35117B8A8AAC09E5C8273D75921F64D1D3B1496E60A53D3E1C66` |

## 7. 자동 검증

```powershell
$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
Set-Location -LiteralPath 'D:\study\nocfree\NocFree-and-rust'
& pwsh -NoProfile -ExecutionPolicy Bypass -File '.\tools\build-release.ps1'
$buildExit = $LASTEXITCODE
if ($buildExit -ne 0) { throw ('build failed with exit code {0}' -f $buildExit) }
```

마지막 결과:

- Rust host tests: 71/71
- Python contract/artifact tests: 18/18
- fmt: 통과
- host lib 및 central/right ARM Clippy `-D warnings`: 통과
- central/right release: 통과
- BIN/UF2 주소·family·vector·round-trip: 통과
- DFU ZIP manifest와 내부 BIN: 통과

### P2 split 재연결 진단

- `tools/read-split-diagnostics.ps1 -Role Left|Right`는 각 반쪽 USB CDC를
  115200 baud로 열어 최근 32개 RAM 이벤트를 읽음
- 왼쪽: scan, 광고 identity/RSSI, connect, 실제 파라미터, security, GATT,
  split-ready, disconnect reason, attempt 기록
- 오른쪽: advertising mode/interval, connect/security, disconnect reason,
  미연결 상태 key press 기록
- 약 30cm 실기에서 RSSI `-75/-74 dBm`, 첫 reconnect의 MTU exchange 실패,
  다음 attempt의 성공을 구분했고 HCI disconnect `0x08`도 확인
- 진단 출력 추가 뒤 양쪽 독립 1200-baud CDC DFU와 같은 P2 이미지 복귀,
  왼쪽 `qwer`/오른쪽 `jkl` 입력을 재검증
- 이 단계는 관측성만 추가했습니다. radio parameter와 reconnect policy tuning은
  P3에서 한 변수씩 변경해야 합니다.

### P3.1 표준 ATT MTU

- P2의 `ConnectError::MtuExchange`를 근거로 split central만 ATT MTU 23을
  요청하도록 변경. 최대 split 값 8바이트는 MTU 23의 값 용량 20바이트에 들어감
- 광고, TX 출력, 7.5 ms 연결 주기, latency 30, supervision timeout 4초는 유지
- 약 30cm 오른쪽 전원 재인가 2회가 거리 이동과 복구 없이 입력까지 성공;
  RSSI `-85/-79 dBm`, split-ready `55,431/17,736 ms`로 속도 문제는 남음
- Wired USB와 기존 Windows 11 Bluetooth 출력에서 오른쪽 입력 통과
- 왼쪽 물리 스위치를 Wired로 둔 1200-baud 복구에서 `RUST-LEFT` COM19 →
  `VID_239A&PID_0029` COM9/G:, serial `52CF50988BD1E6EE`를 확인하고 같은
  P3.1 UF2 복귀. 이후 ATT MTU 23, 첫 시도 4,358 ms split-ready 확인
- P3 다음 단계는 오른쪽의 BLE 연결 가능 알림 패킷(advertising) 주기를 단계화하고
  미연결 키 입력 때 빠른 주기로 복귀시키는 것. 이 결과를 측정하기 전에는 TX
  출력과 latency를 동시에 바꾸지 말 것
- `NocFree 1/2/3`은 하나의 BLE identity를 공유함. 다른 PC에 bond된 슬롯을
  현재 Windows에서 선택하면 같은 항목 이름이 바뀌고 페어링됨/연결됨이 반복될
  수 있으므로, 현재 PC의 슬롯으로 돌아가야 함

### P3.2 단계별 연결 알림 패킷

- 오른쪽 단절 시 250 ms 10초 → 500 ms 50초 → 1초 장기, 미연결 오른쪽
  키 입력 시 즉시 250 ms 단계로 복귀
- 2초/3초 진단 이미지로 fast/medium/idle과 key→fast 기록을 먼저 실기 확인
- 최종 이미지 약 30cm 재인가 2회 입력 성공: 6.142초와 17.208초. 두 번째는
  -85 dBm 보안 timeout 뒤 -79 dBm 자동 재시도로 회복
- Wired `jkljj`, Windows 11 Bluetooth 오른쪽 `ㅓ` 통과. Bluetooth는 사용자가
  지연을 체감했고 Windows 삭제/재페어링은 하지 않음
- 오른쪽 COM18 1200-baud → serial `D82A03513BB02626` F: → 같은 최종 이미지
  복구 → RUST-RIGHT, 1.295초 연결과 60 ms 보안 확인
- P3 다음 변수는 오른쪽 TX 출력 비교. latency와 connection timing은 유지

### P3.3 +8 dBm 설정 및 P4 배포

- 현재 오른쪽은 광고와 연결 TX 모두 +8 dBm인 P4 이미지가 배포됨
- P4 Wired/Windows 11 Bluetooth 입력은 통과했지만 +8 dBm 자체의 거리·보안·전력
  효과는 통제 비교하지 않았으므로 그 효과를 검증됐다고 표현하지 말 것

### P4 좌우 입력 순서 완료

- 오른쪽 snapshot: state u64 + source timestamp u64 + sequence u16 + flags/reserved
  2 bytes = 기본 MTU 값 한도 20 bytes
- 왼쪽은 split-ready 전에 3회 왕복 중 최소 RTT로 시계 offset을 잡고 60초마다 갱신
- 로컬 왼쪽도 즉시 출력하지 않고 오른쪽과 같은 3 ms `SnapshotOrderer<32>` 사용
- 10,000개 합성 교차 입력에서 1/2 ms 실패, 3/4/5 ms 무오류라 3 ms 선택
- 실기 Wired `jam` 10회와 긴 `ja`, Windows 11 Bluetooth `asdfjkljam…` 통과
- 오른쪽 `Fn+0`, 왼쪽 `Fn+5` 순서로 역할 시리얼 확인 후 P4 배포·복귀

저장소 기본 target은 MCU이므로 `cargo test --all-targets`를 그대로 실행하면
`std`가 없는 `thumbv7em-none-eabihf`에서 실패합니다. 반드시 빌드 스크립트나
`cargo test --target x86_64-pc-windows-msvc --package nocfree-and-rust`를
사용하십시오.

사용자 작업 규칙: 기능을 개발하면 자동 테스트와 필요한 실기 테스트를 끝낸
직후 문서를 갱신하고 Git commit까지 완료합니다. 검증된 기능 변경을 working
tree에 장기간 남겨 두지 않습니다.

## 8. 최신 실기 증거

### USB와 물리 키

- 좌우/교차: `asdfjkljamjamjam`, `jamjamjamjam`, `jajaj...`
- 문자 배열:

```text
`1234567890-=
qwertyuiop[]\
asdfghjkl;'
zxcvbnm,./
```

- 비문자 자동 sweep 33/33
- Print Screen은 Windows가 먼저 캡처하므로 수동 스크린샷으로 확인
- 한국어 Windows에서 Right Alt는 `KanaMode`로 보고되는 정상 VK alias
- 양쪽 Fn 확인
- 순정 V2.3.0의 ADC 3.3 V 환산, `130/100` divider, 75/25 IIR filter와
  2.31–3.30 V 퍼센트 계산을 복구해 구현
- 양쪽 최신 이미지에서 `Fn+I` 3초 홀드로 완충 상태 `L 100 R 100` 확인.
  전체 방전 주기와 DMM 비교는 장기 검증으로 남음
- 왼쪽 파란 LED는 `Fn+3` 페어링을 시작한 뒤 키에서 손을 떼도 계속 점멸하고,
  `Fn+1`로 bond 슬롯을 선택하면 꺼지는 것을 실기 확인
- 양쪽 빨간 LED는 공유 charger 선을 직접 high로 구동하지 않는 open-drain
  방식이며 10% 이하에서 0.5초 간격 점멸. 자동 테스트는 통과했으나 양쪽이
  완충이라 실물 저전압 점멸은 미확인
- idle 스캔은 기존 10 ms polling 대신 왼쪽 `P0.31`/오른쪽 `P0.05` PCA9555
  active-low INT 또는 250 ms 안전 timer로 깨어남. debounce 중에는 3 ms 유지
- 양쪽 모두 3초 idle 뒤 첫 키 즉시 입력, 2초 hold 반복, release 중단과 좌우
  혼합 입력 순서 실기 통과. idle 전체 스캔은 약 100회/s에서 4회/s로 감소
- Embassy `SimplePwm`이 nRF PWM sample의 polarity bit를 제거하므로 논리 밝기를
  hardware compare 값으로 반전해 ZMK의 `PWM_POLARITY_NORMAL`과 맞춤. 수정 전에는
  0%가 실물에서 완전 점등으로 출력되어 `Fn+Tab`과 timeout이 모두 소등되지 않았음
- USB 전원을 연결한 상태에서 10초 진단 이미지로 `Fn+Tab` 양쪽 소등/점등,
  10초 자동 소등, 첫 키 입력과 동시 wake를 실기 확인. 저장소와 배포 artifact는
  같은 경로에서 상수만 30초로 변경했고 이후 deep-sleep 진단 이미지부터 실물에도
  30초 백라이트 timeout이 설치됨
- 절대 상태 P1 이미지에서 왼쪽 수동 OFF 중 오른쪽 전원 재부팅으로 의도적으로
  상태를 어긋낸 뒤 재연결 한 번으로 OFF에 수렴. `Fn+Tab` 10회 동안 반전 없음,
  30초 양쪽 자동 소등과 오른쪽 첫 키의 양쪽 wake, 재연결 오른쪽 입력을 확인
- 왼쪽 central은 배터리 상태에서 NocFree 키 무입력 5분 뒤 System OFF. USB 전원,
  BLE pairing, low 상태의 왼쪽 PCA9555 INT에서는 진입하지 않음. 진입 전 양쪽
  백라이트 Idle 명령을 보내고 split 최대 latency 232.5 ms보다 긴 300 ms를 기다림
- 동일 경로의 10초 진단 이미지에서 USB 연결 중 sleep 차단, 배터리 System OFF,
  양쪽 백라이트 소등, 왼쪽 USB wake, 왼쪽 키 홀드 wake, BLE와 오른쪽 입력 복귀 확인.
  System OFF는 reset wake라 짧은 wake key 탭의 문자 보존은 보장하지 않음
- 오른쪽은 왼쪽 wake 후 자동 재연결을 위해 System OFF에 넣지 않음. CPU idle과
  PCA9555 INT scan을 유지하며 최초 split 연결 뒤 단절 광고 주기를 250 ms에서
  1초로 늦춤. 광고 송신 빈도는 1/4이지만 실제 전류 감소량은 아직 미측정

### Link

- Chrome/Edge에서 `NocFree & ANSI`와 키보드 화면 확인
- 물리 A를 B로 변경 후 `b`
- `Fn+Esc` 뒤에도 `b`로 persistence 확인
- A에 Shift+B hotkey를 묶어 `B`, 재부팅 후에도 `B`
- hotkey 삭제와 기본 A 복구 후 `a`

### BLE

- `blejamjamjam`
- `Fn+U`의 `usbok`, `Fn+B`의 `bleback`
- BLE 상태에서 `Fn+Esc`, Windows 연결 조작 없이 `bleautoreconnect`
- 최신 UF2 설치 직후 첫 전환은 Windows의 stale 세션 때문에 장치를 삭제/재등록
- 같은 이미지의 다음 Wired→BLE 전환은 Windows 조작 없이 `bleagainok` 자동 재연결

### 물리 스위치

- 왼쪽 가운데 Wired: `wiredswitchok`, 복귀 `wiredbackok`
- 왼쪽 아래 Bluetooth: 첫 등록 후 `bleswitchok`, 반복 전환 `bleagainok`
- 왼쪽 위 2.4G: 오른쪽 키 `jkl`도 출력되지 않음, Wired 복귀 `24silentok`
- 오른쪽 USB 분리/스위치 아래 ON: 오른쪽 `jkluiop`
- 오른쪽 스위치 위 OFF: 오른쪽 무입력, 왼쪽 `asdf`는 계속 입력
- 오른쪽 스위치 아래 ON 복귀: split 자동 재연결 후 오른쪽 `jkluiop`

### DFU와 순정 원복

양쪽 모두 최신 계열 이미지로 다음 사이클을 실제 통과했습니다.

- `Fn+5`로 왼쪽 DFU 진입 확인
- DFU 단축키를 3초 홀드로 변경한 뒤 짧은 `Fn+5`가 DFU에 들어가지 않음 확인
- `Fn+0` 3초 홀드로 오른쪽 DFU 진입, 최신 오른쪽 UF2 복귀와 `jkl` 확인

1. Rust CDC 1200 touch → NocFree & / S140 7.3.0 UF2
2. 역할별 순정 V2.3.0 UF2 설치와 순정 부모 ID 확인
3. 순정 CDC 1200 touch → 정확한 칩 시리얼의 `PID_002A` bootloader COM
4. 역할별 Rust serial DFU ZIP 전송
5. `RUST-LEFT`/`RUST-RIGHT` 복귀와 입력 확인

오른쪽 순정 왕복 직후 외부 I/O 확장칩이 전원 유지 상태에서 걸려 입력이
멈췄고 케이블 전원 재인가로 복구됐습니다. 이를 근거로 양쪽 부팅 전 I2C bus
clear를 구현했습니다. 최신 이미지 재플래시와 `Fn+Delete` 왕복 후에는 케이블을
뽑지 않고 `jkluiop`가 통과했습니다.

최신 `Fn+Delete` 검증:

1. 사용자가 `Fn+Delete`
2. 오른쪽 앱 소멸과 `F:\INFO_UF2.TXT` 확인
3. 최신 오른쪽 UF2 설치
4. 10초 split 재연결
5. 케이블 전원 재인가 없이 `jkluiop` 통과

## 9. 발견·해결한 장애

- USB interface limit 4 → Link가 5번째 interface여서 panic
- interface limit만 5로 올린 뒤 handler limit 4에서 다시 panic
- 두 값을 모두 5로 설정하고 panic handler를 UF2 fail-safe로 변경
- PCA9555 최초 설정 오류에서 영구 대기 → backoff 재시도
- 순정/DFU 재부팅 뒤 I2C slave bus lock → 부팅 전 9-clock + STOP
- 오래된 split bond 보안 실패 → 해당 split bond만 삭제 후 재페어링
- `adafruit-nrfutil --touch 1200`은 이 PC에서 내부 전송 실패를 출력하면서도
  exit code 0을 반환한 사례가 있음. native exit code만으로 성공 판정 금지

자세한 복구 안전 조건은 [RECOVERY.md](RECOVERY.md)에 있습니다.

## 10. 사용자 알림 규칙

사용자 키/케이블/브라우저 조작이 필요하면 다음 helper로 Windows 알림을 띄운
뒤 즉시 멈추고 다음 사용자 메시지를 기다립니다. 몇 초간 키 입력을 polling하면
사용자가 다른 화면을 보는 동안 놓칩니다.

```powershell
$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
& pwsh -NoProfile -ExecutionPolicy Bypass -File `
    'D:\study\nocfree\NocFree-and-rust\tools\notify-user.ps1' `
    'Codex에서 키보드 확인이 필요합니다.'
$notifyExit = $LASTEXITCODE
if ($notifyExit -ne 0) { throw ('notify failed with exit code {0}' -f $notifyExit) }
```

helper는 system sound와 TopMost Windows Forms 창을 띄우며 사용자가 확인했습니다.

## 11. PowerShell 안전 규칙

모든 스크립트는 다음으로 시작합니다.

```powershell
$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
```

- native process 직후 `$LASTEXITCODE` 저장·확인
- 단, 프로그램 자체가 실패를 0으로 반환할 수 있으므로 성공 메시지/장치 상태도 확인
- `@(query)`로 0/1/N 결과를 배열화한 뒤 `.Count` 사용
- `$var:` 대신 `-f` 사용
- COM 번호가 아니라 USB parent property로 역할 확인
- UF2 복사 직전에 역할, 파일명, SHA-256을 다시 대조
- PowerShell native parameter와 cmd/다른 shell parameter를 섞지 않음

## 12. 코드 읽기 순서

1. `src/keymap.rs`, `src/link_keymap.rs`
2. `src/link_protocol.rs`, `src/link_usb.rs`
3. `src/report.rs`
4. `src/scanner.rs`, `src/pca9555.rs`, `src/hardware_scanner.rs`
5. `src/bin/central.rs`
6. `src/bin/right.rs`, `src/split_protocol.rs`, `src/split_ble.rs`
7. `src/platform.rs`, `src/bond_store.rs`, `src/bond_record.rs`
8. `tools/build-release.ps1`, `tools/test_*.py`, `RECOVERY.md`

`vendor/nrf-softdevice`와 macro는 upstream
`b0ac850c0a5a05b8a5aef4f752b48115755b8542`의 로컬 사본입니다.
