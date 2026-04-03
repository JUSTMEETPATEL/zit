"use client";

import { motion } from "framer-motion";
import { useState } from "react";

const bindings = [
  { key: "s", label: "Stage", color: "emerald" },
  { key: "c", label: "Commit", color: "orange" },
  { key: "b", label: "Branch", color: "purple" },
  { key: "l", label: "Log", color: "blue" },
  { key: "t", label: "Time Travel", color: "red" },
  { key: "r", label: "Reflog", color: "amber" },
  { key: "x", label: "Stash", color: "indigo" },
  { key: "m", label: "Merge", color: "cyan" },
  { key: "B", label: "Bisect", color: "rose" },
  { key: "p", label: "Cherry Pick", color: "pink" },
  { key: "w", label: "Workflow", color: "teal" },
  { key: "g", label: "GitHub", color: "slate" },
  { key: "a", label: "AI Mentor", color: "violet" },
  { key: "?", label: "Help", color: "neutral" },
  { key: "q", label: "Quit", color: "neutral" },
];

const descriptions: Record<string, string> = {
  s: "Interactive file staging — review diffs, stage individual hunks, search files.",
  c: "Commit editor with subject/body validation. Press Ctrl+G for AI-generated message.",
  b: "Create, switch, delete, rename branches. Toggle local/remote view.",
  l: "Visual commit timeline with graph rendering and full-text search.",
  t: "Safe reset and restore — soft, mixed, or hard — with confirmation dialogs.",
  r: "Browse reflog and recover commits that seem lost.",
  x: "Save, pop, apply, drop, and clear stashes without memorizing syntax.",
  m: "Conflict resolution with ours/theirs/manual/AI-assisted merge options.",
  B: "Interactive git bisect — binary search for the commit that introduced a bug.",
  p: "Multi-select cherry-pick — pick individual commits from any branch.",
  w: "Visual workflow builder — compose and run multi-step git workflows.",
  g: "GitHub integration — OAuth, push/pull/sync, PRs, CI/CD actions.",
  a: "AI Mentor panel — explain repo, ask questions, get recommendations.",
  "?": "Context-sensitive help overlay — shows available keys for the current view.",
  q: "Exit zit gracefully.",
};

const colorMap: Record<string, string> = {
  emerald: "border-emerald-500/30 text-emerald-400 hover:bg-emerald-500 hover:text-black hover:border-emerald-400",
  orange: "border-orange-500/30 text-orange-400 hover:bg-orange-500 hover:text-black hover:border-orange-400",
  purple: "border-purple-500/30 text-purple-400 hover:bg-purple-500 hover:text-black",
  blue: "border-blue-500/30 text-blue-400 hover:bg-blue-500 hover:text-black",
  red: "border-red-500/30 text-red-400 hover:bg-red-500 hover:text-black",
  amber: "border-amber-500/30 text-amber-400 hover:bg-amber-500 hover:text-black",
  indigo: "border-indigo-500/30 text-indigo-400 hover:bg-indigo-500 hover:text-black",
  cyan: "border-cyan-500/30 text-cyan-400 hover:bg-cyan-500 hover:text-black",
  rose: "border-rose-500/30 text-rose-400 hover:bg-rose-500 hover:text-black",
  pink: "border-pink-500/30 text-pink-400 hover:bg-pink-500 hover:text-black",
  teal: "border-teal-500/30 text-teal-400 hover:bg-teal-500 hover:text-black",
  slate: "border-slate-400/30 text-slate-300 hover:bg-slate-500 hover:text-black",
  violet: "border-violet-500/30 text-violet-400 hover:bg-violet-500 hover:text-black",
  neutral: "border-white/10 text-white/30 hover:bg-white/20 hover:text-white",
};

export default function Keybindings() {
  const [hovered, setHovered] = useState<string | null>(null);
  const active = bindings.find((b) => b.key === hovered);

  return (
    <section id="keybindings" className="py-32 relative overflow-hidden">
      <div className="max-w-4xl mx-auto px-6">
        {/* Header */}
        <div className="text-center mb-16">
          <motion.p
            initial={{ opacity: 0 }}
            whileInView={{ opacity: 1 }}
            viewport={{ once: true }}
            className="text-sm font-semibold text-orange-400 tracking-widest uppercase mb-4"
          >
            Keybindings
          </motion.p>
          <motion.h2
            initial={{ opacity: 0, y: 16 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            transition={{ delay: 0.1 }}
            className="text-4xl md:text-5xl font-black tracking-tight mb-5"
          >
            Keyboard-first.{" "}
            <span className="text-white/25">Always.</span>
          </motion.h2>
          <motion.p
            initial={{ opacity: 0 }}
            whileInView={{ opacity: 1 }}
            viewport={{ once: true }}
            transition={{ delay: 0.2 }}
            className="text-white/40 text-lg"
          >
            Hover any key to learn what it does.
          </motion.p>
        </div>

        {/* Key grid */}
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          transition={{ delay: 0.2 }}
          className="flex flex-wrap justify-center gap-3 mb-10"
        >
          {bindings.map((b, i) => (
            <motion.div
              key={b.key}
              initial={{ opacity: 0, scale: 0.8 }}
              whileInView={{ opacity: 1, scale: 1 }}
              viewport={{ once: true }}
              transition={{ delay: i * 0.04 }}
              onMouseEnter={() => setHovered(b.key)}
              onMouseLeave={() => setHovered(null)}
              className={`w-14 h-14 rounded-xl border glass font-mono font-black text-lg flex flex-col items-center justify-center gap-0.5 cursor-default transition-all duration-150 ${colorMap[b.color]} ${
                hovered === b.key ? "scale-110 -translate-y-1" : "hover:scale-105"
              }`}
            >
              {b.key}
            </motion.div>
          ))}
        </motion.div>

        {/* Description tooltip */}
        <motion.div
          animate={{ opacity: active ? 1 : 0, y: active ? 0 : 8 }}
          transition={{ duration: 0.2 }}
          className="h-16 flex flex-col items-center justify-center text-center px-4"
        >
          {active && (
            <>
              <div className="text-white font-bold text-sm mb-1">{active.label}</div>
              <div className="text-white/35 text-xs max-w-md">{descriptions[active.key]}</div>
            </>
          )}
          {!active && (
            <p className="text-white/15 text-xs font-mono">Hover a key ↑</p>
          )}
        </motion.div>
      </div>
    </section>
  );
}
