# ⚡ ZIT — JUDGE PRESENTATION CHEATSHEET ⚡
> **AI-Powered Git Mentor for the Terminal** | AI for Bharat Hackathon

---

## 🏠 30-Second Elevator Pitch
"Zit is an AI-powered Git tool that lives in your terminal. When you hit a merge conflict, AI analyzes both sides and tells you which changes to keep. When a git command fails, AI explains the error in plain English. When you commit, AI writes proper conventional messages from your diff. Think of it as having a senior developer looking over your shoulder — but it's a 5MB binary that runs anywhere, supports 5 AI providers including free local AI, and has 205 automated tests."

---

## 🧠 Key Numbers (Memorize These)

| Stat | Value |
|------|-------|
| Lines of Rust code | ~8,000+ |
| TUI views | 14 |
| AI providers | 5 (Bedrock, OpenAI, Anthropic, OpenRouter, Ollama) |
| AI features | 9 (explain, error, recommend, commit, learn, review, merge resolve, merge strategy, gitignore) |
| Automated tests | 205 (178 Rust + 27 Python) |
| CI platforms | 3 (Linux, macOS, Windows) |
| Binary size | ~5 MB |
| RAM usage | < 10 MB |
| Git commands used | 30+ |
| GitHub API endpoints | 15+ |
| Config file | ~/.config/zit/config.toml |
| Dependencies (direct) | 15 Rust crates |
| Lambda request types | 8 |
| CloudWatch alarms | 4 |

---

## 🏗️ Architecture at a Glance

```
┌─────────────────────────┐
│   zit (Rust TUI Binary) │ ← 14 views, immediate-mode rendering
│   ratatui + crossterm    │
└────┬──────┬──────┬──────┘
     │      │      │
     ▼      ▼      ▼
  ┌──────┐ ┌────┐ ┌──────────────────┐
  │ git  │ │ GH │ │  AI Provider     │
  │ CLI  │ │API │ │ (5 options)      │
  └──────┘ └────┘ └──────────────────┘
                     │
        ┌────────────┼────────────┐
        ▼            ▼            ▼
   ┌─────────┐  ┌─────────┐  ┌────────┐
   │ Lambda  │  │ Direct  │  │ Ollama │
   │ +Bedrock│  │OpenAI/  │  │ (Local)│
   │ (AWS)   │  │Anthropic│  │  FREE  │
   └─────────┘  └─────────┘  └────────┘
```

---

## 🔑 Killer Lines (Use in Conversation)

### About the Problem
- "72% of developers report anxiety using advanced Git features"
- "'How to resolve merge conflict' gets 180,000+ monthly Google searches"
- "Bootcamp grads know 5 git commands and are afraid of the other 95"

### About Your Solution
- "AI teaches you Git instead of doing it for you — mentor, not crutch"
- "5MB binary, 10MB RAM — vs 500MB for GitKraken"
- "Works over SSH on 2G connections in tier-3 Indian cities"
- "Ollama support = completely free, completely private AI — no API costs for Indian devs"

### About Technical Depth
- "205 automated tests, CI on 3 OSes, infrastructure-as-code — this isn't a prototype"
- "Background threads + mpsc channels keep the UI responsive during 30s AI calls"
- "Rust's type system catches bugs at compile time — zero runtime crashes in testing"
- "Shell-based Git = 100% compatibility with user's hooks, aliases, credentials"

### About Business
- "Open-core model — same playbook as GitLab ($500M ARR)"
- "Our AI costs $0.003 per request — we can serve 1,000 users for $15/month"
- "India adds 800,000 CS grads/year — they all struggle with Git"

---

## 🎯 Rubric Cheat Map

| Rubric (weight) | Your strongest point |
|-----------------|---------------------|
| **Ideation (30%)** | Only tool combining TUI + AI mentorship + multi-provider + education. Not simplifying Git — making it safe + educational. |
| **Technical (30%)** | Rust + serverless Lambda + 5 AI providers + 205 tests + cross-platform CI + IaC. Production-grade, not a demo. |
| **Impact (20%)** | 5.8M Indian devs, 800K new CS grads/year. Works over SSH, free local AI via Ollama, deployed in Mumbai region. |
| **Business (20%)** | Open-core model. $0 to serve free tier. Managed AI backend + enterprise features for revenue. |

---

## ⌨️ Critical Keyboard Shortcuts (for Demo)

| Key | Action |
|-----|--------|
| `d` | Dashboard |
| `s` | Staging view (select files, space to stage) |
| `c` | Commit view |
| `Ctrl+G` | AI commit message suggestion (in commit view) |
| `b` | Branch management |
| `l` | Timeline / log |
| `a` | AI Mentor panel |
| `g` | GitHub integration |
| `m` | Merge resolve |
| `?` | Help overlay |
| `q` | Quit / back |
| `Esc` | Close popup |

---

## 🚨 If Things Go Wrong During Demo

| Problem | Fix |
|---------|-----|
| Terminal too small | Resize or `Cmd+minus` to shrink font |
| AI times out | "That's why we have retry logic — but let me show you the cached response" |
| Git error during demo | "Perfect! This is exactly what zit handles — watch the AI explain it" |
| Forget a shortcut | Press `?` for help overlay |
| Nervous | Remember: you built this. You have 8,000+ lines of Rust and 205 tests. Own it. |
