# Bluetooth 페어링 실패 실기 조사

상태: 사용자 승인 후 ANSI Left 설치 완료. Bluetooth 모드 및 페어링 LED 점멸 확인.
PC 페어링·양쪽 입력·슬롯 2/3 및 유선 복귀 회귀 확인 대기.

같은 프로젝트의 `키보드 배터리 표시 검토`, `동글모드 좌우 키보드 펌웨어 계획` 채팅을 확인했다.
현재 연결 장치는 Rust ANSI Left이며 이전 설치 기록은 Caps Lock 1초 배터리 실험판이다.
사용자는 Bluetooth 모드에서 Fn+1/2/3을 2초 이상 눌러도 LED가 깜박이지 않는다고 보고했다.
USB 케이블을 유지한 Bluetooth 전환 중 HID 배터리 조회가 실패했다.
이 현상만으로 정확한 SoftDevice 오류 코드는 확정할 수 없다.

실제 저장 키맵 GET 결과, 왼쪽 Fn은 row 5 col 3, 오른쪽 Fn은 row 5 col 7이다.
Fn 레이어의 row 1 col 1/2/3은 각각 Bluetooth 1/2/3으로 정상 저장되어 있다.
전환 전에 좌우 연결 및 배터리 값 100%/60%를 읽었다.

코드상 광고 실패의 즉시 재시도는 실행기를 독점할 수 있다. 실패를 기록하고
100ms 또는 새 제어 이벤트까지 기다리도록 수정했다. 또한 알림 전송 오류로
연결 루프를 나갈 때도 해제 완료를 기다린 후 다음 광고를 시작한다.
이는 확인된 코드 위험의 수정이며 실기 원인 확정이나 해결 완료 선언이 아니다.

검증: ANSI 라이브러리 114개, 저장소 계약 검사 16개 통과. ARM central release
Clippy와 빌드 통과. UF2 family ID 및 application 영역 경계를 확인했다.
부트로더·SoftDevice·저장 키맵을 변경하는 이미지는 생성하지 않았다.

산출물: `target/ble-pairing-recovery/NocFree_BLE_Pairing_Recovery_Experimental_ANSI_Left.uf2`

SHA-256: `7415cd959717c6f476d41a31d55544540eb0b90bb2e3f5d18dd6dc2d49e1dbaa`

설치 전 현재 Left UF2 백업 및 사용자 배포 승인이 필요하다. 설치 후 확인:

1. USB를 연결한 채 왼쪽 스위치를 Bluetooth로 바꾸고 USB 상태 조회가 계속 응답하는지 확인.
2. 왼쪽 스페이스 바로 왼쪽 Fn+1을 2초 유지. 해당 슬롯의 기존 본딩 삭제가 의도된 동작이다.
3. 파란 LED 점멸 및 호스트의 NocFree 1 검색·페어링·양쪽 입력을 확인.
4. 2/3번 슬롯도 같은 방식으로 확인. 기존 본딩 유지가 필요한 슬롯은 삭제 시험에서 제외하고 기록.
5. 유선 복귀·Caps Lock 1초 동작·오른쪽 입력을 확인.
6. 실패 시 `tools/read-split-diagnostics.ps1 -Role Left`로 광고 오류 코드를 수집.

host advertising failure code: 1=Timeout, 2=NoFreeConn,
0x10000 이상=0x10000 OR SoftDevice RawError 코드.

## 배포 및 실기 결과

2026-10-17 사용자가 설치를 승인했다. 1200-baud CDC로 Left 부트로더에 진입했고,
기존 `CURRENT.UF2`를 `target/ble-pairing-recovery/deployment-20261017/before-ANSI-Left.uf2`에 백업했다.
백업 SHA-256: `07931d8bf3e590d4b656e777032a794bee906d801bdad155bd47b9811bd1da47`.
검증된 application UF2를 설치한 뒤 COM19 재인식, USB 배터리 조회,
저장 Fn 키맵 보존 및 오른쪽 split-ready를 확인했다.

사용자는 수정판에서 Fn+1을 길게 누르면 파란 LED가 깜박인다고 확인했다.
동시에 USB 배터리 상태의 selected_output=1(Bluetooth)을 읽었다.
최초 제보인 페어링 모드 진입 불가는 이 조합에서 해소되었다.
이후 split timeout(0x08) 1회와 자동 재연결 성공을 관측했다.
장시간 안정성과 반복 전환은 미검증이며 광고 오류 코드 자체는 아직 수집되지 않았다.
