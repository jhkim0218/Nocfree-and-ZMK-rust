# 물리 배열 변형

[English](LAYOUTS.md) · [한국어](LAYOUTS_ko.md) · [日本語](LAYOUTS_ja.md)

펌웨어는 컴파일할 때 ANSI, ISO, JIS, KR 중 정확히 하나의 물리 배열을
선택합니다. ANSI가 기본값이며 유지보수자의 키보드에서 검증된 유일한 배열입니다.

| 배열 | 코드 | 왼쪽 | 오른쪽 | 근거 | 상태 |
|---|---|---:|---:|---|---|
| ANSI | `src/keymap/ansi.rs` | 37 | 47 | 원본 커뮤니티 ZMK와 로컬 실물 | 안정 후보·실기 검증 이력 있음 |
| ISO | `src/keymap/iso.rs` | 38 | 47 | 공식 updater ISO 이미지 | Experimental·실물 미검증 |
| JIS | `src/keymap/jis.rs` | 37 | 48 | `electricdoc187`의 `jis-custom` 스캔맵 | Experimental·Rust 실물 미검증 |
| KR | `src/keymap/kr.rs` | 39 | 50 | 공식 updater/product 자료와 `0x21/P0` 추가 포트 | Experimental·실물 미검증 |

모든 배열은 지원되는 공용 Rust 2.4G Dongle UF2를 사용합니다. ANSI는 2.4G 실기
검증을 통과했고, ISO/JIS/KR은 반드시 대응 실물 키보드에서 테스트해야 합니다.

공통 동작은 `src/keymap.rs`, 물리 개수·좌표 변환·HID usage·Fn 위치·PCA9555
주소는 각 배열 파일에 있습니다. 선택한 배열 정보는 scanner, HID report,
NocFree Link 저장, USB 제품명과 패키징에 전달됩니다. 저장된 Link 키맵에는 version,
layout ID, key count, CRC가 들어가므로 다른 배열의 기록을 잘못 적용하지 않습니다.
version 3 기록은 배열 ID가 없어 version 4 첫 부팅에서 기본 키맵으로 돌아갑니다.
BLE host bond와 split bond는 별도 기록이므로 이 초기화 대상이 아닙니다.

## 빌드

Windows, macOS, Linux에서 다음 중 하나를 실행합니다.

```text
python3 -B tools/build_release.py --layout ANSI
python3 -B tools/build_release.py --layout ISO
python3 -B tools/build_release.py --layout JIS
python3 -B tools/build_release.py --layout KR
```

필요하면 `python3` 대신 `python`을 사용합니다. Windows PowerShell의 기존
`tools/build-release.ps1 -Layout ANSI`도 같은 Python 빌더를 호출하는 래퍼입니다.
ANSI 결과는 `firmware`, ISO/JIS/KR은 `firmware/experimental`에 생성됩니다.
반드시 같은 빌드·같은 배열의 Left/Right 한 쌍만 플래시하십시오.

## 제한 사항

- ISO, JIS, KR은 컴파일·합성 입력·아티팩트 검사를 통과해도 실물 검증이 아닙니다.
- JIS의 0x87/0x89/0x8a HID usage와 48번째 오른쪽 키를 보고할 수 있도록 report와
  디바운서 크기를 확장했습니다. Rust 실물 키보드 검증은 아직 없습니다.
- 참고 ZMK의 왼쪽 Eisu 키는 tap Muhenkan / hold Fn입니다. 초기 Rust JIS는
  테스터가 없으므로 Fn만 유지하고 tap/hold는 보류합니다.
- 호스트의 키보드 배열과 IME는 OS 설정입니다. 펌웨어가 자동 설치·전환하지 않습니다.
- 서로 다른 물리 배열에 Experimental 이미지를 올려 시험하면 안 됩니다.

## 다음 실물 순서

먼저 ANSI 새 한 쌍을 오른쪽, 왼쪽 순으로 설치해 84키, 양쪽 Fn, Wired USB,
Windows 11 BLE 재연결, 백라이트 동기화와 새 밝기 곡선, `Fn+5`/`Fn+0` 복구,
NocFree Link 변경·재부팅 저장을 확인합니다. ANSI 회귀가 끝난 뒤 해당 실물을 가진
지원자가 ISO, JIS, KR을 각각 시험해야 합니다. 이 세 배열은 먼저 Left만 플래시해
Wired 입력을 확인한 뒤 공용 동글을 페어링하십시오.

매핑 근거는 원본 <https://github.com/NocFreeKB/NocFree-and-zmk>, JIS 참고
<https://github.com/electricdoc187/NocFree-and-zmk/tree/jis-custom>, 공식 제품 페이지
<https://www.nocfree.com/products/nocfree-and-reservation>입니다.
