# KR 실물 매핑 코드 변경 내역

## 문서 목적

이 문서는 실물 매핑 확인 과정에서 작업 브랜치에 적용한 구현 내용을 별도로 기록한다. 원 저장소 유지관리자가 수정 방향과 적용 범위를 검토할 때 참고하기 위한 문서이며, 실물 매핑 결과 자체는 `KR_HARDWARE_MAPPING_REPORT_ko.md`에 정리되어 있다.

## Git 기준

- 기준 커밋: `cd594541b27d173c4eedc6e994e0e5aa08184802`
- 작업 브랜치: `fix/kr-hardware-mapping`
- 코드 커밋: `349e242 fix(kr): align scan mapping with hardware`

## 로컬 구현 내용

### `src/keymap/kr.rs`

- `LEFT_ROW_COUNTS`: `[7,7,7,7,6,5]`
- `RIGHT_ROW_COUNTS`: `[8,8,8,7,8,7]`
- `EXTRA_LEFT_KEYS`: `0`
- `EXTRA_RIGHT_KEYS`: `4`
- `LEFT_FN_RAW`: `34`
- `RIGHT_FN_RAW`: `80`
- Left/Right `VISUAL_TO_RAW`를 실측 순서로 변경
- 기존 visual 위치의 HID 동작을 새 physical raw에 연결하는 변환 추가

### `src/keymap.rs`, `src/scanner.rs`

- 오른쪽 여섯 행의 전체 raw 순서 테스트 추가
- `0x24 P1.7` 제외와 `0x21 P0.0~P0.3` 포함 테스트 추가
- 왼쪽 `Y/H`가 `0x22 P0.6/P1.6`에 있는지 테스트 추가
- Fn 위치와 KR 경계 중복키 동작 테스트 추가

### `src/link_keymap.rs`

- KR 저장 키맵 레코드 버전을 `6`으로 변경
- 이전 KR 저장 키맵이 새 기본 매핑을 덮어쓰지 않도록 무효화
- 다른 배열의 레코드 버전은 `4` 유지

왼쪽이 키맵의 권위 장치이므로 오른쪽 매핑 변경도 Left/Right를 같은 소스로 빌드해 함께 적용했다.

### 하드웨어 진단

- `src/pca9555.rs`
  - 설정에 성공한 주소 마스크 추적
  - 읽기에 성공한 입력 포트 마스크 추적
- `src/hardware_scanner.rs`
  - 확장기 설정 재시도 결과 기록
  - 입력 가능 포트 변화 기록
- `src/split_diagnostics.rs`
  - 이벤트 19 `KeyScanConfigured`
  - 이벤트 20 `KeyScanInputs`
  - 이벤트 21 `BacklightPwm`
  - 네 16비트 입력값의 64비트 packing 함수
- `src/bin/central.rs`, `src/bin/right.rs`
  - 하드웨어 스캐너에 진단 객체 전달
  - 왼쪽 PWM 초기값 기록
- `tools/test_repository_contract.py`
  - `loop/match` 기반 초기화 재시도 구조에 맞게 계약 테스트 갱신

물리 키를 매핑하기 위해 사용한 per-key raw 진단 이벤트는 최종 코드에서 제거했다. 부팅 시 확장기와 입력 포트 상태를 확인하는 이벤트만 유지했다.

## 자동 검증

- KR host unit tests: `86 passed`
- documentation tests: `11 passed`
- repository contract tests: `15 passed`
- host Clippy: 통과, warnings denied
- ARM Left `central` Clippy/build: 통과
- ARM Right `right` Clippy/build: 통과
- UF2 tests: `6 passed`

## 로컬 산출물

| 파일 | SHA-256 |
|---|---|
| `firmware/experimental/NocFree_And_Rust_ZMK_Based_KR_Experimental_Left.uf2` | `6aa02207dd6d26096c97496941efe9532bc22854648568caac9c5a36038a42e9` |
| `firmware/experimental/NocFree_And_Rust_ZMK_Based_KR_Experimental_Right.uf2` | `5d04e6008aa4b2dc2d9ed68eacae5dad9aa11050611e7963f3f0d60d15bfa0a5` |
| `firmware/experimental/NocFree_And_Rust_ZMK_Based_KR_Experimental_Left_DFU.zip` | `73478004d766f6233909c8e6d06db185fe27ffbd4c6fb95e83100e20e1c66743` |
| `firmware/experimental/NocFree_And_Rust_ZMK_Based_KR_Experimental_Right_DFU.zip` | `00b9c311c284ced9af5fe39177068ad342f76fe04d78fc637069552d367840d0` |

DFU ZIP은 application-only 패키지이며 내부 BIN 해시와 빌드 BIN 해시가 일치한다. BIN과 DFU ZIP은 `.gitignore` 대상이고, 두 UF2만 추적된다.

## 구현상 남은 차이

현재 Rust 펌웨어의 macOS/Windows 설정은 `Fn+F3/F4` 동작만 바꾸며 modifier 물리 배열 전체를 전환하지 않는다.

- Rust macOS: Mission Control, `Cmd+Space`
- Rust Windows: `Win+Tab`, `Win+S`
- Rust 전환: `Fn+M`/`Fn+N` 1초 홀드

순정 공식 자료에는 `Fn+M`/`Fn+N` 3초 홀드가 적힌 안내와, 유선 NocFree Link에서 시스템 레이아웃을 변경하라는 최신 안내가 함께 존재한다. 따라서 이 부분은 원 저장소 유지관리자가 목표 호환 범위를 다시 결정해야 한다.

## 안전 범위

- application-only UF2/DFU만 사용
- MBR, SoftDevice, bootloader, factory filesystem, 보호 flash, UICR 변경 없음
- 동글 변경 없음
