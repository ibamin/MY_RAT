# Red Team Simulation 개발 문서 (BAS + Game UI)

본 문서는 “C2 프레임워크 구현”이 아니라, 조직 통제하의 랩 환경에서 **Breach & Attack Simulation(BAS)** 을 수행하고 결과를 게임 UI로 이해하기 쉽게 제시하는 제품을 목표로 한다.

- 상태: 업데이트됨 (2026-02)
- 핵심 목표: 위협행위(ATT&CK 기반) 시뮬레이션을 **명확한 PASS/FAIL + 증거(evidence)** 로 평가하고, 게임 UI로 학습/운영 접근성을 높인다.

---

## 0. 목표 / 비목표 (중요)

### 목표
- “시나리오 기반 공격 시뮬레이션”을 제공한다 (TTX/BAS).
- 결과는 모호한 로그가 아니라, **사전 정의된 assertions** 를 근거로 한 **명확한 PASS/FAIL** 을 제공한다.
- UI는 C2/BAS에 친숙한 게임 감성을 입혀, 보안 비전공자도 ‘무엇이 일어났고 왜 성공/실패했는지’를 이해하도록 한다.

### 비목표
- 실제 환경에서의 원격 제어/원격 실행/파일 전송/페이로드 생성/리스너 운영 등 “현실 C2” 구현은 목표가 아니다.
- 자유 입력 커맨드 기반의 임의 실행(terminal-style command execution)은 제공하지 않는다.

---

## 1. BAS Safety Contract (현실성은 살리고, 안전하게)

"BAS의 위협행위 시뮬레이션"을 막지 않되, **랩 경계** 와 **평가 구조** 로 안전성과 재현성을 확보한다.

### 1.1 랩/스코프
- 시스템은 조직 통제 하의 랩 환경(VM/컨테이너/테스트 서브넷)에서만 운영한다.
- 대상은 “스코프 manifest” 에 의해 allowlist로 제한한다. (스코프를 못 읽으면 fail-closed)

### 1.2 네트워크 경계
- 랩 외부로의 egress는 기본 차단한다.
- 허용된 내부 sink/telemetry 서비스로만 통신할 수 있다.
- 허용/차단된 모든 네트워크 시도는 이벤트로 기록한다.

### 1.3 행위 시뮬레이션 경계
- 전술/기술은 “통제된 효과(controlled effect)”로 시뮬레이션한다.
- 결과는 반드시 관측 가능한 증거(evidence)로 남겨 평가 가능해야 한다.
- 실제 침투를 목적으로 하는 기능(지속성/은닉 원격제어/임의 실행 전달 등)은 제공하지 않는다.

---

## 2. 핵심 도메인 모델: Scenario -> Run -> Verdict

Metasploit/MSF류 도구에서 자주 발생하는 “성공/실패가 애매한 리포팅” 문제를 피하기 위해,
타임라인(Event)과 평가(Outcome)를 분리하고, PASS는 반드시 Evidence를 요구한다.

### 2.1 엔티티
- Scenario: 정적 정의(steps, assertions, 정책, allowed targets, seed)
  - scenario_id: 내부 식별자(UUID 또는 slug). API/DB 참조용.
  - test_id: 표시용 코드(예: BAS-DEMO-001). UI/리포트/게임 미션 코드.
- Run: Scenario의 1회 실행(누가/언제/어떤 대상/어떤 버전)
- Step: 시나리오의 원자 단위 목표
- Action: Step 수행을 위한 내부 동작(구현 상세는 캡슐화)
- Event: append-only 타임라인 기록(사실)
- Evidence: Event가 참조하는 불변 아티팩트(해시/로케이터)
- Assertion: Evidence를 근거로 하는 boolean 검증
- Verdict: PASS/FAIL (이유 코드 포함)

### 2.2 원칙
- Step은 반드시 PASS 또는 FAIL로 끝난다(“완료됨” 같은 중간 상태 금지).
- PASS는 반드시 evidence_refs가 1개 이상 있어야 한다.
- FAIL은 reason_code + 관련 evidence를 반드시 포함한다.

예시 reason_code:
- FAIL_POLICY_DENIED
- FAIL_EVIDENCE_MISSING
- FAIL_ASSERTION_FAILED
- FAIL_EXECUTION_ERROR
- FAIL_TIMEOUT
- FAIL_CLEANUP_NOT_VERIFIED

---

## 3. 전체 아키텍처 (현재 구현 + 확장 방향)

현재 레포는 Rust 기반의 로컬 BAS MVP 형태로 구현되어 있으며(`server/`, `sim_agent/`, `ui/`),
문서는 이 구조를 기준으로 확장 로드맵을 정의한다.

```mermaid
graph TB
  subgraph "Game UI Layer"
    UI[Dashboard\nElectron + React]
    VIZ[Optional Visualization\n2D/3D Map]
  end

  subgraph "Simulation Backend"
    API[Simulation Server\nHTTP API]
    DB[(SQLite\nRuns/Events/...)]
    RULES[Rules\nFingerprint/Scenario]
  end

  subgraph "Runners & Sensors"
    RUNNER[Scenario Runner\n(sim_agent today)]
    SINK[Telemetry Sink (optional)\nlab-only]
  end

  UI --> API
  API --> DB
  API --> RULES
  RUNNER <--> API
  RUNNER -.-> SINK
  UI -.-> VIZ
```

---

## 4. 현재 MVP 스펙 (코드 기준)

### 4.1 데이터 모델(현재)
- Agent: id, hostname, ip, os, arch, user, last_seen, status
- Run: id, agent_id, test_id, params_json, status, result_json, created_at, updated_at
- Event: id, run_id, agent_id, level, message, ts

문서 기준의 권장 확장:
- Run은 scenario_id를 별도 컬럼으로 가져가고, test_id는 표시용 코드로 유지한다.
- 현재 MVP에서는 runs.test_id가 사실상 시나리오 식별자 역할을 겸한다.

### 4.2 API(현재)
- `POST /api/agents/register`
- `POST /api/agents/:id/heartbeat`
- `GET /api/agents/list`
- `POST /api/runs`
- `GET /api/runs/pending/:agent_id`
- `POST /api/runs/:run_id/result`
- `POST /api/events`
- `GET /api/events`
- `POST /api/fingerprint/match`

### 4.3 test_id / scenario_id 규칙
- scenario_id: 내부 참조용(버전관리/리플레이/연관 테이블 조인에 안정적)
- test_id: 표시용(게임 미션 코드). 예: BAS-DEMO-001

현재 MVP 호환:
- test_id는 Run의 필수 값이며, 현재 구현에서는 test_id 하나로 시나리오를 식별한다.
- sim_agent는 이벤트 메시지에 run_start test_id=... 형태로 기록한다.

---

## 5. 문서 업데이트 포인트: "명확한 결과"를 위한 스키마 확장

현재는 Runs/Events 중심이며, Verdict/Evidence가 없다. 이를 추가해 MSF식 모호함을 제거한다.

### 5.1 추가 제안(스키마/개념)
- `steps` (run_id, step_id, name, status)
- `evidence` (evidence_id, run_id, step_id, kind, locator, sha256, created_at)
- `assertions` (assertion_id, step_id, description, required, status, evidence_refs)
- `verdicts` (run_id, step_id, verdict, reason_code, summary)

### 5.2 불변 규칙(Invariants)
- Step 최종 상태는 반드시 `PASS|FAIL`이다.
- `PASS`는 반드시 `evidence_refs`가 1개 이상 존재해야 한다.
- `FAIL`은 반드시 `reason_code`가 존재해야 한다.
- Event(사실 기록)와 Verdict(평가)는 분리한다. (Event에 "성공"을 쓰지 않는다)

### 5.3 Event / Evidence / Verdict JSON 계약(초안)

Event는 타임라인에 기록되는 "사실"이다.

```json
{
  "id": "<uuid>",
  "run_id": "<uuid>",
  "agent_id": "<uuid>",
  "level": "info",
  "message": "run_start test_id=BAS-DEMO-001",
  "ts": "2026-01-01T00:00:00Z"
}
```

Evidence는 Verdict를 뒷받침하는 불변 아티팩트이다.
- locator는 "어디에 있는지"만 나타낸다(예: server 내부 object store 경로). 실제 민감 데이터는 금지.

```json
{
  "evidence_id": "<uuid>",
  "run_id": "<uuid>",
  "step_id": "<uuid>",
  "kind": "telemetry|log|pcap_ref|marker_ref",
  "locator": "evidence://runs/<run_id>/steps/<step_id>/artifact-1.json",
  "sha256": "<hex>",
  "created_at": "2026-01-01T00:00:00Z"
}
```

Verdict는 평가 결과이며 "모호함"을 허용하지 않는다.

```json
{
  "run_id": "<uuid>",
  "step_id": "<uuid>",
  "verdict": "PASS",
  "reason_code": null,
  "summary": "Expected telemetry observed",
  "assertions": [
    {
      "assertion_id": "<uuid>",
      "description": "telemetry event emitted",
      "required": true,
      "status": "PASS",
      "evidence_refs": ["<evidence_id>"]
    }
  ]
}
```

FAIL 예시(정책 거부):

```json
{
  "run_id": "<uuid>",
  "step_id": "<uuid>",
  "verdict": "FAIL",
  "reason_code": "FAIL_POLICY_DENIED",
  "summary": "Out-of-scope target requested",
  "assertions": []
}
```

### 5.4 UI가 보여줘야 하는 것
- 타임라인(Event Stream): 사실 기록
- 평가 화면(Verdict): assertions별 PASS/FAIL + evidence 링크

---

## 6. Game UI 방향 (프로젝트 핵심 목표 반영)

게임 UI는 "실행 능력"을 강화하는 것이 아니라, "이해/학습/운영"을 강화해야 한다.

### 6.1 UI 원칙
- 자유 커맨드 입력(terminal)은 넣지 않는다.
- 운영자 행동은 버튼/선택지(allowlist)로 제한한다.
- 모든 상호작용은 "시나리오 진행" 또는 "리포팅/분석"에만 연결된다.

### 6.2 화면 구성(현재 + 확장)
현재 구현:
- Agents
- Queue Runs
- Events
- Fingerprint

추가 제안:
- Missions/Scenarios: 시나리오 선택, 난이도/목표, 진척도
- Run Detail: Step 트리 + Verdict(Assertion/Evidence)
- Map View(옵션): 네트워크/자산을 "시뮬레이션 엔티티"로 표시 (실자산 혼동 방지)
- Achievements/Skill Tree: 탐지/대응/분석 관점의 성취, 단계별 튜토리얼

### 6.3 Run Detail UX 요구사항(명세)

Run Detail은 "C2 콘솔"의 감성을 가지되, 입력은 안전하게 제한한다.

필수 섹션:
- Header: run_id, scenario_id, test_id(표시용), started_at/ended_at, run_verdict(PASS/FAIL/IN_PROGRESS)
- Step Tree: 단계별 상태(IN_PROGRESS/PASS/FAIL) + reason_code 표시
- Timeline: 해당 run의 events 필터링 뷰 (사실 기록)
- Verdict 탭:
  - assertion 리스트(필수/선택 구분)
  - assertion별 PASS/FAIL
  - evidence_refs 링크(클릭하면 Evidence 탭에서 locator/sha256 확인)
- Evidence 탭:
  - evidence 목록(kind, created_at)
  - locator, sha256 표시
  - 다운로드/열람은 "안전한 포맷"만(예: JSON, 텍스트). 실행 파일/스크립트는 금지.

운영자 입력(Operator Input) 가드레일:
- 자유 텍스트 커맨드 입력 금지.
- 허용되는 입력은 다음만:
  - 시나리오 분기 선택(choice_id)
  - action 카드 승인/실행 요청(action_id)
  - 주석/태그(학습용 메모)

"C2 느낌"을 위한 표현(가능):
- Sessions(=Simulated Endpoints) 리스트
- Jobs(=Actions) 큐
- Operator Log(누가 어떤 승인/중지/분기 선택을 했는지)

"C2로의 악용"을 막기 위한 표현/기능 금지:
- 커맨드 문자열 직접 입력
- 파일 업로드/다운로드
- 네트워크 리스너 생성/외부 연결 설정

---

## 7. 로드맵 (문서 우선, 구현은 안전하게)

### Phase 0: 문서 정합화 (즉시)
- Go/Gin 기반 "C2" 표현을 정리하고, BAS 중심 모델(Scenario/Run/Verdict)로 재정의
- Safety Contract를 문서에 포함

### Phase 1: PASS/FAIL + Evidence 도입
- verdict/evidence/assertions 모델 추가
- UI에 "Run Detail" 뷰 추가(왜 성공/실패했는지 증거로 설명)

### Phase 2: 시나리오 DSL(선언형) + 재현성
- 시나리오 정의 포맷(JSON/YAML) + validator
- 동일 seed에서 재현 가능한 replay

### Phase 3: Game UI 확장
- Missions/Progression/Achievements
- 초보자 친화적 내러티브(미션 로그, 목표, 힌트) 추가

---

## 8. API 계약(초안, 안전 버전)

현재 MVP API(runs/events/agents)는 유지하되, "시나리오"와 "평가 결과"를 표현하는 엔드포인트를 추가한다.

### 8.1 Scenarios
- `GET /api/scenarios`
  - 반환: 시나리오 메타 목록(scenario_id, test_id, title, difficulty, version, estimated_time)
- `GET /api/scenarios/:scenario_id`
  - 반환: steps/actions/assertions 정의(선언형)

### 8.2 Runs (확장)
- `POST /api/runs`
  - 입력(권장): agent_id(또는 simulated endpoint id), scenario_id, params_json
  - 표시용: 서버는 scenario_id로부터 test_id를 결정해 Run에 포함시킨다.
  - MVP 호환: 기존 구현은 test_id가 필수이므로, 전환기에는 (a) test_id만 받는 v1 또는 (b) scenario_id를 받아 내부에서 test_id를 채워 넣는 v2 중 하나로 정리한다.
  - 금지: command string, target address 직접 입력
- `GET /api/runs/:run_id`
  - 반환: run 메타 + run_verdict 요약
- `GET /api/runs/:run_id/steps`
  - 반환: step 트리 + step 상태
- `GET /api/runs/:run_id/verdict`
  - 반환: step별 verdict + assertions + evidence_refs
- `GET /api/runs/:run_id/evidence`
  - 반환: evidence 목록(kind/locator/sha256)

### 8.3 Operator Actions (승인/분기 선택)

운영자 입력은 allowlist 기반이다.

- `POST /api/runs/:run_id/operator-actions`
  - 입력 예시:
  ```json
  {
    "type": "approve_action",
    "action_id": "<action_id>",
    "note": "approve step-2 action"
  }
  ```
  - 입력 예시(분기 선택):
  ```json
  {
    "type": "select_choice",
    "choice_id": "<choice_id>",
    "note": "take stealthy branch"
  }
  ```

금지:
- command 문자열 전달
- 임의 스크립트/바이너리/파일 업로드
- 외부 네트워크 목적지 지정

---

## 9. 레거시 초안 (참고용)

이 문서의 초기 버전에는 Go/Gin 기반 C2, 원격 실행, child agent 등의 내용이 포함되어 있었으나,
현재 방향(BAS + 안전 경계 + 명확한 PASS/FAIL)에 맞추어 **제품 목표에서 제외**한다.
