# NocFree 로컬 패치

[English](README.nocfree.md) · [한국어](README.nocfree_ko.md) · [日本語](README.nocfree_ja.md)

이 디렉터리는 `embassy-rs/nrf-softdevice` commit
`b0ac850c0a5a05b8a5aef4f752b48115755b8542`에서 가져왔습니다.

동작 변경은 `src/ble/gap.rs`에 있습니다. peer가 2M을 요청하더라도 PHY update에
RX/TX 모두 1 Mbps로 응답해 원본 NocFree 펌웨어의 1M 전용 BLE 조건을 유지합니다.

관련 path dependency는 같은 고정 Git revision으로 바꿨습니다. 따라서 불필요한
SoftDevice binding 전체를 복사하지 않고 이 crate만 vendor로 유지할 수 있습니다.

secured notification characteristic의 CCCD도 보호하기 위해 형제
`nrf-softdevice-macro` crate 역시 로컬에 있습니다. 자세한 내용은 해당 patch 문서에
있습니다.
