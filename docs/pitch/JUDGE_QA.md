# JUDGE Q&A — Anticipated Questions & Prepared Answers

---

## 🎯 Ideation Questions

### Q: "What makes this different from GitHub Copilot?"
**A**: "Copilot helps you write *code*. Zit helps you manage *code history*. They're complementary — Copilot generates the code, zit helps you commit it properly, resolve merge conflicts AI-assisted, and understand what went wrong when git fails. No other tool does AI-assisted merge conflict resolution in a terminal UI."

### Q: "Why a TUI and not a web app / VS Code extension?"
**A**: "Three reasons. First, developers live in the terminal — a TUI means zero context switching. Second, TUIs work over SSH, which is critical for Indian developers working on remote servers or cloud VMs. Third, a 5MB native binary uses 100x less memory than an Electron app. We can build a VS Code extension later as a distribution channel, but the core experience is terminal-first."

### Q: "lazygit already exists. Why build another one?"
**A**: "lazygit is an excellent TUI Git client — I admire it. But it has no AI features, no GitHub integration (PRs, Actions, collaborators), no educational features (AI mentor, learning mode), no merge conflict AI resolution, and no safety labels on operations. Zit is what lazygit would be if it was built with AI and education as core principles."

---

## 🔧 Technical Questions

### Q: "Why Rust instead of Python/Go?"
**A**: "Three concrete reasons. First, Rust compiles to a single native binary with zero runtime dependencies — users don't need to install Python or Node. Second, Rust's ownership system prevents the data races we'd otherwise face with our multi-threaded AI client. Third, the binary is 5MB and uses <10MB RAM — important for developers on older hardware. The trade-off is longer compile times, but that's a developer cost, not a user cost."

### Q: "How do you handle AI hallucinations?"
**A**: "We limit the blast radius. AI suggestions are always *advisory*, never *auto-executed*. When AI suggests a commit message, the user reviews and edits it. When AI recommends a merge resolution, the user applies it manually. We also structure our prompts with strict output formats (RECOMMENDATION: ACCEPT_CURRENT | MERGE_BOTH) so the AI produces actionable, verifiable output rather than open-ended prose. And all AI responses have a 1024 token cap to prevent rambling."

### Q: "What's the latency like?"
**A**: "Two parts. Git operations take 50-200ms (local, fast). AI requests take 3-15 seconds depending on the provider. That's why we use background threads — the UI stays responsive during AI calls and shows a loading indicator. The user can continue working with git while the AI thinks."

### Q: "Why do you shell out to git instead of using libgit2?"
**A**: "Compatibility. By calling the real git binary, we automatically support the user's git config, aliases, hooks, credential helpers, and any custom setup. libgit2 reimplements only a subset of git's features and has known behavioral differences. The trade-off is process spawning overhead, but git commands return in <200ms on typical repos, so it's not a bottleneck."

### Q: "How do you handle the AI provider going down?"
**A**: "Graceful degradation. All 14 TUI views work without AI. When AI is unavailable, git error messages show the raw output instead of an AI explanation. The AI Mentor panel shows setup instructions. We also support 5 independent providers — if OpenAI is down, the user can switch to Anthropic, Ollama, or any other provider. And Ollama runs locally, so it never has downtime."

### Q: "What's the test coverage?"
**A**: "205 automated tests. 178 Rust tests (unit + integration) and 27 Python Lambda tests. The CI runs on 3 operating systems (Linux, macOS, Windows) on every push. Tests cover config parsing, git output parsing, AI client retry logic, request validation, and UI state transitions. We can't test the TUI rendering automatically (it's visual), but every state machine transition is tested."

---

## 🌍 Impact Questions

### Q: "Do you have any users?"
**A**: "The project is open source on GitHub (MIT license). We have a Homebrew formula for easy installation. As this is a hackathon project, we're in the early adoption phase, but the README, CI/CD, and distribution infrastructure are all production-ready. The project is designed for zero-friction onboarding — `brew install zit` and you're running in 30 seconds."

### Q: "How is this specific to India / Bharat?"
**A**: "Four ways. First, India graduates 800,000+ CS students annually — most struggle with Git because bootcamps spend 2 hours on it. Second, many Indian developers work remotely via SSH on low-bandwidth connections — a TUI works perfectly there, unlike web-based tools. Third, Ollama support means zero API costs — critical for developers in India where $20/month for a tool is a real barrier. Fourth, our AWS backend is deployed in `ap-south-1` (Mumbai) — single-digit latency for Indian users."

### Q: "Who exactly is your target user?"
**A**: "Developers in their first 2 years of professional work. They know enough Git to be dangerous but not enough to be safe. Specifically: bootcamp graduates joining startups, CS students contributing to open source for the first time, and team leads whose junior developers create merge conflict chaos every sprint."

---

## 💰 Business Questions

### Q: "How do you monetize an open-source CLI tool?"
**A**: "Open-core model. The core product is free forever. Revenue comes from: (1) Managed AI backend — $5-15/month so users don't need AWS accounts, and (2) Enterprise features — SSO, audit logs, team analytics at $25/user/month. This is the same model that works for GitLab ($500M ARR), Supabase, and PostHog."

### Q: "What's your competitive moat?"
**A**: "Three things. First, multi-provider AI — we're the only Git tool that supports 5 AI providers including local Ollama. Second, the educational angle — our prompts are designed to teach, not just answer. This builds long-term loyalty. Third, being Rust-native — our binary is faster and smaller than any Electron competitor, and the codebase is easier to maintain than Python alternatives."

### Q: "What would you do in the next 90 days?"
**A**: "Three priorities. First, ship a VS Code sidebar extension that embeds zit's AI features — reaching 76% of developers who use VS Code. Second, publish tutorial content targeting 'how to resolve merge conflict' SEO queries — 180K monthly searches. Third, pilot with 2-3 Indian coding bootcamps (Scaler, Masai School) to validate the educational use case and get testimonials."

### Q: "What's the cost structure?"
**A**: "Near-zero. Lambda free tier covers 1M requests/month. Bedrock costs ~$0.003 per AI request. GitHub Actions is free for open-source repos. Our only hard cost is the ~$3/month for the API Gateway + CloudWatch. At 1,000 daily active users making 5 AI requests each, our AI cost is ~$15/month. We can run for years on $50/month."

---

## 🎤 Demo Tips

### What to Demo (In Order)
1. **Dashboard** (10 sec) — Show the at-a-glance repo status
2. **AI Commit Suggestion** (30 sec) — Stage a file, press Ctrl+G, show AI-generated commit message
3. **AI Error Explanation** (20 sec) — Trigger a git error intentionally, show auto-explanation
4. **AI Merge Conflict Resolution** (30 sec) — Pre-prepare a merge conflict, show AI recommendation
5. **GitHub PR Management** (20 sec) — Show listing/viewing PRs from the terminal
6. **Health Check** (10 sec) — Show AI backend connectivity test

### What NOT to Demo
- Multiple AI providers (talk about it, don't switch live)
- Complex Git operations (rebasing, bisecting) — too much context needed
- Homebrew installation (just mention it)
- AWS deployment (just mention SAM template)

### Demo Safety Net
- Pre-stage a git repo with modified files ready to commit
- Pre-create a merge conflict that's ready to resolve
- Have a backup video recording in case live demo fails
- Know the keyboard shortcuts: `s`=staging, `c`=commit, `a`=AI Mentor, `Ctrl+G`=suggest commit, `?`=help
