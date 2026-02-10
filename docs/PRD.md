# Product Requirements Document: zit

**Version:** 1.0

**Last Updated:** February 10, 2026

**Status:** MVP Specification

---

## Executive Summary

**zit** is an AI-powered, terminal-based Git and GitHub CLI tool that transforms Git from a command-memorization challenge into an intuitive, visual learning experience. By combining a rich TUI (Terminal User Interface) with intelligent AI mentorship, zit helps developers—especially beginners—learn Git faster, work more safely, and build confidence through guided workflows and real-time explanations.

### Vision

Make Git accessible and understandable for every developer by replacing error-prone commands with safe, visual workflows that teach as they work.

### Mission

Empower developers to master Git through an intelligent terminal interface that prevents mistakes, explains concepts in plain English, and builds intuition about version control.

---

## Problem Statement

### Current Pain Points

1. **Steep Learning Curve** : Git's complex command syntax and abstract concepts (HEAD, detached HEAD, rebase, cherry-pick) overwhelm beginners
2. **Fear of Mistakes** : Developers avoid powerful Git features due to fear of losing work or breaking history
3. **Cryptic Error Messages** : Git errors are technical and rarely explain what went wrong or how to fix it
4. **Hash-Based Navigation** : Navigating history via SHA hashes is unintuitive and error-prone
5. **Context Switching** : Developers leave the terminal to use GUI tools or search StackOverflow for help
6. **Lack of Guidance** : Git doesn't recommend safe vs. risky operations or explain trade-offs

### Target Users

* **Primary** : Junior developers (0-2 years experience) learning Git fundamentals
* **Secondary** : Intermediate developers (2-5 years) wanting to master advanced features safely
* **Tertiary** : Senior developers seeking productivity gains and reduced cognitive load

---

## Product Overview

### Core Philosophy

**"Terminal-Native Learning Assistant"**

zit is not just a Git UI wrapper—it's a learning tool that:

* Shows the current state visually
* Explains what's happening in plain English
* Guides users toward safe, best-practice workflows
* Provides guardrails without removing power
* Teaches Git concepts through use, not documentation

### Key Differentiators

1. **AI Mentor Integration** : Context-aware AI explains repo state, errors, and recommends actions
2. **Opinionated Safety** : Defaults to non-destructive operations with clear warnings for risky actions
3. **Visual Timeline Navigation** : Browse history by message/time/branch instead of hashes
4. **Terminal-Native** : No context switching; everything happens in the terminal developers already use
5. **Shell-Based Truth** : Uses actual Git commands as source of truth, not reimplementation

---

## Core Features (MVP)

### 1. Repository Dashboard

 **User Story** : As a developer, I want to see my repository's current state at a glance so I can understand where I am and what needs attention.

 **Requirements** :

* Display current branch, commit status, and working tree state
* Show staged vs. unstaged changes with file counts
* Indicate ahead/behind status relative to remote
* Display recent commit summary (last 5 commits)
* Show stash count if present
* Visual indicators for clean/dirty state, conflicts, detached HEAD

 **Acceptance Criteria** :

* Dashboard loads in <500ms for typical repos
* All information is accurate and reflects current Git state
* Visual hierarchy clearly distinguishes critical info from context
* Refreshes automatically when repo state changes

---

### 2. Smart Staging (Interactive)

 **User Story** : As a developer, I want to stage changes visually and selectively so I can create focused, logical commits.

 **Requirements** :

* Display all modified/new/deleted files in a scrollable list
* Show diff preview for selected files (side panel or toggle view)
* Stage/unstage individual files with keyboard shortcuts (Space, Enter)
* Stage/unstage all with single command (Ctrl+A)
* Support partial staging (hunk-level) for files with multiple changes
* Filter by file type or status (modified, new, deleted)
* Search files by name

 **Acceptance Criteria** :

* Navigating files is instant and responsive
* Diff preview updates in real-time as selection changes
* Partial staging works for at least 90% of common diff scenarios
* File count indicator shows staged vs. total modified
* Visual distinction between staged/unstaged states

---

### 3. Guided Commits

 **User Story** : As a developer, I want help writing good commit messages so I can maintain a clean, understandable history.

 **Requirements** :

* Multi-line commit message editor within TUI
* Show staged files before committing
* Provide commit message templates/examples
* Validate message format (warn on missing subject, overly long lines)
* Optional AI suggestions for commit message based on staged changes
* Preview commit before finalizing
* Amend previous commit option

 **Acceptance Criteria** :

* Message editor supports standard Vim/Emacs-like keybindings
* Validation warnings are helpful, not blocking (can override)
* AI suggestions appear within 2 seconds
* Commit operation fails gracefully with clear error if Git rejects it

---

### 4. Visual Branching (TUI)

 **User Story** : As a developer, I want to create, switch, and manage branches through a visual interface so I can organize my work without memorizing commands.

 **Requirements** :

* List all local and remote branches in a tree structure
* Show current branch with visual highlight
* Create new branch from current HEAD or selected commit
* Switch branches with confirmation if uncommitted changes exist
* Delete branches (local only in MVP) with safety checks
* Rename current branch
* Show branch metadata (last commit, author, date)
* Track relationship to remote branches (upstream info)

 **Acceptance Criteria** :

* Branch list loads in <1s even for repos with 50+ branches
* Current branch is always visually distinct
* Creating/switching branches updates dashboard immediately
* Cannot delete current branch (prevented with helpful message)
* Warns before switching if uncommitted changes would be lost

---

### 5. Commit Timeline & Visual Graph

 **User Story** : As a developer, I want to navigate commit history visually by message and time so I can find and understand past changes without memorizing hashes.

 **Requirements** :

* Display commit graph showing branch structure and merges
* Scrollable timeline with commit messages, authors, dates
* Search commits by message text, author, date range
* Filter by branch
* Select commit to view full details (message, diff, files changed)
* Visual indicators for: HEAD, current branch, tags, merge commits
* Navigate using arrow keys or Vim-like keybindings
* Copy commit hash to clipboard for external use

 **Acceptance Criteria** :

* Timeline renders 100+ commits without performance degradation
* Graph accurately represents branch topology
* Search returns results in <500ms for repos with 1000+ commits
* Commit details view shows complete diff with syntax highlighting
* Timeline updates when new commits are created

---

### 6. Safe Time Travel (Reset & Restore)

 **User Story** : As a developer, I want to safely restore previous states or undo mistakes so I can experiment without fear of losing work.

 **Requirements** :

* Navigate to any commit in timeline and create branch from it
* Soft reset to previous commit (keeps changes in working tree)
* Mixed reset to previous commit (unstages changes)
* Hard reset with explicit confirmation and reflog backup mention
* Restore individual files from previous commits
* Reflog viewer for recovering "lost" commits
* Clear explanations of what each operation does before executing
* Warn about uncommitted work before destructive operations

 **Acceptance Criteria** :

* All reset operations require explicit confirmation
* Confirmation dialogs explain impact in plain English
* Hard reset shows 2-step confirmation with warning message
* Reflog viewer displays entries in chronological order
* File restore works for deleted files
* Operations default to safest option (soft reset over hard)

---

### 7. Reflog Recovery

 **User Story** : As a developer, I want to recover commits that seem "lost" so I can undo mistakes or find work I thought was gone.

 **Requirements** :

* Display reflog entries in chronological order (most recent first)
* Show operation type (commit, reset, checkout, merge, etc.)
* Include commit message and timestamp for each entry
* Preview commit diff before recovery
* Create branch from reflog entry
* Reset to reflog entry with safety confirmations
* Filter reflog by operation type or date range

 **Acceptance Criteria** :

* Reflog loads entries for past 90 days (Git default)
* Each entry clearly shows what operation created it
* Recovery operations follow same safety patterns as timeline actions
* User can preview changes before committing to recovery

---

### 8. GitHub Integration (TUI)

 **User Story** : As a developer, I want to create repositories and manage collaborators from the terminal so I don't have to leave my workflow.

 **Requirements** :

* Authenticate with GitHub via personal access token (PAT)
* Create new GitHub repository (public or private)
* Set repository description and default branch name
* Add repository as remote to local Git repo
* Push local repo to newly created GitHub repo
* List current collaborators for connected repo
* Add collaborators by GitHub username
* Remove collaborators with confirmation
* View collaborator permissions (read, write, admin)

 **Acceptance Criteria** :

* Authentication persists across sessions (secure token storage)
* Repository creation completes in <5s
* Push operation shows progress indicator
* Collaborator operations reflect in GitHub within 10s
* API errors are translated to user-friendly messages
* Rate limiting is handled gracefully with retry logic

---

### 9. AI Mentor Layer

 **User Story** : As a developer, I want an AI assistant that understands my repository context and explains Git operations in plain English so I can learn while I work.

 **Requirements** :

#### Context Analysis

* Analyze current repo state (branch, uncommitted changes, conflicts)
* Understand selected commits or ranges
* Parse and explain Git error messages

#### Capabilities

* **Explain Mode** : Describe what's currently happening in the repo
* "You're on branch `feature-login` with 3 uncommitted files"
* "This merge commit brought changes from `main` into your branch"
* **Recommend Mode** : Suggest safest next actions
* "To save your work before switching branches, you should commit or stash"
* "Use a new branch instead of reset to preserve this commit"
* **Warn Mode** : Alert about risky operations
* "Hard reset will permanently delete uncommitted work—are you sure?"
* "This rebase will rewrite public history; consider merge instead"
* **Error Explanation** : Translate Git errors to plain English
* Git error: `fatal: refusing to merge unrelated histories`
* AI: "These branches don't share a common ancestor. Use `--allow-unrelated-histories` if you're combining separate projects, or check if you're in the right repository."

#### Technical Implementation

* Lightweight backend on AWS Lambda
* LLM access via Amazon Bedrock (Claude or GPT-4)
* Context sent: repo state, command intent, relevant diffs, error messages
* Responses cached for 5 minutes to reduce API calls
* Fallback to basic help text if AI unavailable
* Privacy: only repo metadata sent, never full file contents unless user explicitly approves

 **Acceptance Criteria** :

* AI responses appear in <3 seconds (90th percentile)
* Explanations are accurate for 95% of common Git scenarios
* Recommendations clearly indicate safety level (safe/caution/destructive)
* Error explanations include both "what happened" and "how to fix"
* User can disable AI layer via config if desired
* Backend handles rate limiting and retries gracefully

---

## Technical Architecture

### Technology Stack

 **Frontend (TUI Client)** :

* **Language** : Rust (safety, performance, zero-cost abstractions)
* **CLI Framework** : `clap` (argument parsing, subcommands, help generation)
* **TUI Framework** : `ratatui` (rendering) + `crossterm` (terminal manipulation)
* **Git Integration** : Shell-based execution of Git commands (source of truth)
* **HTTP Client** : `reqwest` (GitHub API communication)

 **Backend (AI Mentor)** :

* **Platform** : AWS Lambda (serverless, pay-per-use)
* **Runtime** : Python 3.11 or Node.js 20
* **LLM Provider** : Amazon Bedrock (Claude Sonnet 4.5 or equivalent)
* **API Gateway** : AWS API Gateway (RESTful endpoint)
* **Authentication** : API key per user (generated on first use)

### Data Flow

```
User Input → zit CLI → ratatui TUI → Git Commands (shell) → Parse Output → Update UI
                                   ↓
                          GitHub REST API (if applicable)
                                   ↓
                        AI Mentor API (if invoked)
                                   ↓
                     AWS Lambda → Bedrock LLM → Response → UI Display
```

### Key Design Decisions

1. **Shell-Based Git Execution** : Use actual Git binary, not reimplementation

* **Rationale** : Git is the source of truth; ensures compatibility and correctness
* **Trade-off** : Slightly slower than library calls, but negligible in practice

1. **TUI Over GUI** : Terminal-native interface

* **Rationale** : Developers already work in terminals; no context switching
* **Trade-off** : Limited visual richness vs. GUI, but faster workflow

1. **Opinionated Defaults** : Non-destructive operations by default

* **Rationale** : Safety first, especially for beginners
* **Trade-off** : Advanced users may find extra confirmations annoying (mitigated via config)

1. **AI as Optional Layer** : Works without AI, enhanced with it

* **Rationale** : Not everyone wants/needs AI; tool should be useful standalone
* **Trade-off** : Two codepaths to maintain

---

## User Workflows

### Workflow 1: First-Time Commit

 **Actor** : Junior developer making their first commit

1. Open zit in repository: `zit`
2. Dashboard shows "3 modified files, 0 staged"
3. Navigate to staging view (keyboard shortcut: `s`)
4. Select files to stage using arrow keys + Space
5. Press `c` to open commit interface
6. Type commit message; AI suggests: "Add user authentication endpoints"
7. Confirm commit; dashboard updates to show clean state
8. AI explains: "You've created a snapshot of your changes. Your work is now saved in Git history."

 **Success Metrics** :

* Time to first commit: <2 minutes
* User completes workflow without consulting external documentation
* User understands what they did (measured via optional feedback prompt)

---

### Workflow 2: Recovering from a Mistake

 **Actor** : Developer who accidentally hard reset

1. User realizes they lost a commit after hard reset
2. Opens zit: `zit`
3. Dashboard shows AI warning: "Recent reset detected. Your commit may be in reflog."
4. Press `r` to open reflog viewer
5. Sees deleted commit in list with message and timestamp
6. Selects commit, presses `b` to create recovery branch
7. AI confirms: "Branch `recovery-feb10` created from lost commit. Your work is safe."

 **Success Metrics** :

* Time to recover: <1 minute
* Recovery success rate: 100% if commit is in reflog
* User confidence increase (measured via survey)

---

### Workflow 3: Creating a GitHub Repository

 **Actor** : Developer starting a new project

1. Initialize local repo: `git init` (or `zit init` if we build that)
2. Make initial commits using zit
3. Press `g` to open GitHub integration
4. Select "Create Repository"
5. Enter name: `my-awesome-project`, description, visibility (private)
6. zit creates repo, adds remote, asks to push
7. Confirm push; AI explains: "Your local commits are now on GitHub at github.com/username/my-awesome-project"

 **Success Metrics** :

* Time to GitHub: <3 minutes from first commit
* Zero failed pushes due to misconfiguration
* User successfully shares repo link with collaborator

---

## Non-Functional Requirements

### Performance

* TUI must respond to user input within 100ms (perceived instant)
* Commit timeline renders 100+ commits in <1 second
* AI responses delivered in <3 seconds (p90)
* Dashboard refresh: <500ms

### Reliability

* Git operations succeed 99.9% of the time (failures only due to Git errors)
* AI backend uptime: 99.5%
* Graceful degradation: tool works without AI layer
* Data loss: zero (all operations use Git's ACID guarantees)

### Security

* GitHub PAT stored in system keychain (macOS Keychain, Windows Credential Manager, Linux Secret Service)
* No repo content sent to AI unless explicitly user-initiated
* AI backend API keys rotated every 90 days
* HTTPS for all network communication

### Usability

* New user can make first commit without external help: 80% success rate
* Keyboard-driven navigation: all features accessible without mouse
* Color-blind accessible color scheme (tested with simulators)
* Help text available via `?` key in any view

### Compatibility

* Supported Git versions: 2.30+ (past 3 years)
* Supported terminals: all xterm-compatible terminals
* Supported platforms: macOS, Linux, Windows (via WSL or native)
* Minimum terminal size: 80x24 characters

---

## Success Metrics (OKRs)

### Objective 1: Make Git approachable for beginners

 **Key Results** :

* 70% of new users complete their first commit workflow without external help
* 50% of users report feeling "more confident with Git" after 1 week (survey)
* Average time to first commit reduced from 10 minutes (git CLI) to <3 minutes

### Objective 2: Reduce Git-related mistakes

 **Key Results** :

* 80% reduction in "destructive operation regret" incidents (measured via reflog recovery usage)
* Zero data loss incidents due to zit bugs (Git errors excluded)
* 60% of users report "fewer Git mistakes" after 1 month (survey)

### Objective 3: Drive adoption and engagement

 **Key Results** :

* 10,000 total installs within 6 months of launch
* 30% DAU/MAU ratio (daily active users / monthly active users)
* 500 GitHub stars within 3 months
* 20% of users enable AI mentor within first session

### Objective 4: Validate AI mentor value

 **Key Results** :

* 70% of AI interactions rated "helpful" or "very helpful"
* 40% of users who try AI mentor keep it enabled long-term
* AI explains Git errors correctly in 95% of cases (manual review)
* Average AI response latency <3 seconds (p90)

---

## Risks & Mitigations

### Risk 1: Git Version Fragmentation

 **Probability** : Medium

 **Impact** : High

 **Description** : Different Git versions may have incompatible output formats, breaking parsing.

 **Mitigation** :

* Require Git 2.30+ (widely available, stable output)
* Use `--porcelain` flags where available for stable output
* Extensive testing across Git versions 2.30, 2.35, 2.40, latest
* Graceful error handling with version detection and user notification

---

### Risk 2: AI Hallucination or Bad Advice

 **Probability** : Medium

 **Impact** : High

 **Description** : LLM gives incorrect Git advice, leading to user mistakes.

 **Mitigation** :

* Context prompts explicitly instruct AI to prioritize safety and admit uncertainty
* AI recommendations labeled with confidence levels (high/medium/low)
* Destructive operations always require manual confirmation regardless of AI advice
* Feedback mechanism to report bad AI advice
* Regular review of AI responses for quality assurance
* Fallback to curated help text for critical operations

---

### Risk 3: Performance on Large Repos

 **Probability** : Low

 **Impact** : Medium

 **Description** : TUI becomes sluggish on repos with 10,000+ commits or 1,000+ files.

 **Mitigation** :

* Pagination for commit timeline (load 100 commits at a time)
* Virtual scrolling for file lists (render only visible items)
* Git operations run asynchronously with loading indicators
* Performance benchmarking on Linux kernel repo (1M+ commits)
* Configurable limits for history depth and file counts

---

### Risk 4: Cross-Platform Terminal Incompatibility

 **Probability** : Medium

 **Impact** : Medium

 **Description** : TUI rendering breaks on certain terminal emulators.

 **Mitigation** :

* Test on top 10 terminal emulators (Terminal.app, iTerm2, Alacritty, Windows Terminal, GNOME Terminal, etc.)
* Use crossterm's compatibility layer
* Fallback to simple line-based UI if TUI initialization fails
* User-selectable color schemes (including monochrome)

---

### Risk 5: GitHub API Rate Limiting

 **Probability** : High

 **Impact** : Low

 **Description** : Heavy users hit GitHub API rate limits (5,000 requests/hour for authenticated users).

 **Mitigation** :

* Cache repository metadata locally (refresh on demand)
* Batch API requests where possible
* Display remaining rate limit in GitHub integration view
* Graceful error handling with retry after rate limit resets
* Educate users about rate limits via AI mentor

---

## Future Enhancements (Post-MVP)

### Phase 2: Advanced Git Operations

* Interactive rebase with visual conflict resolution
* Cherry-pick commits across branches
* Bisect workflow for bug hunting
* Submodule management
* Worktree support

### Phase 3: Collaboration Features

* Pull request creation and management
* Code review workflows
* Issue tracking integration
* Team activity feed
* Merge conflict resolution with AI suggestions

### Phase 4: Learning & Onboarding

* Interactive Git tutorial mode (in-TUI lessons)
* Achievement system for learning milestones
* Playground mode (safe sandbox for experimenting)
* Export history as learning portfolio

### Phase 5: Extensibility

* Plugin system for custom commands
* Theme customization
* Custom AI prompts
* Integration with other tools (Jira, Linear, Slack)

---

## Open Questions

1. **Licensing** : Should zit be open-source (MIT/Apache) or proprietary?

* **Recommendation** : Open-source (MIT) to drive adoption and community contributions

1. **Monetization** : How to sustain AI backend costs?

* **Options** : Freemium (limited AI calls), sponsorware, enterprise licenses
* **Recommendation** : Freemium with 100 AI calls/month free, then $5/month for unlimited

1. **Installation** : Package via Homebrew, Cargo, or standalone binaries?

* **Recommendation** : All three (Homebrew for macOS, Cargo for Rust users, binaries for everyone else)

1. **Telemetry** : Should we collect anonymous usage data to improve the product?

* **Recommendation** : Opt-in telemetry with transparent disclosure and easy opt-out

1. **Windows Support** : Native Windows build or WSL-only?

* **Recommendation** : WSL-only for MVP (simpler), native in Phase 2 if demand exists

---

## Appendix

### Glossary

* **TUI** : Text User Interface (terminal-based graphical interface)
* **Reflog** : Git's reference log (history of HEAD movements)
* **Detached HEAD** : Git state where HEAD points to a commit, not a branch
* **SHA/Hash** : Commit identifier (e.g., `a3f7d8e`)
* **Porcelain** : Git commands designed for humans (vs. "plumbing" for scripts)
* **Hunk** : A contiguous section of changed lines in a diff

### References

* Git Documentation: https://git-scm.com/doc
* GitHub REST API: https://docs.github.com/en/rest
* Ratatui Docs: https://ratatui.rs
* Amazon Bedrock: https://aws.amazon.com/bedrock/

### Competitive Analysis

| Tool                     | Type          | Strengths                                    | Weaknesses                             |
| ------------------------ | ------------- | -------------------------------------------- | -------------------------------------- |
| **GitKraken**      | GUI           | Visual, beginner-friendly                    | Requires leaving terminal, proprietary |
| **lazygit**        | TUI           | Fast, keyboard-driven                        | Lacks AI, steep learning curve         |
| **GitHub Desktop** | GUI           | Polished, GitHub integration                 | Limited Git features, GUI-only         |
| **Magit (Emacs)**  | Editor Plugin | Powerful, text-based                         | Requires Emacs, not standalone         |
| **zit**            | AI TUI        | Learning-focused, AI mentor, terminal-native | New (no ecosystem yet)                 |

### User Personas

**Persona 1: Alex the Bootcamp Grad**

* **Age** : 24
* **Experience** : 3 months coding
* **Goal** : Land first dev job
* **Pain Point** : Scared of breaking Git history during interviews
* **zit Value** : Confidence through guardrails and AI explanations

**Persona 2: Jordan the Self-Taught Developer**

* **Age** : 29
* **Experience** : 1 year coding
* **Goal** : Contribute to open-source
* **Pain Point** : Doesn't understand advanced Git (rebase, cherry-pick)
* **zit Value** : Learn-by-doing with real-time mentorship

**Persona 3: Sam the Senior Engineer**

* **Age** : 35
* **Experience** : 10 years coding
* **Goal** : Increase productivity, reduce context switching
* **Pain Point** : Tired of memorizing Git commands, wants speed
* **zit Value** : Fast TUI workflows with AI for edge cases

---

## Approval & Sign-Off

 **Product Manager** : [Name] - [Date]

 **Engineering Lead** : [Name] - [Date]

 **Design Lead** : [Name] - [Date]

 **Stakeholder** : [Name] - [Date]

---

 **Document History** :

* v1.0 (Feb 10, 2026): Initial MVP specification
