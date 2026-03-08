# Security Equivalence Checklist (A_secure vs B_note)

목적: `A_secure`와 `B_note`가 동일한 보안 의미론을 만족하는지 확인하여 apples-to-apples 비교를 보장한다.

## 공통 보안 동등성 체크

- [ ] Canonical residue enforcement  
  모든 지정 residue 셀이 canonical range(예: `0..p-1`)를 만족한다.

- [ ] Quotient/carry range binding  
  quotient/carry witness가 선언된 클래스 범위(예: `q31`, `q66`, bit, u3)에 정확히 바인딩된다.

- [ ] No-wrap interval guarantees  
  LHS/RHS 및 관련 결합식에 대해 no-wrap 전제가 명시되고 검증된다.

- [ ] Wiring/inventory completeness  
  quotient/carry inventory와 wiring 매핑이 완전하며 누락/중복이 없다.

## Negative Case Rejection (필수 5종)

- [ ] 1) `p^{-1}` witness toy attack  
  취약 데모 입력이 `A_secure`, `B_note`에서는 거부된다.

- [ ] 2) omitted quotient wiring  
  quotient wiring 누락 시 검증이 실패한다.

- [ ] 3) class-map mismatch (31/66)  
  잘못된 class-map 태깅 시 검증이 실패한다.

- [ ] 4) digest mismatch  
  certificate/manuscript/backend digest 불일치 시 검증이 실패한다.

- [ ] 5) inactive-row zero-extension violation  
  inactive-row zero-extension 위반 시 검증이 실패한다.

## 운영 체크

- [ ] 동일 입력/동일 scale에서 A/B 비교 로그가 동일 스키마(`docs/results_schema.json`)로 출력된다.
- [ ] 실패 케이스는 `status`와 `notes`에 원인이 명시된다.
