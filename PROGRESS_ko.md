# NocFree Rust 진행 기록

[English](PROGRESS.md) · [한국어](PROGRESS_ko.md) · [日本語](PROGRESS_ja.md)

이 문서는 긴 실험 로그의 현재 판정을 한국어로 요약합니다. 세부 시간·RSSI·과거
해시는 [PROGRESS.md](PROGRESS.md), 다음 우선순위는 [ROADMAP_ko.md](ROADMAP_ko.md),
복구 절차는 [RECOVERY_ko.md](RECOVERY_ko.md)를 기준으로 합니다.

## 검증 기준선

- 저장소: `https://github.com/jhkim0218/Nocfree-and-ZMK-rust.git`
- 원본: `https://github.com/NocFreeKB/NocFree-and-zmk`
- 기본 배열: ANSI 37+47키. 모든 배열에서 공용 Rust 2.4G 동글을 지원하지만 ISO/JIS/KR은 실물 미검증
- 왼쪽만 USB/BLE host HID이며 오른쪽은 암호화 split으로 입력 전달
- 앱 영역 `0x27000..0x64fff`, 양쪽 독립 1200-baud 복구와 순정 왕복 보존
- BLE host 실기 범위는 Windows 11과 Android뿐임

## 완료된 실기 범위

과거 matching ANSI 이미지에서 USB, BLE, 전체 84키, 양쪽 Fn, 비문자 키, 물리
Wired/Bluetooth/2.4G 위치, 오른쪽 전원 스위치, BLE profile 1/2, Windows↔Android
멀티페어링, 양쪽 DFU, 순정 V2.3.0 왕복, NocFree Link 키 변경·재부팅 저장을
확인했습니다. 오른쪽 USB는 HID가 아니라 독립 CDC 복구이며 정상입니다.

배터리 계산은 순정 V2.3.0의 divider/SAADC/단계식과 3:1 filter를 복원했습니다.
완충한 양쪽에서 `Fn+I` 3초로 `L 100 R 100`을 확인했습니다. 전체 방전과 실제
소비전류는 미검증입니다. 파란 pairing 점멸은 profile long-hold 뒤 지속되고 bonded
profile을 선택하면 꺼졌습니다. 빨간 10% 이하 점멸은 배터리를 낮추지 못해 미검증입니다.

백라이트 절대 상태 동기화, 오른쪽 재부팅 뒤 수렴, 10회 toggle, 오른쪽 키 wake,
30초 양쪽 소등이 과거 이미지에서 통과했습니다. USB 전원 여부와 무관한 소등 경로도
검증했습니다.

## 동글 D0 복구

2026-08-27 순정 동글은 `NocFree_Dongle`, `VID_2886&PID_8029`, serial
`E19D2CEA0B437049`, COM6, HID `MI_02`로 확인됐습니다. 1200-baud touch 뒤
`VID_239A&PID_002A`, `NocFree &`, `nRF Serial`, COM14인 serial DFU로 진입했고,
공식 V2.3.1 application-only 패키지 전송 뒤 원래 USB identity와 interface가 모두
복구됐습니다. 이 D0 절차는 순정 패키지만 올린 것이므로 현재 Rust 키보드와의 순정 ESB
호환을 뜻하지 않습니다. 양쪽이 Rust라 순정 ESB 입력은 이번 D0 범위에서 확인하지 못했습니다.

## ANSI Rust 2.4G 동글 — 실기 검증 완료

ANSI 왼쪽/오른쪽과 동글은 custom BLE 링크로 연결되어 keyboard와 consumer HID report를
전송합니다. 동글 재연결, 빠른 양쪽 입력과 modifier, BLE→2.4G 복귀, 각 장치의
1200-baud 복구 왕복까지 통과했습니다. ISO/JIS/KR 동글 이미지는 빌드되지만 해당 배열
실기 검증은 남아 있습니다. ANSI는 2.4G에서 6분 이상 idle 뒤에도 동글을 다시 꽂지 않고
재연결되는 것을 확인했습니다.

## Split P3

초기 reconnect 실패는 불필요한 ATT MTU 교환 단계에서 포착됐습니다. P3.1은 표준
MTU 23을 사용해 해당 단계를 제거했습니다. 약 30cm에서 전원 재인가 뒤 입력이
55.431초와 17.736초에 복구됐고, 왼쪽 복구 뒤에는 4.358초에 split-ready였습니다.

P3.2는 미연결 오른쪽 advertising을 10초간 250 ms, 다음 50초간 500 ms, 이후
1초로 낮추고, 미연결 키 입력 시 250 ms로 되돌립니다. 짧은 진단과 최종 설정이
단계 전환·키 reset·Wired/BLE 입력·오른쪽 복구를 통과했습니다. 낮은 RSSI에서
보안 timeout과 체감 지연이 남았습니다.

P3.3은 nRF52833의 허용 최대인 +8 dBm을 양쪽 split 연결에 적용했습니다. 입력은
통과했지만 거리, reconnect, 보안 성공률, 소비전류를 같은 조건에서 비교하지 않아
최적화 완료로 보지 않습니다.

## 좌우 입력 순서 P4

`jam -> ajm`, `삼 -> ㅅ마`처럼 빠른 좌우 교차 입력 문제를 위해 source timestamp,
오른쪽 sequence, split clock sync, reconcile, 하나의 전역 queue를 구현했습니다.
1~5 ms와 10,000 합성 이벤트에서 1/2 ms는 실패하고 3/4/5 ms는 clean이었습니다.
이전 3 ms 이미지는 Wired `jam` 반복과 Windows 11 BLE 입력을 통과했습니다.

이후 여유를 위해 8 ms 후보가 만들어졌고, 현재 코드는 지연과 jitter 여유의 절충인
5 ms입니다. 현재 `REORDER_WINDOW_MS=5`는 자동 테스트만 통과했으며 실물 미검증입니다.
실제 queue 전체, 장시간 drift, reconnect 직후, Android에서 loss/duplicate/reorder/stuck
0을 확인해야 합니다.

## 현재 변경 후보

- `Fn+F5`가 밝기 down, `Fn+F6`가 up임을 코드·테스트로 고정
- 양쪽 PWM 10 kHz 공통 상수 유지
- 20% 설정이 위쪽에서 같아 보이던 문제를 해결하도록 0/20/40/60/80/100%에
  체감 보정 이차 duty curve 적용; 실기 검증 필요
- JIS 0x87/0x89/0x8a usage만 19-byte HID report로 확장하고 ANSI/ISO/KR은 기존
  16-byte 유지
- JIS 48키·KR 50키 오른쪽을 수용하도록 debouncer를 선택 배열 최대치로 확장
- Windows/macOS/Linux 공용 `tools/build_release.py`; PowerShell은 호환 래퍼

## 다음 판정 순서

1. 네 배열 host/ARM Clippy, 양쪽 release build, UF2/DFU와 계약 테스트를 모두 통과.
2. 새 ANSI matching pair를 오른쪽, 왼쪽 순으로 설치하고 각각 복구 가능 확인.
3. Wired에서 84키, 양쪽 Fn, `jam`/`삼`, 백라이트 여섯 단계, timeout/wake 확인.
4. Windows 11 BLE에서 profile 전환, Wired 왕복, 자동 reconnect와 같은 입력 확인.
5. Android 기존 bond 입력 확인.
6. `Fn+5`/`Fn+0` 짧은 입력 무동작과 3초 역할별 DFU를 확인.
7. 그 뒤에만 문서·산출물 해시를 확정하고 commit/push.

Quick Text, 추가 조명 효과, 전원 설정 UI, firmware updater, ZMK Studio는 현재
필수 범위가 아닙니다. ANSI USB dongle 2.4G 입력 경로는 실기 검증까지 완료됐습니다.
