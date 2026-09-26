# TraceForge AI — ContractGuard

**From Bug Report to Verified Fix**

A production-oriented developer workflow application that orchestrates API contract investigation, IBM Bob-assisted root cause analysis, human approval, code correction, and independent verification.

## Quick Start

### Prerequisites

- Rust 1.70+ and Cargo
- Node.js 18+ and npm
- (Optional) Docker and Docker Compose

### Backend

```powershell
# Windows PowerShell
cd backend
copy ..\\.env.example .env
cargo build
cargo run
```

```bash
# Linux / macOS
cd backend
cp ../.env.example .env
cargo build
cargo run
```

Backend listens at `http://127.0.0.1:8080`

### Frontend

```powershell
cd frontend
npm install
npm run dev
```

Frontend opens at `http://localhost:5173`

### Docker Compose

```bash
docker compose up
```

## Architecture

```
ContractGuard
├── Rust Axum backend   — workflow state, DB, verification engine
├── React frontend      — investigation dashboard
├── IBM Bob IDE         — investigation and implementation
└── UploadLab           — sample project with deliberate defects
```

## Running Tests

### Backend

```bash
cd backend
cargo test
```

### Frontend

```bash
cd frontend
npm test
```

### Sample Project (UploadLab)

```bash
# Expected failures: BUG-001 and BUG-002
cd sample_project/broken/backend
cargo test

cd sample_project/broken/frontend
npm install && npm test
```

## Repository Structure

```
devflow-ai/
├── backend/               Rust Axum API
├── frontend/              React TypeScript dashboard
├── sample_project/        UploadLab broken fixture
├── bob/                   IBM Bob prompts and schemas
├── runs/                  Investigation workspaces (gitignored)
├── demo_data/             Demo investigation artifacts
├── submission/            Hackathon submission materials
└── scripts/               Verification scripts
```

## Investigation Workflow

1. Create investigation on the dashboard
2. ContractGuard creates isolated workspace from sample_project/broken
3. Open workspace in IBM Bob, run investigation
4. Bob writes structured artifacts to `runs/{workspace_id}/bob_artifacts/`
5. Click "Sync Results" in ContractGuard
6. Review findings, approve fix plan
7. Bob implements corrections in isolated workspace
8. ContractGuard runs independent verification tests
9. Review evidence graph and final report

## License

For demonstration purposes — IBM Bob 2.0 Hackathon submission.
