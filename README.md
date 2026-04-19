# Shadow Protocol — Breach & Attack Simulation (BAS)

실제 에이전트 실행, 서버 기반 단계 제어, 게임 스타일 전술 UI를 갖춘 침해 및 공격 시뮬레이션(BAS) 플랫폼입니다. 보안 교육 및 인가된 보안 테스트를 목적으로 설계되었습니다.

## 아키텍처

```
┌──────────────┐       HTTP/JSON        ┌──────────────┐
│   Agent       │ ◄──────────────────►  │   Server     │
│  (Rust)       │  등록 / 하트비트      │  (Rust/Axum) │
│  Windows +    │  실행 폴링 / 결과     │  SQLite DB   │
│  Linux        │  증거 / 이벤트        │  시나리오     │
└──────────────┘                        └──────┬───────┘
                                               │
                                        REST API (JSON)
                                               │
                                        ┌──────┴───────┐
                                        │     UI       │
                                        │  Electron +  │
                                        │  React/Vite  │
                                        └──────────────┘
```

| 컴포넌트 | 기술 스택 | 설명 |
|----------|-----------|------|
| **Server** | Rust, Axum, SQLite (sqlx) | REST API, 시나리오 엔진, 업적 시스템, AI 스크립트 생성기 |
| **Agent** | Rust (크로스 플랫폼) | 실제 OS 실행기, 스캐너, 회피 모듈 |
| **UI** | Electron, React 19, Vite, Framer Motion | 게임 스타일 전술 인터페이스 |

## 사전 요구사항

- **Rust** 1.85+ (Cargo 포함)
- **Node.js** 18+ (npm 포함)
- (선택) **Electron** — npm을 통해 자동 설치됨

## 빠른 시작

### 1. 서버 시작

```bash
cd server
cargo run
```

서버는 첫 실행 시 `red-sim.db` (SQLite)를 자동 생성하며, `http://127.0.0.1:3000`에서 수신 대기합니다.

### 2. 에이전트 시작

```bash
cd Agent
cargo run
```

에이전트가 서버에 등록되고, 하트비트를 전송하며, 대기 중인 실행을 폴링합니다.

### 3. UI 시작

```bash
cd ui
npm install
npm run dev
```

Vite 개발 서버(`:5173`)와 Electron이 함께 시작됩니다. Electron 창을 사용하거나 `http://localhost:5173`에 접속하세요.

### 4. 시나리오 실행

1. UI에서 **Operations** 탭을 엽니다
2. 시나리오와 대상 에이전트를 선택합니다
3. **DEPLOY**를 클릭하여 실행을 생성합니다
4. **Missions** 탭에서 에이전트가 실시간으로 단계를 실행하는 것을 확인합니다

## 환경 변수 설정

### 서버 환경 변수

| 변수 | 기본값 | 설명 |
|------|--------|------|
| `DATABASE_URL` | `sqlite:red-sim.db` | SQLite 데이터베이스 경로 |
| `SCENARIOS_PATH` | `data/scenarios` | 시나리오 JSON 파일이 있는 디렉토리 |
| `FINGERPRINT_RULES_PATH` | — | 핑거프린트 규칙 JSON 경로 |
| `OPERATOR_TOKEN` | — | 운영자 API 인증용 Bearer 토큰. 미설정 시 인증 비활성화 (개발 모드) |
| `AI_KEY_MASTER` | — | AI API 키 AES-256-GCM 암호화용 비밀번호. 미설정 시 평문 저장 |
| `CORS_PERMISSIVE` | `false` | `true`로 설정 시 CORS 전체 허용 (개발용) |
| `LAUNCH_UI` | `false` | `true`로 설정 시 UI 개발 서버 자동 실행 |

### 에이전트 환경 변수

**빌드 타임** (바이너리에 내장) 또는 **런타임**에 설정할 수 있습니다:

| 항목 | 빌드 타임 변수 | 기본값 | 설명 |
|------|---------------|--------|------|
| 서버 URL | `AGENT_SERVER_URL` | `http://127.0.0.1:3000` | C2 서버 주소 |
| 에이전트 GUID | `AGENT_GUID` | `dev-agent-no-guid` | 고유 에이전트 식별자 |
| 슬립 주기 | `AGENT_SLEEP_SEC` | `5` | 하트비트/폴링 주기 (초) |

빌드 타임 내장 예시:

```bash
cd Agent
AGENT_GUID=prod-agent-001 AGENT_SERVER_URL=https://c2.example.com AGENT_SLEEP_SEC=10 cargo build --release
```

Windows 환경 (PowerShell):

```powershell
cd Agent
$env:AGENT_GUID="prod-agent-001"
$env:AGENT_SERVER_URL="https://c2.example.com"
$env:AGENT_SLEEP_SEC="10"
cargo build --release
```

### UI 환경 변수

| 변수 | 기본값 | 설명 |
|------|--------|------|
| `VITE_SERVER_URL` | `http://127.0.0.1:3000` | 서버 API 엔드포인트 |

## 프로젝트 구조

```
├── server/                    # Rust HTTP API 서버
│   ├── src/
│   │   ├── main.rs           # 진입점, 라우트 정의
│   │   ├── handlers.rs       # 모든 API 핸들러
│   │   ├── models.rs         # 데이터 모델 (Agent, Run, Step 등)
│   │   ├── db.rs             # DB 초기화 및 마이그레이션
│   │   ├── scenarios.rs      # 시나리오 엔진 및 카탈로그
│   │   ├── fingerprint.rs    # 오프라인 배너 핑거프린트 매칭
│   │   ├── ai.rs             # AI 스크립트 생성기 (Claude/OpenAI/Gemini)
│   │   ├── auth.rs           # 운영자 토큰 인증 미들웨어
│   │   └── crypto.rs         # AES-256-GCM API 키 암호화
│   └── data/
│       ├── scenarios/        # 시나리오 JSON 정의 파일
│       └── fingerprint_rules.sample.json
│
├── Agent/                     # Rust 크로스 플랫폼 에이전트
│   ├── build.rs              # 빌드 타임 설정 내장
│   ├── src/
│   │   ├── main.rs           # 에이전트 루프: 등록 → 폴링 → 실행 → 보고
│   │   ├── config.rs         # 내장 설정 (GUID, 서버 URL, 슬립)
│   │   ├── lib.rs            # 라이브러리 루트
│   │   ├── executor/         # OS 실행 모듈
│   │   │   ├── windows/      # COM, PowerShell, Process, Registry, Fileless
│   │   │   └── linux/        # Memfd, Shell, Syscall
│   │   ├── scanner/          # 네트워크 정찰
│   │   │   ├── port.rs       # TCP 포트 스캐너 + 배너 수집
│   │   │   ├── banner.rs     # 배너 분석
│   │   │   └── active_directory.rs  # AD/LDAP 스캐너
│   │   ├── transport/        # 서버 통신
│   │   │   ├── http.rs       # HTTP 전송 (인증 포함)
│   │   │   └── protocol.rs   # 와이어 프로토콜 타입
│   │   └── evasion/          # 회피 기법
│   │       ├── anti_analysis.rs     # 안티디버깅, VM 탐지
│   │       └── string_obfuscation.rs # 문자열 난독화
│   └── Cargo.toml
│
├── ui/                        # Electron + React UI
│   ├── src/
│   │   ├── App.tsx           # 메인 레이아웃 (게임 스타일 네비게이션)
│   │   ├── main.tsx          # React 진입점
│   │   ├── lib/
│   │   │   ├── api.ts        # 서버 API 클라이언트
│   │   │   └── types.ts      # TypeScript 타입 정의
│   │   ├── panels/           # 뷰 패널
│   │   │   ├── MissionsPanel.tsx      # 시나리오 선택 및 배포
│   │   │   ├── AgentsPanel.tsx        # 에이전트 로스터 및 관리
│   │   │   ├── GroupsPanel.tsx        # 분대/그룹 관리
│   │   │   ├── RunsPanel.tsx          # 미션 큐 및 실행 목록
│   │   │   ├── RunDetailPanel.tsx     # 단계별 실행 진행 상황
│   │   │   ├── EventsPanel.tsx        # 전투 로그 (이벤트)
│   │   │   ├── FingerprintPanel.tsx   # 정보 센터 (핑거프린트)
│   │   │   ├── BriefingPanel.tsx      # 미션 브리핑
│   │   │   ├── MapPanel.tsx           # 네트워크 맵
│   │   │   ├── AchievementsPanel.tsx  # 업적 트래커
│   │   │   └── AIScriptPanel.tsx      # AI 시나리오 생성기
│   │   ├── components/       # 재사용 게임 UI 컴포넌트
│   │   ├── hooks/            # 사운드 이펙트 및 게임 오디오
│   │   └── styles/theme.css  # 사이버펑크 테마
│   └── package.json
│
└── README.md
```

## 작동 방식

### 단계 생명주기

```
LOCKED → READY → COMPLETED / FAILED
```

1. 서버가 시나리오에서 순차적 **단계(step)**를 가진 **실행(run)**을 생성합니다
2. 첫 번째 단계는 `READY`, 나머지는 `LOCKED` 상태로 시작합니다
3. 에이전트가 대기 중인 실행을 폴링하고, 단계를 가져와 `READY` 단계를 실행합니다
4. 에이전트가 결과와 함께 `complete_step`을 호출합니다
5. 서버가 다음 `LOCKED` 단계를 `READY`로 전환합니다
6. 선택지 기반 단계는 운영자가 UI에서 분기를 선택하면 `READY`로 전환됩니다

### 에이전트 실행 흐름

```
등록 → 승인 (UI) → 대기 실행 폴링 → 단계 조회
    → READY 단계 찾기 → 실행 → POST complete_step
    → 서버가 다음 단계 잠금 해제 → 모든 단계 완료까지 반복
    → POST 실행 결과 → 하트비트 루프 계속
```

### 에이전트 빌드 시스템

서버에서 설정이 내장된 커스텀 에이전트 바이너리를 빌드할 수 있습니다:

```bash
# API를 통한 빌드
POST /api/agents/build
{
  "target_platform": "windows-x86_64",
  "server_url": "https://c2.example.com",
  "sleep_sec": 10
}
```

빌드 시 `build.rs`를 통해 고유 GUID, 서버 URL, 슬립 주기가 바이너리에 컴파일 타임으로 주입됩니다.

## 에이전트 실행기

### Windows
| 실행기 | 설명 |
|--------|------|
| **COM** | `WScript.Shell`, `MMC20.Application`을 통한 COM 자동화 |
| **PowerShell** | 파이프 캡처가 포함된 PowerShell 명령 실행 |
| **Process** | Win32 `CreateProcessW` 직접 호출 |
| **Registry** | Windows 레지스트리 읽기/쓰기/삭제 |
| **Fileless** | `VirtualAlloc` + `CreateThread`를 통한 인메모리 셸코드 실행 |

### Linux
| 실행기 | 설명 |
|--------|------|
| **Memfd** | `memfd_create` + `fexecve`를 통한 메모리 전용 ELF 실행 |
| **Syscall** | 익명 fd에서의 `SYS_execveat` 직접 호출 |
| **Shell** | 표준 `/bin/sh -c` 명령 실행 |

### 크로스 플랫폼
| 모듈 | 설명 |
|------|------|
| **포트 스캐너** | TCP 연결 스캔 + 배너 수집 (상위 1000개 포트) |
| **배너 분석기** | 배너에서 서비스/버전 식별 |
| **AD/LDAP** | Active Directory 정찰 |

## API 레퍼런스

### 인증

`OPERATOR_TOKEN`이 설정된 경우, 운영자용 엔드포인트에 다음 헤더가 필요합니다:
```
Authorization: Bearer <token>
```

에이전트용 엔드포인트(등록, 하트비트, 폴링, 결과 제출)는 운영자 인증이 불필요합니다.

### 시나리오
| 메서드 | 엔드포인트 | 설명 |
|--------|----------|------|
| `GET` | `/api/scenarios` | 전체 시나리오 목록 |
| `GET` | `/api/scenarios/:id` | 시나리오 상세 조회 |
| `POST` | `/api/scenarios/validate` | 시나리오 JSON 검증 |

### 에이전트
| 메서드 | 엔드포인트 | 인증 | 설명 |
|--------|----------|------|------|
| `POST` | `/api/agents/register` | Agent | 신규 에이전트 등록 |
| `POST` | `/api/agents/:id/heartbeat` | Agent | 하트비트 전송 |
| `GET` | `/api/agents/list` | Operator | 에이전트 목록 (최대 200개) |
| `GET` | `/api/agents/pending` | Operator | 승인 대기 에이전트 목록 |
| `POST` | `/api/agents/:id/approve` | Operator | 에이전트 승인 |
| `POST` | `/api/agents/:id/block` | Operator | 에이전트 차단 |
| `POST` | `/api/agents/build` | Operator | 에이전트 바이너리 빌드 |
| `GET` | `/api/agents/builds` | Operator | 빌드 목록 |

### 실행 및 단계
| 메서드 | 엔드포인트 | 인증 | 설명 |
|--------|----------|------|------|
| `POST` | `/api/runs` | Operator | 실행 생성 |
| `GET` | `/api/runs` | Operator | 실행 목록 |
| `GET` | `/api/runs/:id` | Operator | 실행 상세 조회 |
| `GET` | `/api/runs/:id/steps` | Operator | 실행 단계 조회 |
| `GET` | `/api/runs/:id/verdict` | Operator | 단계 판정 조회 |
| `POST` | `/api/runs/:id/replay` | Operator | 완료된 실행 재생 |
| `GET` | `/api/runs/pending/:agent_id` | Agent | 대기 실행 폴링 |
| `POST` | `/api/runs/:run_id/result` | Agent | 실행 결과 제출 |
| `POST` | `/api/runs/:run_id/steps/:step_id/complete` | Agent | 단계 완료 보고 |

### 그룹
| 메서드 | 엔드포인트 | 설명 |
|--------|----------|------|
| `GET` | `/api/groups` | 그룹 목록 (최대 200개) |
| `POST` | `/api/groups` | 그룹 생성 |
| `POST` | `/api/groups/:id/assign` | 그룹에 에이전트 배정 |
| `POST` | `/api/groups/:id/runs` | 그룹 전체 에이전트에 실행 생성 |

### 증거 및 이벤트
| 메서드 | 엔드포인트 | 인증 | 설명 |
|--------|----------|------|------|
| `POST` | `/api/evidence` | Agent | 증거 제출 |
| `POST` | `/api/events` | Agent | 이벤트 제출 |
| `GET` | `/api/events` | Operator | 이벤트 목록 |
| `GET` | `/api/runs/:id/events` | Operator | 실행별 이벤트 목록 |

### AI 스크립트 생성기
| 메서드 | 엔드포인트 | 설명 |
|--------|----------|------|
| `GET` | `/api/ai/accounts` | AI 계정 목록 |
| `POST` | `/api/ai/accounts` | AI 계정 추가 (Claude/OpenAI/Gemini) |
| `POST` | `/api/ai/conversations` | 대화 시작 |
| `POST` | `/api/ai/conversations/:id/chat` | 메시지 전송 |
| `POST` | `/api/ai/conversations/:id/save-scenario` | 생성된 시나리오 디스크에 저장 |

### 기타
| 메서드 | 엔드포인트 | 설명 |
|--------|----------|------|
| `POST` | `/api/fingerprint/match` | 오프라인 배너 핑거프린트 매칭 |
| `GET` | `/api/achievements` | 업적 목록 |
| `POST` | `/api/achievements/check` | 업적 진행도 확인/갱신 |

## 보안 기능

- **운영자 인증**: Bearer 토큰 미들웨어로 에이전트/운영자 API 접근 분리
- **API 키 암호화**: AI 제공자 키를 AES-256-GCM으로 암호화 (`AI_KEY_MASTER` 필요)
- **에이전트 승인**: 신규 에이전트는 실행 수신 전 수동 승인 필요
- **빌드 동시실행 방지**: Semaphore로 동시 에이전트 빌드 방지 (429 반환)
- **입력 검증**: 상태값 허용 목록, 경로 탐색 방지, 시나리오 ID 유효성 검사
- **DB 트랜잭션**: 실행 + 단계 생성을 트랜잭션으로 원자성 보장
- **HTTPS 경고**: 에이전트가 비로컬 주소에 평문 HTTP로 연결 시 경고 로그 출력

## 테스트

```bash
# 서버
cd server && cargo test

# 에이전트
cd Agent && cargo test

# UI
cd ui && npm run lint && npm run build
```

## 시나리오 형식

시나리오는 `data/scenarios/` 디렉토리의 JSON 파일로 정의합니다:

```json
{
  "scenario_id": "example-scenario",
  "test_id": "BAS-001",
  "title": "예제 시나리오",
  "description": "단계 실행을 시연합니다",
  "category": "discovery",
  "mitre_ids": ["T1057"],
  "steps": [
    {
      "step_id": "step-1",
      "name": "프로세스 탐색",
      "executor": "powershell",
      "command": "Get-Process | Select-Object -First 10",
      "assertions": [
        {
          "description": "명령이 성공해야 합니다",
          "type": "exit_code",
          "kind": "equals",
          "contains": "0",
          "required": true
        }
      ]
    }
  ]
}
```

## 안전 주의사항

이 플랫폼은 프로세스 생성, COM 자동화, 레지스트리 조작, 메모리 인젝션, 직접 시스콜 등 실제 OS 실행 기능을 포함하고 있습니다. **인가된 보안 테스트 및 교육 목적으로만 사용**하도록 설계되었습니다.

- 명시적 인가 없이 운영 시스템에 에이전트를 배포하지 마세요
- 엄격한 접근 통제 없이 컴파일된 에이전트 바이너리를 배포하지 마세요
- 로컬이 아닌 환경에서는 반드시 HTTPS와 운영자 인증을 사용하세요
- 실행 전 시나리오 정의를 반드시 검토하세요
