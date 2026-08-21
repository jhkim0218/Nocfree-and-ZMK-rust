# NocFree-and-rust

NocFree & ANSI 키보드의 ZMK 동작을 nRF52833용 `no_std` Rust 펌웨어로
옮긴 프로젝트입니다. 기준 소스는
[`jhkim0218/NocFree-and-zmk`](https://github.com/jhkim0218/NocFree-and-zmk.git)의
커밋 `e5e2f470795e92609f7ee6e810470fa6976557d1`입니다.

> 비공식 펌웨어입니다. 이전 체크포인트는 USB 전체 84키, BLE 좌우 교차 입력,
> 양쪽 DFU 및 역할별 순정 원복을 실제로 통과했습니다. 현재 작업 중인 Link
> 호환 이미지는 자동 검증만 완료됐고 아직 실기에 설치하지 못했습니다. 현재
> 왼쪽 복구 상태는 [복구 절차](RECOVERY.md)를 먼저 확인하십시오.

## 왼쪽과 오른쪽

역할을 혼동하면 안 됩니다.

- **왼쪽 (`central`)**: 왼쪽 37키 스캔, 오른쪽 상태 수신, 전체 84키 키맵,
  USB HID, BLE HID, 5개 BLE 프로필을 담당합니다.
- **오른쪽 (`right`)**: 오른쪽 47키 스캔 후 BLE split으로 왼쪽에 전달합니다.
  USB 연결은 입력용 HID가 아니라 CDC 복구용입니다.

두 펌웨어 모두 PCA9555 세 개(`0x20`, `0x22`, `0x24`)를 100 kHz I2C로
읽고, active-low 입력과 5 ms press/release debounce를 사용합니다. BLE는
1M PHY만 허용합니다. 왼쪽과 오른쪽의 split 링크는 별도 Just Works bond로
암호화하며, 호스트용 5개 BLE 프로필과 다른 flash page에 저장합니다.

## 빌드

Windows PowerShell에서 실행합니다. Rust, Python 3와 application-only serial
DFU 패키지를 만드는 `adafruit-nrfutil` 0.5.3.post16이 필요합니다.

```powershell
cd D:\study\nocfree\NocFree-and-rust
.\tools\build-release.ps1
```

스크립트는 다음을 한 번에 수행하며 어느 단계든 실패하면 즉시 중단합니다.

1. Rust 포맷 검사
2. Windows 호스트 단위 테스트
3. `central`과 `right` release 빌드
4. ELF에서 앱 바이너리 추출
5. nRF52833 UF2와 serial-DFU ZIP 생성
6. 주소·family ID·벡터·왕복 및 DFU ZIP/BIN 일치 검증

결과 파일은 역할별 `.bin`, `.uf2`, `_DFU.zip`입니다. UF2와 serial-DFU ZIP은
모두 앱 영역 `0x27000..0x64fff`만 기록합니다.

## 실기 검증 상태

2026-08-21 현재 상태입니다. 이전 체크포인트의 실기 결과와 최신 미설치 이미지의
상태를 혼동하면 안 됩니다.

| 항목 | 상태 |
|---|---|
| 이전 체크포인트 USB/BLE/84키/양쪽 DFU | 통과; 자세한 입력 문자열과 범위는 [HANDOFF.md](HANDOFF.md) 참조 |
| 최신 Link 호환 이미지 자동 검증 | 통과; 43 Rust 테스트, 14 Python 테스트, 양쪽 clippy/release 빌드 |
| 최신 Link 호환 이미지 실기 설치 | 미실시 |
| 현재 왼쪽 | USB/CDC/UF2 모두 없음; 중간 이미지가 USB 생성 중 panic |
| 현재 오른쪽 | 이전 `RUST-RIGHT`, COM18, 정상 열거 |
| 현재 UF2 드라이브 | 없음 |
| 최신 이미지의 Link 웹앱/키 변경 | 구현됨, 실기 미검증 |

현재 UF2 SHA-256은 다음과 같습니다.

- 왼쪽: `368CDD50F457680561CD0D0360F092A756675472F5A2297A9531B7DF734B7707`
- 오른쪽: `A75EEAAD39B0B10ACF4B38B61EEBA361B44FE9E39A13EB8DB29423547F472E3C`

다른 작업자나 AI가 현재 상태부터 이어갈 때는 [HANDOFF.md](HANDOFF.md)를 먼저
읽으십시오. USB 전체 84키는 통과했지만 BLE에서 동일한 84키 전체 sweep을
반복한 것은 아니며, BLE는 좌우 교차 입력과 출력 전환으로 검증했습니다.

## NocFree Link 키 변경

최신 왼쪽 이미지는 `link.nocfree.com`이 선택하는 VID/PID `2886:8029`, 제품명
`NocFree & ANSI`, WinUSB vendor bulk 인터페이스를 제공합니다. 8개 레이어의
84개 물리 키와 16개 hotkey 슬롯을 flash에 저장하며 키 변경 결과가 실제 HID
입력에 사용됩니다. 빠른 문자열(quick text)은 조회 시 빈 슬롯으로 응답하지만
저장·실행은 아직 구현하지 않았습니다. ZMK Studio 프로토콜은 구현하지 않았고
두 선택지 중 NocFree Link 호환 경로를 선택했습니다. 이 항목은 왼쪽을 복구한
뒤 Chrome/Edge에서 실제 연결·키 변경·재부팅 후 보존까지 확인해야 완료입니다.

## 키 바인딩

- `Fn+1` .. `Fn+5`: BLE 프로필 1..5 선택
- `Fn+0`: 현재 BLE 프로필 삭제
- `Fn+U`: USB 출력 강제 선택
- `Fn+B`: BLE 출력 강제 선택
- `Fn+F10/F11/F12`: 음소거/볼륨 내림/볼륨 올림
- `Fn+Esc`: **왼쪽 애플리케이션 재시작**; DFU가 아님
- `Fn+Delete`: split 연결을 통해 **오른쪽** UF2 부트로더 진입

왼쪽 DFU와 split이 끊긴 오른쪽 DFU는 각 반쪽의 USB CDC 1200-baud
touch를 사용합니다. 자세한 안전 조건과 원복 순서는 [RECOVERY.md](RECOVERY.md)에
있습니다.

최신 이미지에서는 처리되지 않은 panic도 GPREGRET `0x57`을 설정하고 UF2로
재시작합니다. 현재 왼쪽에 설치된 중간 이미지는 이 안전장치가 들어가기 전
이미지이므로 이 설명을 현재 장치의 복구 경로로 오해하면 안 됩니다.

## 코드 읽는 순서

1. `src/keymap.rs`: 물리 입력 84개와 키 동작
2. `src/scanner.rs`, `src/pca9555.rs`: 입력 디코딩과 debounce
3. `src/report.rs`: USB/BLE NKRO 및 명령 생성
4. `src/bin/right.rs`, `src/split_ble.rs`: 오른쪽 split 주변장치
5. `src/bin/central.rs`: 왼쪽 USB/BLE/split 통합
6. `src/platform.rs`, `src/bond_store.rs`: DFU와 BLE 프로필 저장

`vendor/nrf-softdevice`와 `vendor/nrf-softdevice-macro`는 upstream 커밋
`b0ac850c0a5a05b8a5aef4f752b48115755b8542`의 로컬 사본입니다. 변경점은
연결 후 PHY 변경 요청에도 1M만 응답하는 부분과, 보안 characteristic의
CCCD에도 같은 보안을 적용하는 부분입니다. 각 폴더의
`README.nocfree.md`에 출처와 정확한 변경 이유를 기록했습니다.
