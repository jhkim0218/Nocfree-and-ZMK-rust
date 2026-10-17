# Bluetooth 페어링 실패 실기 조사

상태: 최종 구독 확인 수정판에서 휴대폰 슬롯 2 및 Windows 슬롯 1 연결·양쪽 입력 확인.
Windows 목록 표시는 고급 검색으로 해결. 슬롯 3 및 장시간 반복 전환은 미검증.

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

## 후속: 검색 및 연결 실패

사용자는 NocFree 1이 Windows 장치 추가 목록에 나타나지 않는다고 보고했다.
Windows BLE AdvertisementWatcher 직접 검색에서 NocFree 1, HID UUID 0x1812,
connectable undirected 광고 및 RSSI -48 dBm을 확인했다.
Windows AssociationEndpoint 조회도 IsPaired=False, CanPair=True,
IsConnectable=True였다. 사용자가 고급 검색을 선택한 뒤 목록에 나타났으나 연결은 실패했다.
직접 기본/ConfirmOnly 암호화 페어링 API도 Failed, GATT 탐색은 Unreachable을 반환했다.

host 단계 로그를 추가한 진단판 SHA-256:
`80aacf833e8d5c15b71be56508555feba57f7b9f58340f72d690ff3a38ac4311`.
설치 후 연결 요청 허용(accepted=1), 약 5초 뒤 secured=0 반복을 관측했다.
보안 대기 중 GATT 처리가 실행되지 않아 SYS_ATTR_MISSING/권한 부여 요청을 처리하지
못하는 코드 위험을 확인하고 GATT 서버를 보안 대기와 함께 실행하도록 수정했다.
해당 GATT 수정판 SHA-256:
`42c733f1b42e0757a2a7e8ce1c4b986463f621c40580d4213eb5237a959ebd62`.
설치·split-ready는 확인했으나 이후에도 secured=0의 약 5초 연결 반복이 관측되었다.

신규 Bluetooth 페어링의 보안 대기만 30초(20ms × 1,500회)로 늘린 다음 후보를 준비했다.
기존 본딩 재연결과 동글은 5초(20ms × 250회) 유지. 출력/프로필 변경은 계속 즉시 취소 가능하다.
파일: `target/ble-pairing-recovery/NocFree_BLE_Pairing_30s_Experimental_ANSI_Left.uf2`
SHA-256: `a7029a1802704e41fc6b515f5ad8db3867ede558e4c0b74c1766698eb898acfe`.
계약 검사 17개, ARM Clippy/build, UF2 application 경계 통과.
USB 재연결 후 이 파일을 설치했고 split-ready를 확인했다.
실기에서는 연결 허용 후 약 30초 뒤에도 secured=0이었다. 사용자도 연결 실패를 확인했다.
따라서 대기 시간 증가만으로 해결되지 않았다. 다른 기존 연결 기기는 없다고 사용자가 확인했다.
Windows Bluetooth를 끄고 휴대폰의 신규 슬롯 2 페어링을 시험하도록 요청한 상태다.

모든 시도는 application UF2만 교체했고 저장 키맵·split bond는 유지했다.
배포 전 백업은 각각 `deployment-20261017-diagnostic/`, `deployment-20261017-gatt/`,
`deployment-20261017-30s/`에 있다.

## 입력 알림 구독 준비 수정

사용자는 휴대폰에서도 실패한다고 확인했다. 이후 로그에서 암호화·설정 복원 성공 직후
연결이 끊기는 사례를 관측했다. 기존 `notify_ble_reports`는 구독 여부와 무관하게 키보드·
consumer·pointer 알림을 모두 전송했고, 아직 구독되지 않은 특성의 오류도 연결 종료로 이어졌다.
세 입력 특성의 CCCD를 현재 connection handle로 조회해 구독된 항목만 전송하도록 수정했다.
복원된 본딩의 구독도 같은 조회로 처리한다. 이후 CCCD 쓰기 때 기존 최신 상태 재동기화를 사용한다.
알림 실패의 SoftDevice 오류 코드도 진단에 기록한다.

최종 설치 파일: `target/ble-pairing-recovery/NocFree_BLE_Subscription_Experimental_ANSI_Left.uf2`
SHA-256: `db0f57fb915924e7627eeec8a22f7d761ce67a564eb3cbad7dd63201bed6a313`.
백업: `target/ble-pairing-recovery/deployment-20261017-subscription/before-ANSI-Left.uf2`.
계약 검사 18개, ARM Clippy/build 통과. 사용자가 휴대폰 슬롯 2의 연결·양쪽 입력을 확인했다.
로그: slot=2 accepted=1, secured=1, attrs-restored=1.

## Windows 최종 확인

Windows 재시도에서는 슬롯 1이 암호화된 뒤 저장된 상대와 Windows public 주소가 달라
재접속이 accepted=0으로 거절되는 로그를 관측했다. 실제 flash 백업의 슬롯 1/2/3이
모두 같은 identity 주소를 저장하고 있었다. 슬롯 2 휴대폰 성공 직후의 identity와 동일했다.
휴대폰 자동 재접속이 신규 슬롯 1 페어링을 먼저 차지한 것으로 판단했다.
사용자가 휴대폰 Bluetooth를 완전히 끈 뒤 슬롯 1을 다시 새 페어링하자
Windows 연결과 양쪽 입력이 성공했다고 확인했다.

사용 방법: Fn+1/2/3 짧게는 저장된 슬롯 선택, 1초 이상은 해당 슬롯 본딩 삭제 후 신규 페어링.
신규 페어링 시 기존 연결 기기의 자동 재접속이 개입하지 않도록 해당 기기의 Bluetooth를
잠시 끈다. 다른 슬롯 identity를 일괄 차단하는 정책은 동일 host의 복수 슬롯 사용에 영향을
주므로 이번 수정에 추가하지 않았다.

검증 범위: Windows 슬롯 1 및 휴대폰 슬롯 2의 신규 연결·양쪽 입력, USB 진단 응답,
split 재연결, ANSI 라이브러리 114개 및 저장소 계약 검사 18개, ARM Clippy/build,
PowerShell 문법과 application UF2 경계. 슬롯 3 실기, 반복 전환·절전·장시간 안정성 및
Caps Lock 동작의 최종판 재시험은 수행하지 않았다.
