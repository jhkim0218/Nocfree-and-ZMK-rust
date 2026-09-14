# Caps Lock 홀드 Layer 2

작성일: 2026-09-06

## 동작

기본 Layer 1 또는 Fn 레이어에서 Caps Lock을 짧게 누르면 일반 Caps Lock HID 키를 한 번 보낸다.
Caps Lock을 2초 미만에 떼면 Caps Lock을 한 번 전환한다. 2초 이상 계속 누를 때만 Layer 2를
누르는 동안 활성화한다. 다른 키를 함께 눌러도 대기 시간을 줄이지 않는다.
Caps Lock을 뗄 때 Layer 2가 해제되며, 홀드 후에는 Caps Lock 전환을 보내지 않는다.
이미 누르고 있던 다른 키는 뗄 때까지 처음 지정된 동작을 유지하므로, 마우스 키는 2초 후에 누른다.

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
2. Caps Lock을 2초 이상 누른 뒤 W/A/S/D, Q, Z/X, E/R을 눌러 탐색 키가 동작하는지 확인한다.
3. Caps Lock을 2초 이상 누른 뒤 I/J/K/L, U/M, C/V/B를 눌러 포인터·휠·세 버튼을 확인한다.
4. Bluetooth 모드에서 1~3을 반복하고, Caps Lock release 뒤 일반 키가 남지 않는지 확인한다.
5. Link에서 Layer 2와 Layer 3~7을 읽어 Layer 2의 지정 키와 나머지 레이어의 기본 바인딩을 확인한다.

자동 검증은 `cargo test --lib --target x86_64-pc-windows-msvc --no-default-features --features layout-ansi caps_hold_layer2`를 사용한다. 실물 USB/Bluetooth 검증은 대상 장치에 새 UF2를 설치한 뒤 위 절차로 기록한다.

## 2026-09-14 수정 검증

0.2초 기준과 다른 키에 의한 즉시 홀드를 제거했다. 새로 누른 Caps Lock을 이전 누름 시각으로
판정하던 오류도 수정했다. 1,999 ms 탭, 2,000 ms 홀드, 긴 가동 시간, 반복 입력,
동시 입력, 주기 처리 없이 키를 뗀 경우를 자동 테스트한다.

자동 검증 결과: ANSI/ISO/JIS 각 114개, KR 121개 전체 라이브러리 테스트 통과(무시된 테스트 없음).
네 레이아웃 Left의 ARM release 빌드와 Clippy(`-D warnings`), 포맷 검사 통과.
아래 UF2 4개의 SHA-256과 application 영역(`0x27000..0x65000`) 내 주소만 포함하는 것을 확인했다.
검증 로그는 `target/caps-hold-checks.log`에 있다.

실물 검증은 아직 수행하지 않았으며 장치에 설치하지 않았다. 아래 실험용 Left 펌웨어에는
`feature/battery-desktop`의 배터리 기능도 포함된다. 오른쪽과 동글의 변경은 없다.

실물 확인 시 해당 하드웨어 레이아웃의 파일을 사용하고 다음 항목을 Wired와 Bluetooth에서 각각 확인한다.

1. 부팅 후 10초 이상 기다린 뒤 Caps Lock을 짧게 여러 번 눌러 매번 대소문자가 한 번씩 전환되는지 확인한다.
2. Caps Lock과 I를 동시에 누르거나, Caps Lock을 누른 뒤 2초 전에 I를 눌러 문자 I만 입력되고 마우스가 움직이지 않는지 확인한다.
3. Caps Lock을 약 1.9초에 떼면 대소문자가 전환되고, 2.1초 이후 I를 새로 누르면 마우스가 움직이는지 확인한다.
4. 홀드 후 모든 키를 떼면 이동·클릭이 멈추고, 대소문자는 바뀌지 않으며 이후 I는 문자로 입력되는지 확인한다.
5. 위 탐색·휠·클릭 확인 절차를 반복한다. Windows/macOS 결과는 각각 기록한다.

실물 통과 전에는 모든 파일을 Experimental로 취급한다. 2.4G 동글의 마우스 전송은 기존 미지원 상태다.

Artifacts: `target/caps-hold-preview/`

| File | SHA-256 |
|---|---|
| `NocFree_Caps_2s_Battery_Experimental_ANSI_Left.uf2` | `c7fdc5259f5b4ec6ab9b91fb4d7cd64045173d150d81a687c880bd341a349653` |
| `NocFree_Caps_2s_Battery_Experimental_ISO_Left.uf2` | `0d66534384f630bb58d5be25089c590bc3353bedaa470558f1873072a560e337` |
| `NocFree_Caps_2s_Battery_Experimental_JIS_Left.uf2` | `023fd2f804e3c33721b4c1120bdd288c0e76b6cfbdfbd7753b710e109f54b05d` |
| `NocFree_Caps_2s_Battery_Experimental_KR_Left.uf2` | `a1b83da92dccc0b5ecb16bc34788c778d621702267d8615819b8981b989aab16` |
