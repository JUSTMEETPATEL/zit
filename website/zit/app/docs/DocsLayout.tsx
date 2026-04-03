"use client";

import { useState, useEffect, useRef, useCallback } from "react";
import { motion, AnimatePresence } from "framer-motion";
import Link from "next/link";
import {
  Terminal,
  Github,
  Menu,
  X,
  ChevronRight,
  Search,
  Copy,
  Check,
  ArrowUp,
  BookOpen,
  Zap,
  Download,
  Settings,
  Keyboard,
  Bot,
  GitBranch,
  AlertTriangle,
  Code2,
  Layers,
  GitCommit,
  Undo2,
  Archive,
  GitMerge,
  LayoutDashboard,
  Cloud,
  ExternalLink,
} from "lucide-react";

/* ─── Sidebar navigation items ────────────────────────────── */
const sections = [
  {
    id: "overview",
    label: "Overview",
    icon: BookOpen,
    subsections: [],
  },
  {
    id: "installation",
    label: "Installation",
    icon: Download,
    subsections: [
      { id: "install-macos", label: "macOS (Homebrew)" },
      { id: "install-source", label: "From Source" },
      { id: "install-prereqs", label: "Prerequisites" },
    ],
  },
  {
    id: "quick-start",
    label: "Quick Start",
    icon: Zap,
    subsections: [
      { id: "qs-launch", label: "Launch zit" },
      { id: "qs-cli-flags", label: "CLI Flags" },
    ],
  },
  {
    id: "features",
    label: "Features",
    icon: LayoutDashboard,
    subsections: [
      { id: "feat-dashboard", label: "Repository Dashboard" },
      { id: "feat-staging", label: "Smart Staging" },
      { id: "feat-commits", label: "Guided Commits" },
      { id: "feat-branching", label: "Visual Branching" },
      { id: "feat-timeline", label: "Commit Timeline" },
      { id: "feat-timetravel", label: "Time Travel" },
      { id: "feat-reflog", label: "Reflog Recovery" },
      { id: "feat-stash", label: "Stash Manager" },
      { id: "feat-merge", label: "Merge Resolve" },
      { id: "feat-bisect", label: "Git Bisect" },
      { id: "feat-cherry", label: "Cherry Pick" },
      { id: "feat-workflow", label: "Workflow Builder" },
      { id: "feat-github", label: "GitHub Integration" },
    ],
  },
  {
    id: "keybindings",
    label: "Keybindings",
    icon: Keyboard,
    subsections: [],
  },
  {
    id: "ai-mentor",
    label: "AI Mentor",
    icon: Bot,
    subsections: [
      { id: "ai-capabilities", label: "Capabilities" },
      { id: "ai-setup", label: "Setup" },
      { id: "ai-config", label: "Config File" },
      { id: "ai-env", label: "Environment Vars" },
    ],
  },
  {
    id: "configuration",
    label: "Configuration",
    icon: Settings,
    subsections: [
      { id: "config-file", label: "Config File" },
      { id: "config-security", label: "Security" },
    ],
  },
  {
    id: "architecture",
    label: "Architecture",
    icon: Layers,
    subsections: [
      { id: "arch-stack", label: "Tech Stack" },
      { id: "arch-decisions", label: "Design Decisions" },
    ],
  },
  {
    id: "development",
    label: "Development",
    icon: Code2,
    subsections: [
      { id: "dev-building", label: "Building" },
      { id: "dev-testing", label: "Testing" },
      { id: "dev-structure", label: "Project Structure" },
    ],
  },
  {
    id: "troubleshooting",
    label: "Troubleshooting",
    icon: AlertTriangle,
    subsections: [
      { id: "ts-windows", label: "Windows Linker" },
      { id: "ts-ai", label: "AI Not Working" },
    ],
  },
];

/* ─── CodeBlock ────────────────────────────────────────────── */
const langDot: Record<string, string> = {
  bash: "bg-green-400",
  toml: "bg-amber-400",
  text: "bg-sky-400",
  ts: "bg-blue-400",
  tsx: "bg-cyan-400",
};

function CodeBlock({
  code,
  language = "bash",
  filename,
}: {
  code: string;
  language?: string;
  filename?: string;
}) {
  const [copied, setCopied] = useState(false);

  const copy = () => {
    navigator.clipboard.writeText(code);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  const dot = langDot[language] ?? "bg-white/20";

  return (
    <div className="group relative rounded-xl border border-white/8 bg-[#0b0b0b] overflow-hidden my-5 shadow-[0_4px_32px_rgba(0,0,0,0.4)]">
      {/* header bar */}
      <div className="flex items-center justify-between px-4 py-2.5 border-b border-white/6 bg-white/[0.015]">
        <div className="flex items-center gap-2">
          <span className={`w-2.5 h-2.5 rounded-full ${dot} opacity-70`} />
          <span className="text-xs text-white/30 font-mono">
            {filename ?? language}
          </span>
        </div>
        <button
          onClick={copy}
          className="flex items-center gap-1.5 text-xs text-white/25 hover:text-white/70 transition-colors px-2 py-1 rounded-md hover:bg-white/5"
        >
          {copied ? (
            <><Check size={12} className="text-emerald-400" /><span className="text-emerald-400">Copied!</span></>
          ) : (
            <><Copy size={12} /><span>Copy</span></>
          )}
        </button>
      </div>
      <div className="relative">
        <pre className="overflow-x-auto px-5 py-4 text-sm font-mono text-white/70 leading-[1.75]">
          <code>{code}</code>
        </pre>
      </div>
    </div>
  );
}

/* ─── Badge ────────────────────────────────────────────────── */
function Badge({
  children,
  color = "orange",
}: {
  children: React.ReactNode;
  color?: "orange" | "violet" | "emerald" | "blue" | "red" | "cyan";
}) {
  const colors = {
    orange: "bg-orange-500/10 text-orange-400 border-orange-500/20",
    violet: "bg-violet-500/10 text-violet-400 border-violet-500/20",
    emerald: "bg-emerald-500/10 text-emerald-400 border-emerald-500/20",
    blue: "bg-blue-500/10 text-blue-400 border-blue-500/20",
    red: "bg-red-500/10 text-red-400 border-red-500/20",
    cyan: "bg-cyan-500/10 text-cyan-400 border-cyan-500/20",
  };
  return (
    <span
      className={`inline-flex items-center px-2 py-0.5 rounded-md text-xs font-mono border ${colors[color]}`}
    >
      {children}
    </span>
  );
}

/* ─── Kbd ───────────────────────────────────────────────────── */
function Kbd({ children }: { children: React.ReactNode }) {
  return (
    <kbd className="inline-flex items-center px-2 py-0.5 rounded-md text-xs font-mono bg-white/5 border border-white/10 text-white/70">
      {children}
    </kbd>
  );
}

/* ─── SectionAnchor ────────────────────────────────────────── */
function SectionAnchor({ id }: { id: string }) {
  return <span id={id} className="-mt-24 pt-24 block absolute" />;
}

/* ─── Callout ───────────────────────────────────────────────── */
function Callout({
  type = "info",
  children,
}: {
  type?: "info" | "warning" | "tip";
  children: React.ReactNode;
}) {
  const styles = {
    info: "border-blue-500/25 bg-gradient-to-r from-blue-500/8 to-transparent text-blue-200",
    warning: "border-amber-500/25 bg-gradient-to-r from-amber-500/8 to-transparent text-amber-200",
    tip: "border-emerald-500/25 bg-gradient-to-r from-emerald-500/8 to-transparent text-emerald-200",
  };
  const icons = { info: "ℹ️", warning: "⚠️", tip: "💡" };
  const labels = { info: "Note", warning: "Warning", tip: "Tip" };
  const labelColors = {
    info: "text-blue-400",
    warning: "text-amber-400",
    tip: "text-emerald-400",
  };
  return (
    <div className={`flex gap-3 p-4 rounded-xl border my-5 text-sm leading-relaxed ${styles[type]}`}>
      <span className="text-base shrink-0 mt-0.5">{icons[type]}</span>
      <div>
        <span className={`font-bold text-xs uppercase tracking-wider mr-2 ${labelColors[type]}`}>
          {labels[type]}:
        </span>
        {children}
      </div>
    </div>
  );
}

/* ─── Feature Row ───────────────────────────────────────────── */
function FeatureSection({
  id,
  icon: Icon,
  color,
  glowColor,
  title,
  keybind,
  description,
  details,
}: {
  id: string;
  icon: React.ElementType;
  color: string;
  glowColor?: string;
  title: string;
  keybind: string;
  description: string;
  details: string[];
}) {
  return (
    <motion.div
      className="relative mb-6"
      initial={{ opacity: 0, y: 16 }}
      whileInView={{ opacity: 1, y: 0 }}
      viewport={{ once: true, margin: "-60px" }}
      transition={{ duration: 0.4 }}
    >
      <SectionAnchor id={id} />
      <div
        className={`group glass rounded-2xl border border-white/6 p-6 hover:border-white/12 transition-all duration-300 ${
          glowColor ? `hover:shadow-[0_0_40px_rgba(0,0,0,0.3)] hover:${glowColor}` : ""
        }`}
      >
        <div className="flex items-start gap-4 mb-4">
          <div className={`p-3 rounded-xl bg-white/[0.04] border border-white/8 ${color} group-hover:scale-105 transition-transform duration-300`}>
            <Icon size={20} />
          </div>
          <div className="flex-1">
            <div className="flex items-center gap-3 mb-1.5">
              <h3 className="text-white font-bold text-base tracking-tight">{title}</h3>
              <Kbd>{keybind}</Kbd>
            </div>
            <p className="text-white/45 text-sm leading-relaxed">{description}</p>
          </div>
        </div>
        <ul className="space-y-2 border-t border-white/5 pt-4">
          {details.map((d, i) => (
            <li key={i} className="flex items-start gap-2.5 text-sm text-white/35">
              <span className={`mt-1.5 w-1 h-1 rounded-full ${color} opacity-60 shrink-0`} />
              <span>{d}</span>
            </li>
          ))}
        </ul>
      </div>
    </motion.div>
  );
}

/* ─── Main DocsLayout ───────────────────────────────────────── */
export default function DocsLayout() {
  const [sidebarOpen, setSidebarOpen] = useState(false);
  const [activeSection, setActiveSection] = useState("overview");
  const [searchQuery, setSearchQuery] = useState("");
  const [showScrollTop, setShowScrollTop] = useState(false);
  const [scrollProgress, setScrollProgress] = useState(0);
  const contentRef = useRef<HTMLDivElement>(null);

  const updateProgress = useCallback(() => {
    const el = document.documentElement;
    const scrolled = el.scrollTop;
    const total = el.scrollHeight - el.clientHeight;
    setScrollProgress(total > 0 ? Math.min((scrolled / total) * 100, 100) : 0);
  }, []);

  // Scroll spy
  useEffect(() => {
    const allIds = sections.flatMap((s) => [
      s.id,
      ...s.subsections.map((sub) => sub.id),
    ]);

    const observer = new IntersectionObserver(
      (entries) => {
        for (const entry of entries) {
          if (entry.isIntersecting) {
            setActiveSection(entry.target.id);
          }
        }
      },
      { rootMargin: "-20% 0px -70% 0px" }
    );

    allIds.forEach((id) => {
      const el = document.getElementById(id);
      if (el) observer.observe(el);
    });

    return () => observer.disconnect();
  }, []);

  useEffect(() => {
    const fn = () => {
      setShowScrollTop(window.scrollY > 400);
      updateProgress();
    };
    window.addEventListener("scroll", fn, { passive: true });
    return () => window.removeEventListener("scroll", fn);
  }, [updateProgress]);

  const filteredSections = sections.filter(
    (s) =>
      !searchQuery ||
      s.label.toLowerCase().includes(searchQuery.toLowerCase()) ||
      s.subsections.some((sub) =>
        sub.label.toLowerCase().includes(searchQuery.toLowerCase())
      )
  );

  return (
    <div className="min-h-screen bg-[var(--background)] text-[var(--foreground)]">
      {/* Scroll progress bar */}
      <div
        className="fixed top-0 left-0 z-[60] h-[2px] bg-gradient-to-r from-orange-500 via-amber-400 to-orange-600 transition-all duration-75"
        style={{ width: `${scrollProgress}%` }}
      />
      {/* ── Top Nav ─────────────────────────────────────────── */}
      <nav className="fixed top-0 left-0 right-0 z-50 bg-[#080808]/90 backdrop-blur-xl border-b border-white/5 h-14 flex items-center">
        <div className="max-w-screen-2xl w-full mx-auto px-4 flex items-center justify-between gap-4">
          <div className="flex items-center gap-4">
            {/* Mobile sidebar toggle */}
            <button
              className="lg:hidden text-white/40 hover:text-white p-1 transition-colors"
              onClick={() => setSidebarOpen(!sidebarOpen)}
            >
              {sidebarOpen ? <X size={20} /> : <Menu size={20} />}
            </button>
            <Link href="/" className="flex items-center gap-2">
              <span className="bg-orange-500/10 border border-orange-500/20 text-orange-400 p-1.5 rounded-lg">
                <Terminal size={15} />
              </span>
              <span className="font-black text-white text-base tracking-tight">
                zit
              </span>
            </Link>
            <ChevronRight size={14} className="text-white/20 hidden sm:block" />
            <span className="text-white/30 text-sm hidden sm:block">Docs</span>
          </div>

          <div className="flex items-center gap-3">
            <Link
              href="/"
              className="text-white/30 hover:text-white text-sm transition-colors hidden sm:block"
            >
              Home
            </Link>
            <Link
              href="https://github.com/JUSTMEETPATEL/zit"
              target="_blank"
              className="flex items-center gap-1.5 text-white/30 hover:text-white transition-colors text-sm"
            >
              <Github size={16} />
              <span className="hidden sm:block">GitHub</span>
            </Link>
          </div>
        </div>
      </nav>

      <div className="flex pt-14 max-w-screen-2xl mx-auto">
        {/* ── Sidebar ──────────────────────────────────────── */}
        <AnimatePresence>
          {(sidebarOpen || true) && (
            <motion.aside
              initial={false}
              className={`fixed lg:sticky top-14 h-[calc(100vh-3.5rem)] w-64 shrink-0 overflow-y-auto border-r border-white/5 bg-[#080808] z-40 transition-transform duration-300
                ${sidebarOpen ? "translate-x-0" : "-translate-x-full lg:translate-x-0"}
              `}
            >
              <div className="p-4">
                {/* Sidebar header */}
                <div className="flex items-center justify-between mb-4 pb-3 border-b border-white/5">
                  <span className="text-xs font-bold text-white/25 uppercase tracking-widest">Navigation</span>
                  <span className="text-xs font-mono text-orange-400/60 bg-orange-500/8 border border-orange-500/15 px-1.5 py-0.5 rounded">v0.1.0</span>
                </div>
                {/* Search */}
                <div className="relative mb-4">
                  <Search
                    size={14}
                    className="absolute left-3 top-1/2 -translate-y-1/2 text-white/25"
                  />
                  <input
                    type="text"
                    placeholder="Search docs…"
                    value={searchQuery}
                    onChange={(e) => setSearchQuery(e.target.value)}
                    className="w-full bg-white/5 border border-white/8 rounded-lg pl-8 pr-3 py-2 text-sm text-white/70 placeholder:text-white/20 focus:outline-none focus:border-orange-500/30 transition-colors"
                  />
                </div>

                {/* Nav */}
                <nav className="space-y-0.5">
                  {filteredSections.map((section) => {
                    const Icon = section.icon;
                    const isActive =
                      activeSection === section.id ||
                      section.subsections.some(
                        (sub) => sub.id === activeSection
                      );
                    return (
                      <div key={section.id}>
                        <a
                          href={`#${section.id}`}
                          onClick={() => setSidebarOpen(false)}
                          className={`flex items-center gap-2.5 px-3 py-2 rounded-lg text-sm font-medium transition-all group ${
                            isActive
                              ? "bg-orange-500/10 text-orange-400 border border-orange-500/15"
                              : "text-white/35 hover:text-white hover:bg-white/5"
                          }`}
                        >
                          <Icon
                            size={14}
                            className={
                              isActive ? "text-orange-400" : "text-white/20"
                            }
                          />
                          {section.label}
                        </a>
                        {section.subsections.length > 0 && (
                          <div className="ml-6 mt-0.5 mb-1 space-y-0.5">
                            {section.subsections.map((sub) => (
                              <a
                                key={sub.id}
                                href={`#${sub.id}`}
                                onClick={() => setSidebarOpen(false)}
                                className={`block px-3 py-1.5 rounded-md text-xs transition-colors ${
                                  activeSection === sub.id
                                    ? "text-orange-400 bg-orange-500/8"
                                    : "text-white/25 hover:text-white/60"
                                }`}
                              >
                                {sub.label}
                              </a>
                            ))}
                          </div>
                        )}
                      </div>
                    );
                  })}
                </nav>

                {/* Sidebar footer */}
                <div className="mt-6 pt-4 border-t border-white/5">
                  <a
                    href="https://github.com/JUSTMEETPATEL/zit/blob/main/CONTRIBUTING.md"
                    target="_blank"
                    className="flex items-center gap-2 text-xs text-white/20 hover:text-white/50 transition-colors"
                  >
                    <span>Contributing Guide</span>
                    <ExternalLink size={10} />
                  </a>
                </div>
              </div>
            </motion.aside>
          )}
        </AnimatePresence>

        {/* Overlay for mobile */}
        {sidebarOpen && (
          <div
            className="fixed inset-0 bg-black/50 z-30 lg:hidden"
            onClick={() => setSidebarOpen(false)}
          />
        )}

        {/* ── Main Content ──────────────────────────────────── */}
        <main
          ref={contentRef}
          className="flex-1 min-w-0 px-6 lg:px-12 py-10 max-w-4xl"
        >
          {/* ───────────── OVERVIEW ───────────── */}
          <div className="relative mb-20">
            <SectionAnchor id="overview" />

            {/* Hero */}
            <motion.div
              initial={{ opacity: 0, y: 20 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ duration: 0.5 }}
              className="mb-8"
            >
              <div className="flex items-center gap-2.5 mb-5">
                <Badge color="orange">v0.1.0</Badge>
                <Badge color="violet">Rust</Badge>
                <Badge color="blue">MIT License</Badge>
              </div>
              <h1 className="text-5xl lg:text-6xl font-black tracking-tighter mb-5 leading-[1.05]">
                <span className="text-white">zit</span>{" "}
                <span
                  style={{
                    background: "linear-gradient(135deg, #f97316 0%, #fb923c 50%, #fbbf24 100%)",
                    WebkitBackgroundClip: "text",
                    WebkitTextFillColor: "transparent",
                    backgroundClip: "text",
                  }}
                >
                  docs
                </span>
              </h1>
              <p className="text-lg text-white/50 leading-relaxed max-w-2xl mb-6">
                <strong className="text-white/80">zit</strong> is an AI-powered,
                terminal-based Git and GitHub assistant built in Rust. It combines
                a rich TUI with intelligent AI mentorship to make Git accessible,
                safe, and educational.
              </p>

              {/* Terminal preview block */}
              <div className="rounded-2xl border border-white/8 bg-[#0a0a0a] overflow-hidden shadow-[0_8px_48px_rgba(249,115,22,0.08)]">
                <div className="flex items-center gap-2 px-4 py-3 border-b border-white/6 bg-white/[0.015]">
                  <span className="w-3 h-3 rounded-full bg-red-500/60" />
                  <span className="w-3 h-3 rounded-full bg-amber-500/60" />
                  <span className="w-3 h-3 rounded-full bg-emerald-500/60" />
                  <span className="ml-3 text-xs text-white/20 font-mono">terminal</span>
                </div>
                <div className="p-5 font-mono text-sm space-y-1">
                  <div>
                    <span className="text-emerald-400">$</span>
                    <span className="text-white/60"> cd my-project</span>
                  </div>
                  <div>
                    <span className="text-emerald-400">$</span>
                    <span className="text-orange-400 font-bold"> zit</span>
                  </div>
                  <div className="mt-2 text-white/25 text-xs">Launching zit v0.1.0 — AI-Powered Git Assistant</div>
                  <div className="text-white/25 text-xs">Repository: my-project · Branch: main · 3 uncommitted changes</div>
                </div>
              </div>
            </motion.div>

            {/* Stats row */}
            <div className="grid grid-cols-2 sm:grid-cols-4 gap-3 mb-6">
              {[
                { label: "Git Features", value: "14+", color: "text-orange-400", bg: "from-orange-500/10" },
                { label: "Tests", value: "178", color: "text-emerald-400", bg: "from-emerald-500/10" },
                { label: "Language", value: "Rust", color: "text-violet-400", bg: "from-violet-500/10" },
                { label: "License", value: "MIT", color: "text-blue-400", bg: "from-blue-500/10" },
              ].map((stat, i) => (
                <motion.div
                  key={stat.label}
                  initial={{ opacity: 0, y: 12 }}
                  animate={{ opacity: 1, y: 0 }}
                  transition={{ delay: 0.1 + i * 0.08 }}
                  className={`rounded-xl p-4 border border-white/6 bg-gradient-to-br ${stat.bg} to-transparent`}
                >
                  <div className={`text-2xl font-black ${stat.color}`}>{stat.value}</div>
                  <div className="text-xs text-white/30 mt-0.5">{stat.label}</div>
                </motion.div>
              ))}
            </div>

            <Callout type="tip">
              AI is <strong>100% optional</strong> — all core Git features work
              without any AI configuration. The AI Mentor requires an AWS Lambda
              backend (see <a href="#ai-setup" className="underline underline-offset-2">AI Setup</a>).
            </Callout>
          </div>

          {/* ───────────── INSTALLATION ───────────── */}
          <section className="relative mb-16">
            <SectionAnchor id="installation" />
            <div className="flex items-center gap-3 mb-6">
              <div className="p-1.5 rounded-lg bg-orange-500/10 border border-orange-500/20">
                <Download size={16} className="text-orange-400" />
              </div>
              <h2 className="text-2xl font-black text-white">Installation</h2>
              <div className="flex-1 h-px bg-gradient-to-r from-white/8 to-transparent" />
            </div>

            <SectionAnchor id="install-macos" />
            <h3 className="text-base font-bold text-white/80 mb-2">
              macOS — Homebrew
            </h3>
            <p className="text-white/40 text-sm mb-2">
              The recommended way to install zit on macOS:
            </p>
            <CodeBlock
              code={`brew tap JUSTMEETPATEL/zit\nbrew install zit`}
              language="bash"
            />

            <SectionAnchor id="install-source" />
            <h3 className="text-base font-bold text-white/80 mt-8 mb-2">
              From Source (Linux / macOS / Windows)
            </h3>
            <CodeBlock
              code={`cargo install --git https://github.com/JUSTMEETPATEL/zit`}
              language="bash"
            />

            <SectionAnchor id="install-prereqs" />
            <h3 className="text-base font-bold text-white/80 mt-8 mb-2">
              Prerequisites
            </h3>
            <ul className="space-y-2 text-sm text-white/50">
              <li className="flex items-center gap-2">
                <ChevronRight size={14} className="text-white/20" />
                <a
                  href="https://rustup.rs"
                  target="_blank"
                  className="text-orange-400 hover:underline flex items-center gap-1"
                >
                  Rust (via rustup) <ExternalLink size={11} />
                </a>
              </li>
              <li className="flex items-center gap-2">
                <ChevronRight size={14} className="text-white/20" />
                <span>
                  <code className="text-white/70 font-mono">git</code> installed and on your PATH
                </span>
              </li>
              <li className="flex items-center gap-2">
                <ChevronRight size={14} className="text-white/20" />
                <span>A modern terminal with TrueColor support</span>
              </li>
              <li className="flex items-center gap-2">
                <ChevronRight size={14} className="text-white/20" />
                <span>
                  <strong className="text-white/60">Windows only:</strong>{" "}
                  <a
                    href="https://visualstudio.microsoft.com/visual-cpp-build-tools/"
                    target="_blank"
                    className="text-orange-400 hover:underline"
                  >
                    C++ Build Tools
                  </a>{" "}
                  with &ldquo;Desktop development with C++&rdquo; workload
                </span>
              </li>
            </ul>
          </section>

          {/* ───────────── QUICK START ───────────── */}
          <section className="relative mb-16">
            <SectionAnchor id="quick-start" />
            <div className="flex items-center gap-3 mb-6">
              <div className="p-1.5 rounded-lg bg-orange-500/10 border border-orange-500/20">
                <Zap size={16} className="text-orange-400" />
              </div>
              <h2 className="text-2xl font-black text-white">Quick Start</h2>
              <div className="flex-1 h-px bg-gradient-to-r from-white/8 to-transparent" />
            </div>

            <SectionAnchor id="qs-launch" />
            <p className="text-white/50 text-sm mb-2">
              Navigate to any Git repository and run:
            </p>
            <CodeBlock code={`cd my-repo\nzit`} language="bash" />

            <SectionAnchor id="qs-cli-flags" />
            <h3 className="text-base font-bold text-white/80 mt-8 mb-3">
              CLI Flags
            </h3>
            <div className="rounded-xl border border-white/6 overflow-hidden">
              <table className="w-full text-sm">
                <thead>
                  <tr className="border-b border-white/8 bg-gradient-to-r from-white/[0.03] to-transparent">
                    <th className="text-left px-4 py-3 text-white/50 font-semibold text-xs uppercase tracking-wider">Flag</th>
                    <th className="text-left px-4 py-3 text-white/50 font-semibold text-xs uppercase tracking-wider">Description</th>
                  </tr>
                </thead>
                <tbody>
                  {[
                    { flag: "--help, -h", desc: "Print help and available views" },
                    { flag: "--version, -v", desc: "Print version" },
                    { flag: "--verbose", desc: "Enable debug logging (ZIT_LOG=debug)" },
                    { flag: "--no-ai", desc: "Disable AI features for this session" },
                  ].map((row, i) => (
                    <tr key={row.flag} className={`border-b border-white/4 transition-colors hover:bg-white/[0.025] ${i % 2 !== 0 ? "bg-white/[0.01]" : ""}`}>
                      <td className="px-4 py-3">
                        <code className="text-orange-400 font-mono text-xs bg-orange-500/8 border border-orange-500/15 px-2 py-0.5 rounded">
                          {row.flag}
                        </code>
                      </td>
                      <td className="px-4 py-3 text-white/45">{row.desc}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          </section>

          {/* ───────────── FEATURES ───────────── */}
          <section className="relative mb-16">
            <SectionAnchor id="features" />
            <div className="flex items-center gap-3 mb-3">
              <div className="p-1.5 rounded-lg bg-orange-500/10 border border-orange-500/20">
                <LayoutDashboard size={16} className="text-orange-400" />
              </div>
              <h2 className="text-2xl font-black text-white">Features</h2>
              <div className="flex-1 h-px bg-gradient-to-r from-white/8 to-transparent" />
            </div>
            <p className="text-white/40 text-sm mb-8">
              zit ships with 14+ built-in Git features. Press the keybind from
              the dashboard to enter each view.
            </p>

            <FeatureSection
              id="feat-dashboard"
              icon={LayoutDashboard}
              color="text-orange-400"
              glowColor="shadow-orange-500/5"
              title="Repository Dashboard"
              keybind="(default view)"
              description="Your mission control. Shows branch name, dirty state, recent commits, and provides a launching pad for every other view."
              details={[
                "At-a-glance repo health: current branch, uncommitted changes, stash count",
                "Recent commit summary with timestamps",
                "Quick access to all features via keyboard shortcuts",
              ]}
            />

            <FeatureSection
              id="feat-staging"
              icon={GitCommit}
              color="text-emerald-400"
              glowColor="shadow-emerald-500/5"
              title="Smart Staging"
              keybind="s"
              description="Interactive file staging with full diff previews and hunk-level granularity. Never blindly use git add . again."
              details={[
                "Side-by-side diff preview for every changed file",
                "Hunk-level staging: stage only the lines you want",
                "Search through changed files with /",
                "Unstage or discard changes with confirmation",
              ]}
            />

            <FeatureSection
              id="feat-commits"
              icon={GitCommit}
              color="text-violet-400"
              glowColor="shadow-violet-500/5"
              title="Guided Commits"
              keybind="c"
              description="A structured commit editor with subject/body validation and AI-powered message generation."
              details={[
                "Subject line length enforcement (72 char limit)",
                "Separate subject / body / footer fields",
                "Press Ctrl+G to generate a commit message from your staged diff using AI",
                "Conventional commit prefix helpers",
              ]}
            />

            <FeatureSection
              id="feat-branching"
              icon={GitBranch}
              color="text-blue-400"
              glowColor="shadow-blue-500/5"
              title="Visual Branching"
              keybind="b"
              description="Create, switch, delete, and rename branches with a clean visual tree. Toggle between local and remote branches."
              details={[
                "List all local and remote branches",
                "Create a new branch and optionally check it out immediately",
                "Delete or rename branches with safety confirmations",
                "View tracking relationships between local and remote branches",
              ]}
            />

            <FeatureSection
              id="feat-timeline"
              icon={GitCommit}
              color="text-sky-400"
              glowColor="shadow-sky-500/5"
              title="Commit Timeline"
              keybind="l"
              description="Browse the full git log with a visual commit graph. Search, filter, and inspect any commit."
              details={[
                "ASCII commit graph for branch visualization",
                "Full commit metadata: author, date, hash, message",
                "Search commits by message or hash",
                "Click to inspect a commit's diff",
              ]}
            />

            <FeatureSection
              id="feat-timetravel"
              icon={Undo2}
              color="text-red-400"
              glowColor="shadow-red-500/5"
              title="Time Travel"
              keybind="t"
              description="Safe reset and restore with soft, mixed, or hard modes. Every destructive operation requires confirmation."
              details={[
                "Soft reset: moves HEAD, keeps changes staged",
                "Mixed reset: moves HEAD, unstages changes",
                "Hard reset: moves HEAD and discards all changes (with warning)",
                "Restore individual files to their state at any commit",
              ]}
            />

            <FeatureSection
              id="feat-reflog"
              icon={GitBranch}
              color="text-amber-400"
              title="Reflog Recovery"
              keybind="r"
              description='Browse the git reflog to find and recover "lost" commits after resets, merges, or rebases.'
              details={[
                "Full reflog browser with timestamps and operation labels",
                "Identify the exact SHA of any previous HEAD state",
                "One-key restore to recover lost work",
              ]}
            />

            <FeatureSection
              id="feat-stash"
              icon={Archive}
              color="text-amber-400"
              glowColor="shadow-amber-500/5"
              title="Stash Manager"
              keybind="x"
              description="Manage your stash stack visually — save, pop, apply, drop, or clear stashes without memorizing commands."
              details={[
                "List all stashes with messages and timestamps",
                "Pop (apply + drop) or apply-only a stash",
                "Drop a single stash or clear the entire stack",
                "Save the current working tree with a custom message",
              ]}
            />

            <FeatureSection
              id="feat-merge"
              icon={GitMerge}
              color="text-cyan-400"
              glowColor="shadow-cyan-500/5"
              title="Merge Resolve"
              keybind="m"
              description="Resolve merge conflicts visually. Choose 'ours', 'theirs', or let the AI suggest the correct merge."
              details={[
                "Lists all files in conflict",
                "For each conflict: show both versions side-by-side",
                "One-key accept ours / accept theirs",
                "AI-assisted merge suggestion via the Mentor panel",
              ]}
            />

            <FeatureSection
              id="feat-bisect"
              icon={GitBranch}
              color="text-rose-400"
              title="Git Bisect"
              keybind="B"
              description="Find the exact commit that introduced a bug using interactive binary search."
              details={[
                "Set a known-good and known-bad commit to start",
                "Mark the current commit as good or bad",
                "Automatically narrows to the culprit commit",
                "Abort an in-progress bisect session",
              ]}
            />

            <FeatureSection
              id="feat-cherry"
              icon={GitCommit}
              color="text-pink-400"
              title="Cherry Pick"
              keybind="p"
              description="Pick one or more commits from other branches and apply them to your current branch."
              details={[
                "Browse commits from any local branch",
                "Multi-select commits for a bulk cherry-pick",
                "Conflict detection with inline guidance",
              ]}
            />

            <FeatureSection
              id="feat-workflow"
              icon={Layers}
              color="text-indigo-400"
              title="Workflow Builder"
              keybind="w"
              description="Visually compose multi-step git workflows and run them as a sequence."
              details={[
                "Add steps from a menu of common git operations",
                "Reorder or remove steps before executing",
                "Run the workflow and inspect output per step",
              ]}
            />

            <FeatureSection
              id="feat-github"
              icon={Cloud}
              color="text-indigo-400"
              title="GitHub Integration"
              keybind="g"
              description="Full GitHub integration via OAuth device flow — push, pull, create PRs, manage collaborators, and trigger CI/CD actions."
              details={[
                "OAuth device flow — no PAT management needed",
                "Create new GitHub repositories directly from zit",
                "Push, pull, and sync remotes",
                "Create, view, and manage Pull Requests",
                "Trigger and monitor GitHub Actions",
                "Add or remove collaborators",
              ]}
            />
          </section>

          {/* ───────────── KEYBINDINGS ───────────── */}
          <section className="relative mb-16">
            <SectionAnchor id="keybindings" />
            <div className="flex items-center gap-3 mb-6">
              <div className="p-1.5 rounded-lg bg-orange-500/10 border border-orange-500/20">
                <Keyboard size={16} className="text-orange-400" />
              </div>
              <h2 className="text-2xl font-black text-white">Keybindings</h2>
              <div className="flex-1 h-px bg-gradient-to-r from-white/8 to-transparent" />
            </div>
            <div className="rounded-xl border border-white/6 overflow-hidden">
              <table className="w-full text-sm">
                <thead>
                  <tr className="border-b border-white/8 bg-gradient-to-r from-white/[0.03] to-transparent">
                    <th className="text-left px-4 py-3 text-white/50 font-semibold text-xs uppercase tracking-wider w-24">Key</th>
                    <th className="text-left px-4 py-3 text-white/50 font-semibold text-xs uppercase tracking-wider">Action</th>
                  </tr>
                </thead>
                <tbody>
                  {[
                    { key: "s", action: "Staging — interactive file staging with diffs", color: "text-emerald-400" },
                    { key: "c", action: "Commit — write and submit commits", color: "text-violet-400" },
                    { key: "b", action: "Branches — create, switch, delete, rename", color: "text-blue-400" },
                    { key: "l", action: "Log — visual commit timeline / graph", color: "text-sky-400" },
                    { key: "t", action: "Time Travel — reset / restore safely", color: "text-red-400" },
                    { key: "r", action: "Reflog — recover lost commits", color: "text-amber-400" },
                    { key: "x", action: "Stash — save, pop, apply, drop stashes", color: "text-amber-400" },
                    { key: "m", action: "Merge Resolve — resolve merge conflicts", color: "text-cyan-400" },
                    { key: "B", action: "Bisect — binary search for bad commits", color: "text-rose-400" },
                    { key: "p", action: "Cherry Pick — pick commits from other branches", color: "text-pink-400" },
                    { key: "w", action: "Workflow — build multi-step git workflows", color: "text-indigo-400" },
                    { key: "g", action: "GitHub — sync, push/pull, PRs, actions, collaborators", color: "text-indigo-400" },
                    { key: "a", action: "AI Mentor — explain repo, ask questions, get recommendations", color: "text-orange-400" },
                    { key: "?", action: "Help — context-sensitive keybinding reference", color: "text-white/50" },
                    { key: "Ctrl+G", action: "Generate AI commit message (in Commit view)", color: "text-orange-400" },
                    { key: "q", action: "Quit", color: "text-white/30" },
                  ].map((row, i) => (
                    <tr key={row.key} className={`border-b border-white/4 transition-colors hover:bg-white/[0.025] ${i % 2 === 0 ? "" : "bg-white/[0.01]"}`}>
                      <td className="px-4 py-2.5">
                        <kbd className={`inline-flex items-center px-2 py-0.5 rounded-md text-xs font-mono border border-white/10 bg-white/5 ${row.color}`}>
                          {row.key}
                        </kbd>
                      </td>
                      <td className="px-4 py-2.5 text-white/45">{row.action}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          </section>


          {/* ───────────── AI MENTOR ───────────── */}
          <section className="relative mb-16">
            <SectionAnchor id="ai-mentor" />
            <div className="flex items-center gap-3 mb-6">
              <div className="p-1.5 rounded-lg bg-orange-500/10 border border-orange-500/20">
                <Bot size={16} className="text-orange-400" />
              </div>
              <h2 className="text-2xl font-black text-white">AI Mentor</h2>
              <div className="flex-1 h-px bg-gradient-to-r from-white/8 to-transparent" />
            </div>
            <p className="text-white/45 text-sm mb-6 leading-relaxed">
              The AI Mentor panel (<Kbd>a</Kbd> from the dashboard) provides
              intelligent guidance powered by Amazon Bedrock (Claude 3 Sonnet)
              via an AWS Lambda backend.
            </p>

            <SectionAnchor id="ai-capabilities" />
            <h3 className="text-base font-bold text-white/80 mb-3">Capabilities</h3>
            <div className="grid sm:grid-cols-2 gap-3 mb-8">
              {[
                { icon: "🔍", title: "Explain Repo", desc: "AI analyzes and explains your current repository state in plain English.", color: "border-blue-500/20 bg-blue-500/5" },
                { icon: "💬", title: "Ask a Question", desc: "Ask anything about Git — conventions, commands, workflows — and get a clear answer.", color: "border-violet-500/20 bg-violet-500/5" },
                { icon: "🛡️", title: "Recommend", desc: "Get safe, context-aware recommendations for your next Git operation.", color: "border-emerald-500/20 bg-emerald-500/5" },
                { icon: "🏥", title: "Health Check", desc: "Test connectivity to the AI backend and verify your configuration.", color: "border-amber-500/20 bg-amber-500/5" },
              ].map((cap) => (
                <motion.div
                  key={cap.title}
                  initial={{ opacity: 0, scale: 0.97 }}
                  whileInView={{ opacity: 1, scale: 1 }}
                  viewport={{ once: true }}
                  className={`rounded-xl p-4 border ${cap.color} hover:brightness-110 transition-all`}
                >
                  <div className="text-xl mb-2">{cap.icon}</div>
                  <div className="font-bold text-white/80 text-sm mb-1">{cap.title}</div>
                  <div className="text-white/35 text-xs leading-relaxed">{cap.desc}</div>
                </motion.div>
              ))}
            </div>

            <div className="glass rounded-xl border border-white/6 p-4 mb-6">
              <h4 className="text-sm font-bold text-white/70 mb-2">Automatic AI Features</h4>
              <ul className="space-y-2 text-sm text-white/40">
                <li className="flex items-start gap-2">
                  <ChevronRight size={14} className="mt-0.5 text-white/20 shrink-0" />
                  <span><Kbd>Ctrl+G</Kbd> in the Commit view — generates a commit message from your staged diff</span>
                </li>
                <li className="flex items-start gap-2">
                  <ChevronRight size={14} className="mt-0.5 text-white/20 shrink-0" />
                  <span><strong className="text-white/60">Auto Error Explainer</strong> — when a git command fails, AI automatically explains the error and suggests fixes</span>
                </li>
              </ul>
            </div>

            <SectionAnchor id="ai-setup" />
            <h3 className="text-base font-bold text-white/80 mt-8 mb-3">Setup</h3>
            <p className="text-white/40 text-sm mb-3">
              AI features require an AWS Lambda backend. Deploy with one command:
            </p>
            <CodeBlock
              code={`cd aws\n./deploy.sh`}
              language="bash"
              filename="aws/deploy.sh"
            />
            <p className="text-white/40 text-sm mt-3 mb-3">
              This deploys the Lambda function and API Gateway using AWS SAM. See{" "}
              <a
                href="https://github.com/JUSTMEETPATEL/zit/blob/main/aws/README.md"
                target="_blank"
                className="text-orange-400 hover:underline"
              >
                aws/README.md
              </a>{" "}
              for full instructions.
            </p>

            <SectionAnchor id="ai-config" />
            <h3 className="text-base font-bold text-white/80 mt-8 mb-2">
              Option A — Config File
            </h3>
            <CodeBlock
              code={`[ai]\nenabled = true\nendpoint = "https://your-api.execute-api.region.amazonaws.com/dev/mentor"\napi_key = "your-api-key"\ntimeout_secs = 30`}
              language="toml"
              filename="~/.config/zit/config.toml"
            />

            <SectionAnchor id="ai-env" />
            <h3 className="text-base font-bold text-white/80 mt-6 mb-2">
              Option B — Environment Variables
            </h3>
            <CodeBlock
              code={`export ZIT_AI_ENDPOINT="https://your-api.execute-api.region.amazonaws.com/dev/mentor"\nexport ZIT_AI_API_KEY="your-api-key"`}
              language="bash"
            />

            <Callout type="info">
              AI is completely optional — all core features work without it.
              When AI is not configured, the Mentor panel shows setup
              instructions instead of erroring.
            </Callout>
          </section>

          {/* ───────────── CONFIGURATION ───────────── */}
          <section className="relative mb-16">
            <SectionAnchor id="configuration" />
            <div className="flex items-center gap-3 mb-6">
              <div className="p-1.5 rounded-lg bg-orange-500/10 border border-orange-500/20">
                <Settings size={16} className="text-orange-400" />
              </div>
              <h2 className="text-2xl font-black text-white">Configuration</h2>
              <div className="flex-1 h-px bg-gradient-to-r from-white/8 to-transparent" />
            </div>

            <SectionAnchor id="config-file" />
            <p className="text-white/45 text-sm mb-3">
              zit reads its configuration from{" "}
              <code className="text-white/70 font-mono text-xs bg-white/5 px-1.5 py-0.5 rounded">
                ~/.config/zit/config.toml
              </code>
              . All settings are optional.
            </p>
            <CodeBlock
              code={`[general]
tick_rate_ms = 2000          # UI refresh interval (ms)
confirm_destructive = true   # Require confirmation for risky operations

[ui]
color_scheme = "default"
show_help_hints = true

[github]
# pat = "ghp_..."           # Or use OAuth device flow from the GitHub view

[ai]
enabled = true
endpoint = "https://..."
api_key = "..."
timeout_secs = 30`}
              language="toml"
              filename="~/.config/zit/config.toml"
            />

            <SectionAnchor id="config-security" />
            <h3 className="text-base font-bold text-white/80 mt-8 mb-3">Security</h3>
            <Callout type="warning">
              GitHub tokens and AI API keys are automatically migrated from the
              config file to the OS keychain (macOS Keychain, Windows Credential
              Manager, Linux Secret Service) on first run. Plaintext values are
              removed from the config file after migration.
            </Callout>
          </section>

          {/* ───────────── ARCHITECTURE ───────────── */}
          <section className="relative mb-16">
            <SectionAnchor id="architecture" />
            <div className="flex items-center gap-3 mb-6">
              <div className="p-1.5 rounded-lg bg-orange-500/10 border border-orange-500/20">
                <Layers size={16} className="text-orange-400" />
              </div>
              <h2 className="text-2xl font-black text-white">Architecture</h2>
              <div className="flex-1 h-px bg-gradient-to-r from-white/8 to-transparent" />
            </div>

            <SectionAnchor id="arch-stack" />
            <h3 className="text-base font-bold text-white/80 mb-3">Tech Stack</h3>
            <CodeBlock
              code={`zit (Rust TUI)
├── ratatui + crossterm      — Terminal UI rendering
├── Git CLI (shell)          — All git operations via native git
├── reqwest (blocking)       — HTTP for GitHub API + AI backend
└── AI Client                — Background thread + mpsc channel
    └── AWS Lambda (Python 3.12)
        └── Amazon Bedrock (Claude 3 Sonnet)`}
              language="text"
            />

            <SectionAnchor id="arch-decisions" />
            <h3 className="text-base font-bold text-white/80 mt-8 mb-4">Design Decisions</h3>
            <div className="space-y-3">
              {[
                {
                  title: "Shell-based Git",
                  desc: "Runs real git commands — never reimplements git internals. This means 100% compatibility with any git version and no maintenance burden.",
                  icon: "⚙️",
                  color: "border-orange-500/20 bg-gradient-to-r from-orange-500/8 to-transparent",
                },
                {
                  title: "AI is Optional",
                  desc: "Degrades gracefully to static help when AI is unavailable. The core tool is fully functional even in air-gapped environments.",
                  icon: "🔌",
                  color: "border-violet-500/20 bg-gradient-to-r from-violet-500/8 to-transparent",
                },
                {
                  title: "Non-blocking AI",
                  desc: "All AI calls run in background threads via mpsc channels to keep the TUI responsive. The UI never freezes waiting for a network call.",
                  icon: "⚡",
                  color: "border-emerald-500/20 bg-gradient-to-r from-emerald-500/8 to-transparent",
                },
                {
                  title: "Retry with Backoff",
                  desc: "AI client retries transient failures with exponential backoff (2 retries) before surfacing an error to the user.",
                  icon: "🔄",
                  color: "border-blue-500/20 bg-gradient-to-r from-blue-500/8 to-transparent",
                },
              ].map((item, i) => (
                <motion.div
                  key={item.title}
                  initial={{ opacity: 0, x: -12 }}
                  whileInView={{ opacity: 1, x: 0 }}
                  viewport={{ once: true }}
                  transition={{ delay: i * 0.07 }}
                  className={`rounded-xl border p-4 ${item.color} flex gap-3`}
                >
                  <span className="text-lg shrink-0">{item.icon}</span>
                  <div>
                    <div className="font-bold text-white/80 text-sm mb-1">{item.title}</div>
                    <div className="text-white/40 text-xs leading-relaxed">{item.desc}</div>
                  </div>
                </motion.div>
              ))}
            </div>
          </section>

          {/* ───────────── DEVELOPMENT ───────────── */}
          <section className="relative mb-16">
            <SectionAnchor id="development" />
            <div className="flex items-center gap-3 mb-6">
              <div className="p-1.5 rounded-lg bg-orange-500/10 border border-orange-500/20">
                <Code2 size={16} className="text-orange-400" />
              </div>
              <h2 className="text-2xl font-black text-white">Development</h2>
              <div className="flex-1 h-px bg-gradient-to-r from-white/8 to-transparent" />
            </div>

            <SectionAnchor id="dev-building" />
            <h3 className="text-base font-bold text-white/80 mb-2">Building</h3>
            <CodeBlock
              code={`# Build (debug)
cargo build

# Build (release — stripped, LTO)
cargo build --release

# Run in debug mode
cargo run

# Lint
cargo clippy --all-targets -- -D warnings

# Format
cargo fmt --all

# Check everything (CI gate)
make check

# See all make targets
make help`}
              language="bash"
            />

            <SectionAnchor id="dev-testing" />
            <h3 className="text-base font-bold text-white/80 mt-8 mb-2">Testing</h3>
            <CodeBlock
              code={`# Run all Rust tests (143 unit + 35 integration)
cargo test --all-targets

# Run a single test by name
cargo test test_name

# Run Lambda tests (27 tests)
cd aws && python3 -m pytest tests/ -v`}
              language="bash"
            />

            <SectionAnchor id="dev-structure" />
            <h3 className="text-base font-bold text-white/80 mt-8 mb-3">Project Structure</h3>
            <CodeBlock
              code={`src/
├── main.rs            # Entry point, terminal setup, render loop
├── app.rs             # App state, view routing, async AI dispatch
├── config.rs          # Config loading (~/.config/zit/config.toml)
├── event.rs           # Keyboard/tick event handling
├── keychain.rs        # OS keychain integration
├── ai/
│   ├── client.rs      # AI client (retry, error classification, threads)
│   ├── prompts.rs     # AI prompt templates
│   └── provider.rs    # AI provider abstraction
├── git/
│   ├── runner.rs      # Core git command executor
│   ├── status.rs      # git status parser
│   ├── diff.rs        # git diff parser
│   ├── log.rs         # git log parser with graph support
│   ├── branch.rs      # Branch operations
│   ├── merge.rs       # Merge operations & conflict detection
│   ├── remote.rs      # Remote/push/pull operations
│   ├── stash.rs       # Stash operations
│   ├── reflog.rs      # Reflog parser
│   ├── bisect.rs      # Git bisect operations
│   ├── cherry_pick.rs # Cherry-pick operations
│   └── github_auth.rs # GitHub OAuth device flow
└── ui/
    ├── dashboard.rs       # Repository dashboard view
    ├── staging.rs         # Interactive staging view
    ├── commit.rs          # Commit editor view
    ├── branches.rs        # Branch manager view
    ├── timeline.rs        # Commit log/graph view
    ├── time_travel.rs     # Reset/restore view
    ├── reflog.rs          # Reflog viewer
    ├── stash.rs           # Stash manager view
    ├── merge_resolve.rs   # Merge conflict resolution view
    ├── bisect.rs          # Git bisect interactive view
    ├── cherry_pick.rs     # Cherry-pick interactive view
    ├── workflow_builder.rs # Workflow builder view
    ├── github.rs          # GitHub integration view
    ├── ai_mentor.rs       # AI Mentor panel
    ├── help.rs            # Context-sensitive help overlay
    └── utils.rs           # Shared UI utilities
aws/
├── deploy.sh          # One-command AWS SAM deployment
├── lambda/
│   ├── handler.py     # Lambda function (Bedrock integration)
│   └── prompts.py     # AI system prompts per request type
└── infrastructure/
    └── template.yaml  # SAM/CloudFormation template
website/               # Next.js marketing site`}
              language="text"
            />
          </section>

          {/* ───────────── TROUBLESHOOTING ───────────── */}
          <section className="relative mb-16">
            <SectionAnchor id="troubleshooting" />
            <div className="flex items-center gap-3 mb-6">
              <div className="p-1.5 rounded-lg bg-red-500/10 border border-red-500/20">
                <AlertTriangle size={16} className="text-red-400" />
              </div>
              <h2 className="text-2xl font-black text-white">Troubleshooting</h2>
              <div className="flex-1 h-px bg-gradient-to-r from-white/8 to-transparent" />
            </div>

            <SectionAnchor id="ts-windows" />
            <div className="glass rounded-xl border border-red-500/15 p-5 mb-6">
              <h3 className="text-sm font-bold text-red-300 mb-2 flex items-center gap-2">
                <span className="w-5 h-5 rounded-full bg-red-500/20 flex items-center justify-center text-xs text-red-400">1</span>
                Windows: <code className="font-mono text-red-400 text-xs bg-red-500/10 px-1.5 py-0.5 rounded">linker link.exe not found</code>
              </h3>
              <p className="text-white/40 text-sm leading-relaxed">
                Install{" "}
                <a href="https://visualstudio.microsoft.com/visual-cpp-build-tools/" target="_blank" className="text-orange-400 hover:underline">
                  Visual Studio Build Tools
                </a>{" "}
                and select the <strong className="text-white/60">&ldquo;Desktop development with C++&rdquo;</strong> workload.
              </p>
            </div>

            <SectionAnchor id="ts-ai" />
            <div className="glass rounded-xl border border-amber-500/15 p-5">
              <h3 className="text-sm font-bold text-amber-300 mb-4 flex items-center gap-2">
                <span className="w-5 h-5 rounded-full bg-amber-500/20 flex items-center justify-center text-xs text-amber-400">2</span>
                AI Not Working
              </h3>
              <ol className="space-y-3">
                {[
                  { label: "Check connectivity", detail: <span>Use Health Check in the AI Mentor panel (<Kbd>a</Kbd> → select Health Check)</span> },
                  { label: "Verify config", detail: <span>run <code className="text-white/60 font-mono text-xs bg-white/5 px-1.5 py-0.5 rounded">cat ~/.config/zit/config.toml</code> and ensure the <code className="text-white/60 font-mono text-xs">[ai]</code> section is present</span> },
                  { label: "Check env vars", detail: <code className="text-white/60 font-mono text-xs bg-white/5 px-1.5 py-0.5 rounded">echo $ZIT_AI_ENDPOINT $ZIT_AI_API_KEY</code> },
                  { label: "Check Lambda logs", detail: <code className="text-white/60 font-mono text-xs bg-white/5 px-1.5 py-0.5 rounded">aws logs tail /aws/lambda/zit-ai-mentor-dev --region ap-south-1</code> },
                ].map((step, i) => (
                  <li key={i} className="flex gap-3 text-sm">
                    <span className="w-5 h-5 rounded-full bg-white/5 border border-white/10 flex items-center justify-center text-xs text-white/40 shrink-0 mt-0.5">{i + 1}</span>
                    <span className="text-white/40">
                      <strong className="text-white/60">{step.label}:</strong>{" "}{step.detail}
                    </span>
                  </li>
                ))}
              </ol>
            </div>
          </section>

          {/* Footer CTA */}
          <div className="relative mt-4 mb-4">
            <div className="rounded-2xl border border-white/8 bg-gradient-to-br from-orange-500/5 via-transparent to-violet-500/5 p-8 text-center">
              <div className="text-3xl mb-3">🧡</div>
              <h3 className="text-xl font-black text-white mb-2">Ready to level up your Git workflow?</h3>
              <p className="text-white/40 text-sm mb-6">Install zit and experience Git like never before — right in your terminal.</p>
              <div className="flex items-center justify-center gap-3 flex-wrap">
                <Link
                  href="https://github.com/JUSTMEETPATEL/zit"
                  target="_blank"
                  className="flex items-center gap-2 px-5 py-2.5 rounded-xl glass border border-white/10 text-white/60 hover:text-white text-sm font-medium transition-all hover:border-white/20"
                >
                  <Github size={16} />
                  View on GitHub
                </Link>
                <Link
                  href="#installation"
                  className="px-5 py-2.5 rounded-xl bg-orange-500 hover:bg-orange-400 text-black font-bold text-sm transition-all shadow-[0_0_24px_rgba(249,115,22,0.3)] hover:shadow-[0_0_32px_rgba(249,115,22,0.5)]"
                >
                  Install zit →
                </Link>
              </div>
              <p className="text-white/15 text-xs mt-6 font-mono">Built with Rust & ❤️ · MIT License · Free &amp; Open Source</p>
            </div>
          </div>
        </main>
      </div>

      {/* Scroll to top */}
      <AnimatePresence>
        {showScrollTop && (
          <motion.button
            initial={{ opacity: 0, scale: 0.8 }}
            animate={{ opacity: 1, scale: 1 }}
            exit={{ opacity: 0, scale: 0.8 }}
            onClick={() => window.scrollTo({ top: 0, behavior: "smooth" })}
            className="fixed bottom-6 right-6 z-50 p-3 rounded-xl glass border border-white/10 text-white/50 hover:text-white shadow-lg backdrop-blur-xl transition-colors hover:border-white/20"
          >
            <ArrowUp size={18} />
          </motion.button>
        )}
      </AnimatePresence>
    </div>
  );
}
