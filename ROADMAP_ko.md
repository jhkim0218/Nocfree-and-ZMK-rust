# 순정 기능 비교와 로드맵

[English](ROADMAP.md)

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
| Rust 왼쪽 ANSI | 134,144 | `0x27000..0x375FF` |
| Rust 오른쪽 ANSI | 78,336 | `0x27000..0x308FF` |

순정에 별도 동글 이미지가 있다는 점은 공장 2.4 GHz 기능이 세 firmware로 구성된
시스템임을 보여줍니다. 왼쪽과 오른쪽 Rust 이미지만 바꿔서는 복원할 수 없습니다.

## 추가할 기능 목록

| 우선순위 | 영역 | 순정 동작 / 목표 | 현재 Rust 상태 | 완료 판정 |
|---|---|---|---|---|
| P0 | 오른쪽 `P0.05` 핀 역할 | active-low PCA9555 `INT` | 사용하지 않는 `_backlight_enable` 출력으로 설정됨 | high-impedance 입력으로 변경하고 키 스캔·warm recovery 재통과 |
| P0 | 배터리 정확도 | 양쪽 1100 mAh 배터리의 의미 있는 잔량 | 8회 ADC 평균, 고정 `130/100` divider 배율, 3.45–4.20 V 선형 퍼센트 | 방전 구간별 DMM 전압 오차와 퍼센트 오차 기록 |
| P0 | 상태 LED | 페어링/연결/Wired, 저전압, 충전, 완충 표시 | 미구현 | 순정 truth table을 측정하고 전기적 충돌 없이 재현 |
| P0 | 소비전류 기준 | 공식 사용 시간은 충전당 약 2주 | idle 전류와 배터리 사용 시간 미측정 | 같은 반쪽·같은 조건에서 순정/Rust 전류 비교 완료 |
| P1 | idle 스캔 | PCA9555 `INT` wake와 안전용 주기 스캔 | idle에도 expander 세 개를 10 ms마다 polling | 누락/stuck key 없이 I2C 활동 시간과 idle 전류 감소 |
| P1 | 백라이트 timeout | 5분 무입력 후 자동 소등 | 무입력 timer 없음 | 5분 후 꺼지고 첫 wake key 손실 없이 복귀 |
| P1 | deep sleep | 30분 후 sleep, 장시간 sleep은 왼쪽 키로 wake | deep sleep/soft-off 없음 | 반복 sleep/wake/reconnect와 sleep 전류 실측 통과 |
| P1 | 주기적 배터리 관리 | 잔량과 저전압을 계속 감시 | `Fn+I` 요청 때만 측정 | filtering된 주기 측정값을 모든 출력 경로에서 공유 |
| P1 | 배터리 표시 경로 | `Fn+I`와 NocFree Link에서 유효한 정보 표시 | `Fn+I` 동작, Link는 `0xff`, BLE Battery Service 없음 | `Fn+I`/Link/BLE 값 일치, 오른쪽 단절을 0%로 오표시하지 않음 |
| P1 | 충전 상태 인식 | 방전 잔량과 충전/완충 상태 구분 | VBUS/charger 상태를 잔량 계산에 반영하지 않음 | 충전/완충 표시가 맞고 충전 전압을 100%로 오판하지 않음 |
| P1 | 백라이트 효과/설정 | 정적 제어, 자동 동작, 공식 문서의 breathing 지원 | toggle과 20% 정적 단계만 지원 | 선택 효과가 양쪽에서 동작하고 Link 제공 시 재부팅 후 보존 |
| P2 | 공장 2.4 GHz 동글 | 왼쪽·오른쪽·선택적 numpad가 USB receiver와 통신 | 2.4G 스위치 위치에서 안전하게 출력 차단 | 동글 pairing/reconnect/입력 순서/latency/recovery/coexistence 통과 |
| P2 | NocFree Link 완성도 | 배터리, 조명, 전원 설정, macro/Quick Text, updater 관련 경로 | keymap/hotkey 외에는 부분 또는 미구현 | 노출된 각 Link 화면이 timeout 없이 동작하고 재부팅 후 보존 |
| ANSI 범위 밖 | Numpad | 순정의 별도 지원 장치 | 이 84키 ANSI 프로젝트에서는 미지원 | 범위를 넓힐 때 별도 추적 |

## 물리 스위치 옆 LED

포팅 자료에서 확인되는 신호는 다음과 같습니다.

- 왼쪽 빨간 충전/저전압 선: `P0.09`, charger status와 공유
- 왼쪽 파란 상태 LED: `P0.10`, active low
- 오른쪽 빨간 충전/저전압 선: `P0.17`, charger status와 공유

빨간 선은 open-drain 방식처럼 다뤄야 합니다. firmware가 표시할 때만 low로
당기고, USB 전원이 있을 때는 charger 회로가 선을 제어하도록 high-impedance로
해제해야 합니다. 회로도와 hardware revision을 확인하기 전에는 절대로 high를
직접 출력하지 않아야 합니다.

패턴 구현 전에 순정 V2.3.0을 올리고 boot, Wired, BLE advertising, BLE pairing,
연결, 저전압, 충전, 완충, link loss 상황에서 양쪽 LED를 truth table로 기록해야
합니다. 공식 설명서는 pairing 중 파란색 점멸과 저전압 때 빨간색 점멸은
명시하지만 모든 점멸 주기까지 정의하지는 않습니다.

## 배터리 표기를 고치는 방법

### 1. ADC 전압부터 맞추기

임시 진단용으로 raw SAADC 값과 계산된 mV를 CDC 또는 진단용 `Fn+I` 출력에
노출합니다. 양쪽 각각 약 4.20, 4.00, 3.80, 3.65, 3.50 V에서 firmware 값과
배터리 단자의 멀티미터 값을 비교합니다. USB 연결 여부와 백라이트 상태도 같이
기록해야 합니다.

hardware revision별 gain과 offset을 구합니다.

```text
보정_mV = 측정_mV * gain + offset
```

전압 자체가 맞기 전에는 퍼센트 곡선을 조정하면 안 됩니다. 현재 `130/100`
divider 배율은 문서에 나온 시작값이지 완성된 보정값이 아닙니다.

### 2. 실제 방전 곡선 만들기

양쪽 1100 mAh 배터리를 완충하고 USB를 분리한 뒤 잠시 안정화합니다. 백라이트
밝기와 사용 부하를 고정하고 보정된 전압과 경과 시간, 가능하면 battery analyzer
또는 power profiler의 방전 mAh를 기록합니다. 왼쪽과 오른쪽은 radio 역할과
부하가 다르므로 각각 확인해야 합니다.

측정 결과로 작은 piecewise lookup table을 만들고 현재 3.45–4.20 V 선형식을
대체합니다. 일반 Li-ion 표를 그대로 복사하지 말고 이 셀과 PCB에서 검증해야
합니다.

### 3. 표시값 안정화

- divider-enable 핀이 high인 동안만 측정하고 즉시 다시 끕니다.
- 이상값을 제거하고 작은 moving average 또는 exponential filter를 적용합니다.
- 경계에서 퍼센트가 오르내리지 않도록 hysteresis를 둡니다.
- 측정 자체의 전력 소모가 작도록 처음에는 30–60초 주기로 시작하고 실측 후
  조정합니다.
- 오른쪽 연결이 끊겼을 때 명시적인 unknown 값을 사용합니다. 현재 초기값 0은
  완전 방전으로 오해될 수 있습니다.

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
2. 오른쪽 `P0.05` 출력을 중단하고 PCA9555 interrupt 입력으로 검증합니다.
3. idle 10 ms polling을 interrupt wake + 보수적인 주기 전체 스캔으로 바꿉니다.
   debounce 중에는 3 ms active scan을 유지합니다.
4. 5분 백라이트 자동 소등을 구현합니다.
5. 왼쪽 `P0.31`, 오른쪽 `P0.05`, USB와 필요한 mode-switch source로 깨는 30분
   deep sleep을 구현합니다. 첫 wake key가 사라지지 않는지 확인합니다.
6. BLE advertising/scanning/reconnect 전류를 측정하고 실제 영향이 큰 경로에만
   backoff를 추가합니다.
7. 전원 변경마다 latency, rollover, split reconnect, BLE 멀티 페어링, DFU,
   warm I2C recovery를 다시 검사합니다.

같은 실물 장치에 순정 이미지를 올렸을 때의 전류를 가장 중요한 목표로 삼는 것이
좋습니다. 공식 약 2주 사용 시간은 전체 판정에는 유용하지만 백라이트와 사용량의
영향이 커서 직접 전류 측정을 대신할 수는 없습니다.

## 자료

- [원본 NocFree community ZMK 포팅 자료](https://github.com/NocFreeKB/NocFree-and-zmk)
- [NocFree & 공식 설명서](https://www.nocfree.com/pages/nocfree-and-manual)
- [NocFree 공식 제품 사양](https://www.nocfree.com/products/nocfree-and-reservation)
