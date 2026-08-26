# NocFree 로컬 패치

[English](README.nocfree.md) · [한국어](README.nocfree_ko.md) · [日本語](README.nocfree_ja.md)

이 디렉터리는 `embassy-rs/nrf-softdevice` commit
`b0ac850c0a5a05b8a5aef4f752b48115755b8542`에서 가져왔습니다.

upstream의 characteristic `security = "..."` 옵션은 value attribute에는 보안을
적용하지만 자동 생성 CCCD/SCCD에는 적용하지 않습니다. 로컬 패치는 같은 security
mode를 descriptor metadata에도 적용합니다. NocFree는 암호화 split-state notification
구독을 보호할 때 이 동작을 사용합니다.
