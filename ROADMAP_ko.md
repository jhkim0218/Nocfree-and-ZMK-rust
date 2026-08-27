# 순정 기능 비교와 로드맵

[English](ROADMAP.md) · [한국어](ROADMAP_ko.md) · [日本語](ROADMAP_ja.md)

이 문서는 로컬 NocFree & ANSI V2.3.0 UF2, 공식 NocFree 문서와 현재 Rust
firmware를 비교합니다. 바이너리 크기 차이만으로 기능 존재를 단정할 수 없으므로,
아래 기능 판정은 문서 또는 실기에서 확인된 동작을 기준으로 합니다.

## 비교 기준

모든 이미지는 nRF52833 family ID `0x621E937A`를 사용하고 application 주소
`0x27000`부터 시작합니다.

| V2.3.0 이미지 | UF2 크기(bytes) | 기록 범위 |
|---|---:|---|
| 순정 왼쪽 ANSI | 295,936 | `0x27000..0x4B1FF` |
| 순정 오른쪽 ANSI | 162,304 | `0x27000..0x3ACFF` |
| 순정 2.4 GHz 동글 | 143,872 | `0x27000..0x388FF` |
| Rust 왼쪽 ANSI | 162,304 | `0x27000..0x3ACFF` |
| Rust 오른쪽 ANSI | 95,232 | `0x27000..0x329FF` |
| Rust 동글 D1 | 28,672 | `0x27000..0x2A7FF` |

순정에 별도 동글 이미지가 있다는 점은 공장 2.4 GHz 기능이 세 firmware로 구성된
시스템임을 보여줍니다. 왼쪽과 오른쪽 Rust 이미지만 바꿔서는 복원할 수 없습니다.

## 추가할 기능 목록

| 우선순위 | 영역 | 순정 동작 / 목표 | 현재 Rust 상태 | 완료 판정 |
|---|---|---|---|---|
| 완료 | 오른쪽 `P0.05` 핀 역할 | active-low PCA9555 `INT` | 잘못된 출력을 제거하고 현재 오른쪽 스캐너 wake에 사용 | 핀 변경 후 키 스캔과 warm recovery 통과 |
| P0 | 배터리 정확도 | 양쪽 1100 mAh 배터리의 의미 있는 잔량 | 순정 V2.3.0 환산과 75/25 필터 복구; 완충된 양쪽에서 100% 확인 | 방전 구간별 DMM 전압 오차와 퍼센트 오차 기록 |
| P0 | 상태 LED | 페어링/연결/Wired, 저전압, 충전, 완충 표시 | 파란 페어링과 open-drain 빨간 저전압 점멸 구현. 충전/완충 등은 남음 | 파란 페어링 실기 통과, 10% 이하 빨간 점멸 실물 확인, 나머지 순정 truth table을 전기적 충돌 없이 재현 |
| P0 | 소비전류 기준 | 공식 사용 시간은 충전당 약 2주 | idle 전류와 배터리 사용 시간 미측정 | 같은 반쪽·같은 조건에서 순정/Rust 전류 비교 완료 |
| 후보 | 좌우 입력 순서 | 누락·중복·고착 없이 물리적인 좌우 이벤트 순서 보존 | 원본 timestamp·순번·시계 변환·5 ms 전역 대기열로 도착 순서 의존 제거; 5 ms는 실기 미검증 | 실제 jitter 측정과 runtime queue 전체 10,000회 검증, Wired/Windows 11/Android 및 재연결 직후 스트레스 통과 |
| P0 회귀 / P3.3 배포 | split 재연결 | 반쪽을 가까이 옮기지 않고 일반 책상 거리에서 오른쪽 재연결 | P3.2에서 6.1/17.2초 입력 복귀. 현재 P4 오른쪽은 +8 dBm 배포 | +8 dBm 거리·보안·전류를 통제 비교; P4 입력 성공만으로 무선 효과를 단정하지 않음 |
| 완료 | 백라이트 동기화 | 명령·timeout·wake·재부팅·재연결 뒤 양쪽 상태 일치 | 왼쪽 기준 version 포함 절대 상태가 상대 split toggle을 대체하고 GATT 탐색 뒤 재전송됨 | 왼쪽 OFF 중 오른쪽 재부팅 뒤 1회 수렴, 토글 10회·timeout·오른쪽 키 wake까지 동기 유지 |
| 완료 | idle 스캔 | PCA9555 `INT` wake와 안전용 주기 스캔 | 왼쪽 `P0.31`과 오른쪽 `P0.05`로 즉시 wake, active debounce는 3 ms 유지, interrupt 유실은 250 ms 전체 스캔으로 복구 | 양쪽 idle 첫 키·hold·release·혼합 순서 실기 통과, idle 스캔 약 100회/s에서 4회/s로 감소 |
| 완료 | 백라이트 timeout | NocFree 키 무입력 30초 후 자동 소등 | USB 전원과 무관하게 양쪽이 꺼지고 첫 키 손실 없이 복귀 | 동일 경로의 10초 진단 이미지를 실기 검증했고 배포 이미지는 상수만 30초로 변경 |
| 완료 | 왼쪽 central System OFF | 배터리 상태 5분 후 sleep, 왼쪽 키 또는 왼쪽 USB로 wake | 10초 진단 이미지로 USB 차단, 배터리 System OFF, 양쪽 백라이트 사전 소등, USB/키 wake, BLE와 오른쪽 split 복귀 확인. 배포 timeout은 5분 | sleep 전류 측정과 장시간 반복 wake 검증 필요. 짧은 wake 키 탭은 reset 부팅을 통과한다고 보장하지 않음 |
| 완료 | 주기적 배터리 관리 | 잔량과 저전압을 계속 감시 | 양쪽 모두 60초마다 그리고 `Fn+I` 요청 때 측정하며 필터값을 `Fn+I`, split 배터리 전달, 저전압 LED에서 공유 | 자동 테스트, 실기 `L 100 R 100`, 장시간 표시 안정성 확인 |
| P1 | 배터리 표시 경로 | `Fn+I`와 NocFree Link에서 유효한 정보 표시 | `Fn+I` 동작, Link는 `0xff`, BLE Battery Service 없음 | `Fn+I`/Link/BLE 값 일치, 오른쪽 단절을 0%로 오표시하지 않음 |
| P1 | 충전 상태 인식 | 방전 잔량과 충전/완충 상태 구분 | VBUS/charger 상태를 잔량 계산에 반영하지 않음 | 충전/완충 표시가 맞고 충전 전압을 100%로 오판하지 않음 |
| P1 | 백라이트 효과/설정 | 정적 제어, 자동 동작, 공식 문서의 breathing 지원 | toggle과 20% 정적 단계만 지원 | 선택 효과가 양쪽에서 동작하고 Link 제공 시 재부팅 후 보존 |
| P2 | USB 동글 / 2.4 GHz | 왼쪽이 상태를 소유하는 `오른쪽 -> 왼쪽 -> 동글 -> PC` | D1 무선 코드 없는 USB keyboard/consumer/CDC와 복구 완료, 2.4G 스위치는 아직 출력 차단 | 전용 링크, pairing, 실제 HID 입력, reconnect/입력 순서/latency/coexistence 통과 |
| P2 | NocFree Link 완성도 | 배터리, 조명, 전원 설정, macro/Quick Text, updater 관련 경로 | keymap/hotkey 외에는 부분 또는 미구현 | 노출된 각 Link 화면이 timeout 없이 동작하고 재부팅 후 보존 |
| ANSI 범위 밖 | Numpad | 순정의 별도 지원 장치 | 이 84키 ANSI 프로젝트에서는 미지원 | 범위를 넓힐 때 별도 추적 |

## 다음 진행 순서

1. **D1 완료:** 무선 코드 없는 동글 USB keyboard/consumer/CDC 열거와 공장
   UF2/CDC 복구 왕복을 Windows 11에서 확인했습니다.
2. **D2 다음:** 합성 절대 키 상태로 왼쪽→동글 전용 BLE 링크를 검증하고 기존
   `Fn+1/2/3` host profile과 동글 bond를 분리합니다.
3. **D3:** 실제 왼쪽 기준 keyboard/consumer report를 물리 2.4G 스위치에 연결하고
   출력 전환 때 이전 경로의 모든 키를 release합니다.
4. **D4:** 재연결, sleep/wake, 동글 분리, 모드 전환, stuck key, latency와
   순정/Rust 복구 왕복을 검증합니다.
5. **D5 선택:** 측정 결과 더 낮은 latency나 순정 프로토콜 호환이 필요할 때만
   S140 Radio Timeslot과 ESB를 조사합니다.

## 물리 스위치 옆 LED

포팅 자료에서 확인되는 신호는 다음과 같습니다.

- 왼쪽 빨간 충전/저전압 선: `P0.09`, charger status와 공유
- 왼쪽 파란 상태 LED: `P0.10`, active low
- 오른쪽 빨간 충전/저전압 선: `P0.17`, charger status와 공유

빨간 선은 open-drain 방식처럼 다뤄야 합니다. firmware가 표시할 때만 low로
당기고, USB 전원이 있을 때는 charger 회로가 선을 제어하도록 high-impedance로
해제해야 합니다. 회로도와 hardware revision을 확인하기 전에는 절대로 high를
직접 출력하지 않아야 합니다.

빨간 출력은 `Standard0Disconnect1`을 사용합니다. 10% 이하에서만 0.5초 켜짐/
0.5초 꺼짐으로 low에 당기고 그 외에는 선을 해제합니다. active-low 파란 LED도
BLE 프로필이 페어링 중일 때 같은 주기로 점멸합니다. `Fn+3`에서 손을 뗀 뒤에도
파란 LED가 계속 점멸하고 `Fn+1`로 bond 슬롯을 선택하면 꺼지는 것을 실기로
확인했습니다. 양쪽 배터리가 완충이라 빨간 점멸은 실물 확인하지 못했습니다.

남은 작업은 순정 V2.3.0의 boot, Wired, 연결, 충전, 완충, link loss 동작을
truth table로 기록하고 확인된 패턴만 재현하는 것입니다. 공식 설명서는 pairing
중 파란색 점멸과 저전압 때 빨간색 점멸은 명시하지만 모든 상태와 점멸 주기를
정의하지는 않습니다.

## 배터리 표기 검증과 보정 방법

순정 V2.3.0의 배터리 경로를 복구해 구현했습니다.

```text
adc_mV = raw * 3300 / 4095
battery_mV = adc_mV * 130 / 100
filtered_mV = previous * 0.75 + new * 0.25
percent = clamp(((floor(filtered_mV / 33) - 70) * 10) / 3, 0, 100)
```

divider는 측정할 때만 켜고 8회 평균을 사용하며, 양쪽 모두 60초마다 측정합니다.
연결되지 않은 오른쪽 초기값은 0%가 아니라 unknown입니다. 아래 절차는 실제
셀에서 복구한 순정 동작을 검증하기 위한 것이며, 측정으로 문제가 드러날 때만
순정 곡선을 바꿔야 합니다.

### 1. ADC 전압부터 맞추기

임시 진단용으로 raw SAADC 값과 계산된 mV를 CDC 또는 진단용 `Fn+I` 출력에
노출합니다. 양쪽 각각 약 4.20, 4.00, 3.80, 3.65, 3.50 V에서 firmware 값과
배터리 단자의 멀티미터 값을 비교합니다. USB 연결 여부와 백라이트 상태도 같이
기록해야 합니다.

hardware revision별 gain과 offset을 구합니다.

```text
보정_mV = 측정_mV * gain + offset
```

전압 오차를 알기 전에는 퍼센트 곡선을 조정하면 안 됩니다. `130/100` divider
배율은 순정 구현과 일치하지만 PCB와 저항 오차는 여전히 실측해야 합니다.

### 2. 실제 방전 곡선 만들기

양쪽 1100 mAh 배터리를 완충하고 USB를 분리한 뒤 잠시 안정화합니다. 백라이트
밝기와 사용 부하를 고정하고 보정된 전압과 경과 시간, 가능하면 battery analyzer
또는 power profiler의 방전 mAh를 기록합니다. 왼쪽과 오른쪽은 radio 역할과
부하가 다르므로 각각 확인해야 합니다.

측정 결과를 복구한 순정 계산과 비교합니다. 정확도가 충분하면 순정 계산을
유지하고, 문제가 확인될 때만 측정값 기반의 작은 piecewise lookup table로
바꿉니다. 일반 Li-ion 표를 그대로 복사하지 말고 이 셀과 PCB에서 검증해야
합니다.

### 3. 표시값 안정화

- divider-enable 핀이 high인 동안만 측정하고 즉시 다시 끄는 현재 동작을 유지합니다.
- 현재 75/25 IIR filter는 순정과 일치합니다. 로그가 필요성을 보일 때만 이상값 제거를 추가합니다.
- 경계에서 퍼센트가 오르내리지 않도록 hysteresis를 둡니다.
- 현재 60초 측정 주기가 소비전력을 유의미하게 늘리지 않는지 확인합니다.
- 아직 값을 받지 못한 오른쪽은 명시적인 unknown 상태를 유지합니다.

### 4. 충전 중 상태를 분리

충전 중에는 전압이 올라가므로 정확한 잔량 기준이 아닙니다. VBUS 감지와 해제된
빨간 charger-status 선을 함께 사용해 방전/충전/완충을 구분합니다. 단순히
4.20 V에 도달했다는 이유만으로 100%를 표시하지 말고 완충 상태를 확인해야
합니다.

### 5. 하나의 측정값을 모든 기능에서 공유

배터리 manager 하나가 다음에 값을 제공하도록 구성합니다.

- `Fn+I`의 왼쪽/오른쪽 출력
- 현재 `0xff`인 NocFree Link 배터리 응답
- 왼쪽/central용 표준 BLE Battery Service
- 암호화 split link를 통한 오른쪽 잔량
- 저전압 LED와 전원 정책

## 배터리 최적화 진행 순서

1. 코드 변경 전에 순정/Rust 소비전류를 측정합니다. 양쪽, 연결/비연결,
   백라이트 off/20%/100%, USB 연결/분리를 같은 조건으로 비교합니다.
2. **완료:** 오른쪽 `P0.05` 출력 구동을 중단했습니다.
3. **완료:** idle 10 ms polling을 왼쪽 `P0.31`/오른쪽 `P0.05` interrupt
   wake + 250 ms 안전 스캔으로 바꾸고 3 ms active debounce를 유지합니다.
4. **완료:** 동일 경로를 10초 진단 이미지로 먼저 확인한 뒤 30초 백라이트 자동 소등을 구현했습니다.
5. **완료:** 배터리 상태 5분 뒤 왼쪽 central이 System OFF에 들어가고 왼쪽
   `P0.31` 또는 왼쪽 USB로 깨도록 구현했습니다. BLE-only split에는 꺼진 오른쪽을
   왼쪽이 깨울 물리 경로가 없어 오른쪽은 System ON idle을 유지합니다.
6. **부분 완료:** 최초 split 연결 뒤 오른쪽 단절 광고를 250 ms에서 1초로
   늦췄습니다. 배터리 사용 시간 개선을 주장하기 전에 실제 전류를 측정해야 합니다.
7. 전원 변경마다 latency, rollover, split reconnect, BLE 멀티 페어링, DFU,
   warm I2C recovery를 다시 검사합니다.

같은 실물 장치에 순정 이미지를 올렸을 때의 전류를 가장 중요한 목표로 삼는 것이
좋습니다. 공식 약 2주 사용 시간은 전체 판정에는 유용하지만 백라이트와 사용량의
영향이 커서 직접 전류 측정을 대신할 수는 없습니다.

## 자료

- [원본 NocFree community ZMK 포팅 자료](https://github.com/NocFreeKB/NocFree-and-zmk)
- [NocFree & 공식 설명서](https://www.nocfree.com/pages/nocfree-and-manual)
- [NocFree 공식 제품 사양](https://www.nocfree.com/products/nocfree-and-reservation)
