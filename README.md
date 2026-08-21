# NocFree-and-rust

NocFree & ANSI 키보드의 ZMK 동작을 nRF52833용 `no_std` Rust 펌웨어로
옮긴 프로젝트입니다. 기준 소스는
[`jhkim0218/NocFree-and-zmk`](https://github.com/jhkim0218/NocFree-and-zmk.git)의
커밋 `e5e2f470795e92609f7ee6e810470fa6976557d1`입니다.

> 비공식 펌웨어입니다. 자동 테스트와 ELF/UF2 범위 검증, USB 전체 84키,
> BLE 좌우 교차 입력, 양쪽 DFU 및 역할별 순정 원복 후 Rust 재설치를 실제로
> 통과했습니다. 플래시는 반드시 [복구 절차](RECOVERY.md)의 순서를 따릅니다.

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

Windows PowerShell에서 실행합니다. Rust와 Python 3가 필요합니다.

```powershell
cd D:\study\nocfree\NocFree-and-rust
.\tools\build-release.ps1
```

스크립트는 다음을 한 번에 수행하며 어느 단계든 실패하면 즉시 중단합니다.

1. Rust 포맷 검사
2. Windows 호스트 단위 테스트
3. `central`과 `right` release 빌드
4. ELF에서 앱 바이너리 추출
5. nRF52833 UF2 생성 및 주소·family ID·벡터·왕복 검증

결과 파일은 `firmware/NocFree_Rust_Left.uf2`와
`firmware/NocFree_Rust_Right.uf2`입니다. 두 UF2는 앱 영역
`0x27000..0x64fff`만 기록합니다.

## 실기 검증 상태

2026-08-21, Windows에서 현재 정확한 이미지로 확인한 상태입니다.

| 항목 | 상태 |
|---|---|
| 왼쪽 UF2 설치 및 부팅 | 통과; `RUST-LEFT`, 키보드/미디어 HID, CDC 열거 |
| 오른쪽 UF2 설치 및 부팅 | 통과; `RUST-RIGHT`, CDC만 열거하고 호스트 HID 없음 |
| UF2 부트로더 정보 | 양쪽 모두 NocFree & / S140 7.3.0 확인 |
| 현재 이미지의 USB 전체 84키 | 통과; 문자 50개와 비문자 34개, 양쪽 Fn 확인 |
| 현재 이미지의 오른쪽 split → 왼쪽 USB 입력 | 통과; `yuiopY`, `jamjamjamjam` 및 교차 반복 입력 순서 확인 |
| BLE 입력과 USB/BLE 전환 | 통과; `blejamjamjam`, `finalusbjamfinalblejamfinalusbback` 확인 |
| 미디어 키 | 통과; 음소거/해제, 볼륨 내림/올림 확인 |
| 현재 이미지의 양쪽 DFU 및 역할별 순정 원복 | 통과; 양쪽 순정 ID 확인 후 같은 Rust 이미지로 복귀 |
| `Fn+Esc` 왼쪽 reset / `Fn+Delete` 오른쪽 DFU | 통과; 재시작 및 UF2 재설치 후 입력 확인 |

현재 UF2 SHA-256은 다음과 같습니다.

- 왼쪽: `7053E74EA87313A106AE15C02724816A0A69D6D4F3E6B722D852DDEA6A725992`
- 오른쪽: `480A9DB7ECC6146CCB54E37FB6F586AEEC48D5DD39EDF2F4FA2E91BACC0F38BA`

다른 작업자나 AI가 현재 상태부터 이어갈 때는 [HANDOFF.md](HANDOFF.md)를 먼저
읽으십시오. USB 전체 84키는 통과했지만 BLE에서 동일한 84키 전체 sweep을
반복한 것은 아니며, BLE는 좌우 교차 입력과 출력 전환으로 검증했습니다.

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
