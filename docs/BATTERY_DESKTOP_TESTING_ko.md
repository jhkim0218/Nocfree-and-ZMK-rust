# 배터리 PC 표시 실험 브랜치 검증·실기 인계

브랜치: `feature/battery-desktop`. 상태: 로컬 소프트웨어 검증 통과, 하드웨어 릴리스 미완료.
작성일: 2026-09-13. 기존 배포 firmware 파일은 교체하지 않았고 실제 키보드·동글에 플래시하지 않았다.

## 구현 내용

- 왼쪽에서 좌우 잔량·측정 나이·연결 상태·장치 ID를 집계한다. USB Feature ID 1과 암호화 GATT로 동일한 20바이트 데이터를 제공한다.
- BLE BAS 대표값, 별도 큐의 배터리 알림, 동글의 선택적 배터리 탐색·캐시·USB 조회를 추가했다. 구형 동글 키 입력 패킷은 바꾸지 않았다.
- 오른쪽 단절과 오래된 값은 무효화한다. USB polling이 측정 시각을 갱신하지 않는다. 측정 기한은 백라이트 이벤트와 독립이다.
- Windows 트레이와 macOS 메뉴 막대 소스를 추가했다. 앱은 직접 BLE를 30초 주기의 최대 5초 읽기 전용 세션으로 조회한다. CCCD 재구독·연결 유지 옵션·CDC 포트·키 입력 캡처는 사용하지 않는다. 동글 내부 연결은 알림을 한 번 구독하고 실패한 구독만 재시도한다.
- 기존 NocFree Link READ_BATTERY는 unavailable 상태를 유지한다. 새 앱은 별도 배터리 HID를 사용하므로 Link 웹 UI가 없어도 동작하도록 구현했다. Link 응답 슬롯의 실제 의미 확인과 웹 UI 연동은 이번 구현에 포함하지 않았다.

## 실제 실행한 검사

- Rust host 테스트: ANSI 112, ISO 112, JIS 112, KR 119개 통과. 각 검사 0 failed/0 ignored.
- 네 배열 host clippy와 역할별 ARM release clippy/build 통과. 좌우 8개+공용 동글 1개 UF2 생성, application vector·보호 영역·바이너리 round-trip 검증 통과.
- Python unittest 36개 통과. 그 안의 실제 Rust codec 실행은 공통 JSON golden vectors와 codec 테스트 5개를 검증한다.
- Windows .NET 8 빌드: 경고 0/오류 0. 공통 vector와 신선도·순서·단절·장치 통합 self-test 통과. x64/ARM64 자체 포함 패키지 publish 통과; x64 네이티브 프로세스 5초 기동 검사 통과 후 종료. ARM64 실행은 미검증.
- macOS Info.plist 구조와 Git Bash의 build-app.sh 문법 검사 통과. **Swift 컴파일·Swift 테스트·Mac 앱 실행은 수행하지 못했다.** Mac/Xcode가 없어 소스 검토만 했다.
- 실제 Windows/macOS USB·Bluetooth·동글 잔량 표시, OS 기본 BAS 표시, 입력/절전/전류/DFU 회귀는 아직 수행하지 않았다.

재현 명령(저장소 루트):

```powershell
python -B tools/build_battery_preview.py
python -B tools/verify_battery_preview.py
dotnet run --project desktop/windows/NocFreeBattery.csproj -c Release -- --self-test
```

전체 로그: `target/battery-preview-build.log`, `target/battery-windows-publish.log`, `target/battery-windows-arm64-publish.log`. 생성물이 없는 새 checkout에서는 위 명령으로 다시 만든다. Mac에서는 `desktop/macos/README.md`에 따라 `swift test`와 `bash build-app.sh`를 실행해야 한다.

## 실험용 파일과 SHA-256

모든 UF2 위치는 `target/battery-preview/`이며, 아래 해시는 현재 실험 빌드용이다. `manifest.json`에는 입력 소스 파일별 해시도 저장된다. 새 빌드 시 해시는 달라질 수 있으므로 실기 기록에는 실제 사용 파일의 해시를 남긴다. 공개 배포판·실기 검증판으로 간주하지 않는다.

| 파일 | SHA-256 |
|---|---|
| `NocFree_Battery_Experimental_ANSI_Left.uf2` | `6eb9c2cf5aba934d9d7a5041b85fa2b18a9c9ef5bd271a7a6ad8c0f973cd7b07` |
| `NocFree_Battery_Experimental_ANSI_Right.uf2` | `0a4afcb47a7a0a9e40e1255d19c6219a65bad4602939b3489380b757de257f68` |
| `NocFree_Battery_Experimental_ISO_Left.uf2` | `fd09270136345a1dc10616b1bdf9937043cb34a63bddb5c2e50f307aa4da78f5` |
| `NocFree_Battery_Experimental_ISO_Right.uf2` | `9cd5062fe0df5e869b2019ca499cceb90a9c894a1207ba67a3fd62de0810e24d` |
| `NocFree_Battery_Experimental_JIS_Left.uf2` | `90bc4a40f4dc097dfa353b87416832ec2af12f582948c1d9ed87609dc87f963f` |
| `NocFree_Battery_Experimental_JIS_Right.uf2` | `bf855b0006ec4d920a9b03d519b47ce26ecbdc4df3275fd3bf7c0e70adb9c956` |
| `NocFree_Battery_Experimental_Universal_Dongle.uf2` | `35baf830b0cda60bf28564989cec9726ef8510b6e638e6ec84db22f567f2023c` |
| `NocFree_Battery_Experimental_KR_Left.uf2` | `2989b33cf16e4a671c47efde344c23bad6597d3fdae4eb46fdc3a5156df3d2b5` |
| `NocFree_Battery_Experimental_KR_Right.uf2` | `893fbefa00dacbc7b34faae382e4f01b9ed6a29bbade9817336a3a5897e7ac5c` |

Windows 앱은 `target/battery-preview/windows-x64/` 또는 `windows-arm64/` 폴더 전체가 필요하다. 아래는 진입 EXE와 실제 관리 코드 DLL의 해시이며 EXE 하나만 복사해서 실행하지 않는다.

| 파일 | SHA-256 |
|---|---|
| `windows-x64/NocFreeBattery.exe` | `1c24d001a300bda2cd0cf058a7d92eaa09f5e8be68f42d5d56c90333de19cd3a` |
| `windows-x64/NocFreeBattery.dll` | `411a12fd50d4faac8e8b2bede7ad82e25d4a863b41cdf84abae3250459c02453` |
| `windows-arm64/NocFreeBattery.exe` | `39be8dc3904bc2885e5049f65bdab724d8bcedc5d1e9cee4f8ee5abf37551013` |
| `windows-arm64/NocFreeBattery.dll` | `ed582fe5b4f87993e232ddc37e724991db0c70122010d4a229de6a555f5cda43` |

## 단계별 실제 장치 검사 — 모두 미실행

실기 배포는 사용자 지시를 받은 뒤에만 진행한다. ANSI부터 시작하고 해당 단계가 통과하기 전 다른 배열이나 공개 배포로 진행하지 않는다. ISO/JIS/KR은 해당 실물이 필요하다.

1. 사용 배열, 좌/우/동글 UF2 파일명·SHA-256, 기존 펌웨어, OS 버전, 앱 실행 파일 해시를 기록한다. 기존 복구 자료/파일과 독립 DFU 경로를 확보한다.
2. 왼쪽/오른쪽의 대응 Experimental 파일을 한쪽씩 적용한 뒤 유선으로 양손 빠른 입력·미디어·포인터·Fn+I·백라이트·Link 기존 설정 동작을 확인한다. USB 배터리 collection FF60/1, Feature ID1/길이21, 좌우 잔량을 검사한다.
3. Bluetooth를 Windows/macOS 각각 정상 페어링한다. GATT 구조가 추가됐으므로 기존 서비스 캐시/CCCD가 맞지 않으면 OS에서 장치를 제거하고 해당 프로필을 다시 페어링한 뒤 재검사한다. 앱이 자동 bond 삭제를 수행하지 않는다. 암호화 read와 Windows BAS 대표값, Mac 기본 UI 표시 여부를 기록한다.
4. 공용 Experimental Dongle 파일로 동글 연결을 검사한다. 새/구 왼쪽 × 새/구 동글 네 조합에서 키 입력을 먼저 확인하고, 지원하지 않는 배터리만 unavailable인지 확인한다. 동글만 꽂힘/다른 키보드 재페어링 때 이전 키보드 값이 사라져야 한다.
5. 두 OS × 세 연결에서 연결·측정 완료 후 앱 시작 5초 목표, cold/reconnect 새 측정 10초 목표를 기록한다. 새 ADC 측정 이후 앱 반영은 35초 목표다. 무선 연결 끊김은 supervision timeout/다음 조회 지연을 포함해 기록하고, 오래된 값은 age 180초 초과 시 지연으로 표시되어야 한다.
6. 오른쪽 OFF/ON, 입력 모드/호스트 프로필 전환, 호스트 sleep/wake, 케이블만 전원 공급, 두 연결 동시 연결, 서로 다른 키보드 두 개를 검사한다. 0%/알 수 없음/지연/연결 끊김 구분, 중복 항목 없음, wake 전 값 재사용 없음이 필요하다.
7. 앱의 Bluetooth 권한 거부·재허용 중 USB/동글 동작, 메뉴 열기·새로 고침·로그인 실행 켜기/끄기·종료를 검사한다. Mac은 Intel/Apple Silicon에서 빌드·실행하고 서명/notarization 및 설치/제거를 별도로 검증한다.
8. 앱 유무의 idle/System OFF 전류와 wake/재연결을 비교한다. 오른쪽 배터리 전달은 수신 시각 기준이며 실제 ADC 시각과의 차이를 확인한다. 양쪽 분리 DFU와 동글 복구를 재검사한다. 합격 증거 전에는 배터리 수명 중립 또는 6개 경로 실기 지원 완료로 표시하지 않는다.

## 남은 출시 조건

Mac/Xcode 빌드와 실행, 실제 six-path 검증, SoftDevice GATT table/RAM·저장된 system attributes 용량/캐시 호환, USB endpoint 열거가 필수다. 현재 빌드 통과만으로 런타임 GATT 등록을 보장하지 않는다. BAS 값은 방전 감소 필터 때문에 앱의 전압 기반 추정값보다 낮을 수 있다. 오른쪽 충전 감지 신호를 확인하지 못했으므로 충전 상태·충전 완료·정확한 배터리 % 보정은 지원 완료로 표시하지 않는다.
