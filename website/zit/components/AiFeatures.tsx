"use client";

import { motion } from "framer-motion";
import { Sparkles, FileSearch, MessageSquare, ShieldCheck, Stethoscope } from "lucide-react";
import { useState } from "react";

const capabilities = [
  {
    id: "explain",
    key: "a → Explain Repo",
    icon: FileSearch,
    title: "Explain Repo",
    desc: "AI reads your repository state and tells you exactly what's going on — in plain English.",
    response: "You're on `feat/ai-mentor`, 3 files modified. Last commit 2h ago — looks like you're building the authentication flow. 2 commits ahead of origin.",
  },
  {
    id: "ask",
    key: "a → Ask a Question",
    icon: MessageSquare,
    title: "Ask Anything",
    desc: "Ask complex Git questions and get accurate, context-aware answers. No Googling.",
    response: '`git reset --soft HEAD~1` — moves the commit pointer back one step but leaves your changes staged, ready to recommit.',
  },
  {
    id: "recommend",
    key: "a → Recommend",
    icon: ShieldCheck,
    title: "Safety Checks",
    desc: "Get AI warnings before destructive operations. It predicts what could go wrong.",
    response: "⚠️ Force-pushing to main will rewrite shared history. Consider `--force-with-lease` to protect teammates who may have pulled.",
  },
  {
    id: "commit",
    key: "Ctrl+G in Commit View",
    icon: Sparkles,
    title: "AI Commit Message",
    desc: "Press Ctrl+G in the commit editor — AI generates a commit message from your staged diff.",
    response: "feat(staging): add hunk-level diff preview with syntax highlighting\n\nImproves the interactive staging view to show colored diffs at the hunk level, making it easier to review changes before committing.",
  },
];

export default function AiFeatures() {
  const [active, setActive] = useState(capabilities[0]);

  return (
    <section id="ai-mentor" className="py-32 relative overflow-hidden">
      {/* Glow */}
      <div className="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-[700px] h-[400px] bg-violet-500/6 blur-[120px] rounded-full pointer-events-none" />

      <div className="relative z-10 max-w-6xl mx-auto px-6">
        <div className="flex flex-col lg:flex-row gap-16 items-start">
          {/* Left */}
          <div className="flex-1 lg:sticky lg:top-28">
            <motion.p
              initial={{ opacity: 0 }}
              whileInView={{ opacity: 1 }}
              viewport={{ once: true }}
              className="text-sm font-semibold text-violet-400 tracking-widest uppercase mb-4"
            >
              AI Mentor
            </motion.p>
            <motion.h2
              initial={{ opacity: 0, y: 16 }}
              whileInView={{ opacity: 1, y: 0 }}
              viewport={{ once: true }}
              transition={{ delay: 0.1 }}
              className="text-4xl md:text-5xl font-black tracking-tight mb-5 leading-[1.05]"
            >
              Git with a{" "}
              <span className="text-purple">teacher
              </span>{" "}
              built in.
            </motion.h2>
            <motion.p
              initial={{ opacity: 0 }}
              whileInView={{ opacity: 1 }}
              viewport={{ once: true }}
              transition={{ delay: 0.2 }}
              className="text-white/40 text-base leading-relaxed mb-10 max-w-sm"
            >
              Powered by Amazon Bedrock (Claude 3 Sonnet). The AI mentor explains errors
              automatically, answers questions, and generates commit messages — all without leaving your terminal.
            </motion.p>

            {/* Capability list */}
            <div className="space-y-2">
              {capabilities.map((cap) => {
                const Icon = cap.icon;
                return (
                  <button
                    key={cap.id}
                    onClick={() => setActive(cap)}
                    className={`w-full flex items-center gap-4 p-4 rounded-xl text-left transition-all duration-200 border ${
                      active.id === cap.id
                        ? "bg-violet-500/10 border-violet-500/20 text-white"
                        : "border-transparent hover:bg-white/3 text-white/40 hover:text-white/70"
                    }`}
                  >
                    <Icon size={18} className={active.id === cap.id ? "text-violet-400" : ""} />
                    <div>
                      <div className={`font-semibold text-sm ${active.id === cap.id ? "text-white" : ""}`}>{cap.title}</div>
                      <div className="text-xs font-mono opacity-50 mt-0.5">{cap.key}</div>
                    </div>
                  </button>
                );
              })}
            </div>
          </div>

          {/* Right — Demo terminal */}
          <motion.div
            key={active.id}
            initial={{ opacity: 0, x: 16 }}
            animate={{ opacity: 1, x: 0 }}
            transition={{ duration: 0.3 }}
            className="flex-1 w-full"
          >
            <div className="rounded-2xl bg-[#0d0d0d] border border-white/8 overflow-hidden shadow-2xl">
              {/* Titlebar */}
              <div className="flex items-center gap-2 px-5 py-4 border-b border-white/5">
                <div className="flex gap-1.5">
                  <div className="w-3 h-3 rounded-full bg-red-500/60" />
                  <div className="w-3 h-3 rounded-full bg-yellow-500/60" />
                  <div className="w-3 h-3 rounded-full bg-green-500/60" />
                </div>
                <span className="ml-3 text-xs text-white/20 font-mono">zit — AI Mentor</span>
                <div className="ml-auto flex items-center gap-1.5 text-[10px] text-violet-400/60 font-mono">
                  <span className="w-1.5 h-1.5 rounded-full bg-violet-400/60 animate-pulse" />
                  Claude 3 Sonnet
                </div>
              </div>

              {/* Chat body */}
              <div className="p-6 space-y-6 min-h-[320px] font-mono text-sm">
                {/* Description card */}
                <div className="flex items-start gap-3 p-4 rounded-xl bg-white/3 border border-white/5">
                  <div className="text-violet-400/60 text-xs mt-0.5">FEATURE</div>
                  <div>
                    <div className="text-white font-bold text-sm mb-1">{active.title}</div>
                    <p className="text-white/40 text-xs leading-relaxed">{active.desc}</p>
                  </div>
                </div>

                {/* AI response */}
                <div className="flex items-start gap-3">
                  <div className="w-7 h-7 rounded-full bg-violet-500/20 border border-violet-500/30 flex items-center justify-center shrink-0 mt-0.5">
                    <Sparkles size={13} className="text-violet-400" />
                  </div>
                  <div className="flex-1">
                    <div className="text-violet-400/70 text-[10px] font-semibold tracking-wider mb-2">ZIT AI</div>
                    <div className="text-white/70 bg-violet-500/5 border border-violet-500/10 rounded-xl px-4 py-3 leading-relaxed text-xs whitespace-pre-wrap">
                      {active.response}
                    </div>
                  </div>
                </div>
              </div>

              {/* Input */}
              <div className="px-6 pb-5">
                <div className="flex items-center gap-3 bg-white/3 border border-white/5 rounded-xl px-4 py-3">
                  <span className="text-white/20 text-xs">Ask anything about Git…</span>
                </div>
              </div>
            </div>
          </motion.div>
        </div>
      </div>
    </section>
  );
}
