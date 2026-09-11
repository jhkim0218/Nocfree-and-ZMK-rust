# Caps Lock 홀드 Layer 2

작성일: 2026-09-06

## 동작

기본 Layer 1 또는 Fn 레이어에서 Caps Lock을 짧게 누르면 일반 Caps Lock HID 키를 한 번 보낸다.
Caps Lock을 200 ms 이상 누르거나, Caps Lock을 누른 상태에서 다른 키를 누르면 Caps Lock은 Layer 2를
누르는 동안만 활성화한다. Caps Lock을 뗄 때 Layer 2도 즉시 해제된다.

Layer 2의 지정되지 않은 키는 transparent다. 실행할 때에는 기본 키를 그대로 통과시키지만,
NocFree Link 화면에는 `Trans`로 보인다. 따라서 JIS 고유 키를 포함한 기존 기본/Fn 키맵은
변경하지 않는다.

| 물리 QWERTY 키 | Layer 2 동작 |
|---|---|
| Q | Browser Back |
| W / A / S / D | Up / Left / Down / Right |
| E / R | Home / End |
| Z / X | Page Up / Page Down |
| C / V / B | Left click / Right click / Middle click |
| U / M | Wheel Up / Wheel Down |
| I / J / K / L | Mouse Up / Left / Down / Right |

포인터 이동은 20 ms 처리 주기마다 8 단위의 relative HID report를 보낸다. 클릭은 키를
누르는 동안 유지한다.

## Link 저장 키맵

새 기본 키맵은 Layer 2에 위 동작을 저장하고 Layer 3~7은 기본 Layer 1 복사본으로 둔다.
레코드 버전은 ANSI/ISO/JIS `8`, KR `10`이다. 이전 버전을 읽으면 Layer 0, Fn, hotkey와
사용자가 변경한 Layer 2~7은 보존한다. 변경되지 않은 이전 기본 레이어 하나가 있으면 그
레이어에 새 Caps Lock 동작을 설치한다. Layer 2~7을 모두 사용자가 변경한 경우에는 보이지 않는
내부 Layer 8을 사용해 Caps Lock Hold 기능을 유지한다.

Link Function에서 Caps Lock을 일반 Caps Lock이 아닌 값으로 다시 지정하면 그 지정값을 그대로
실행하며 Caps Lock 홀드 레이어는 적용하지 않는다. Link Mouse에서는 Mouse Up/Down/Left/Right,
Button 1~3, Wheel Up/Down만 이 펌웨어가 받는다.

## 전송 경로

Wired USB에는 Consumer와 Mouse report ID를 하나의 HID interface로 제공한다. Bluetooth HID에는 Report ID 3 Mouse
report를 추가했다. Caps Lock Layer 2 포인터 동작은 이 두 경로에서 지원한다. 현재 Rust 2.4G
dongle protocol은 keyboard/consumer frame만 전송하므로 이 포인터 동작을 제공하지 않는다.

## 확인 절차

1. Wired 모드에서 Caps Lock 탭이 Caps Lock을 한 번 전환하는지 확인한다.
2. Caps Lock을 누른 채 W/A/S/D, Q, Z/X, E/R을 눌러 탐색 키가 동작하는지 확인한다.
3. Caps Lock을 누른 채 I/J/K/L, U/M, C/V/B를 눌러 포인터·휠·세 버튼을 확인한다.
4. Bluetooth 모드에서 1~3을 반복하고, Caps Lock release 뒤 일반 키가 남지 않는지 확인한다.
5. Link에서 Layer 2와 Layer 3~7을 읽어 Layer 2의 지정 키와 나머지 레이어의 기본 바인딩을 확인한다.

자동 검증은 `cargo test --lib --target x86_64-pc-windows-msvc --no-default-features --features layout-ansi space_hold_layer2`와 `python -B tools/build_release.py --all-layouts --dongle`를 사용한다. 실물 USB/Bluetooth 검증은 대상 장치에 새 UF2를 설치한 뒤 위 절차로 기록한다.
