"use client";

import { motion } from "framer-motion";
import {
  LayoutDashboard,
  GitPullRequest,
  FileDiff,
  GitCommit,
  Undo2,
  Archive,
  GitMerge,
  Cloud,
} from "lucide-react";

const features = [
  {
    icon: LayoutDashboard,
    title: "Repo Dashboard",
    desc: "Branch status, dirty state, and recent commits — your mission control at launch.",
    color: "text-orange-400",
    glow: "group-hover:shadow-[0_0_30px_rgba(249,115,22,0.12)]",
    border: "group-hover:border-orange-500/20",
  },
  {
    icon: FileDiff,
    title: "Smart Staging",
    desc: "Review diffs and stage at the hunk level. Never blindly `git add .` again.",
    color: "text-emerald-400",
    glow: "group-hover:shadow-[0_0_30px_rgba(52,211,153,0.12)]",
    border: "group-hover:border-emerald-500/20",
  },
  {
    icon: GitCommit,
    title: "AI Commits",
    desc: "Write commit messages with validation — or let AI generate one from your diff.",
    color: "text-violet-400",
    glow: "group-hover:shadow-[0_0_30px_rgba(139,92,246,0.12)]",
    border: "group-hover:border-violet-500/20",
  },
  {
    icon: GitPullRequest,
    title: "Visual Branching",
    desc: "Create, switch, delete, and rename branches with a clean visual tree.",
    color: "text-blue-400",
    glow: "group-hover:shadow-[0_0_30px_rgba(96,165,250,0.12)]",
    border: "group-hover:border-blue-500/20",
  },
  {
    icon: Undo2,
    title: "Time Travel",
    desc: "Safe reset and restore — soft, mixed, or hard — with clear confirmation dialogs.",
    color: "text-red-400",
    glow: "group-hover:shadow-[0_0_30px_rgba(248,113,113,0.12)]",
    border: "group-hover:border-red-500/20",
  },
  {
    icon: Archive,
    title: "Stash Manager",
    desc: "Save, pop, apply, drop stashes — without memorizing a single command.",
    color: "text-amber-400",
    glow: "group-hover:shadow-[0_0_30px_rgba(251,191,36,0.12)]",
    border: "group-hover:border-amber-500/20",
  },
  {
    icon: GitMerge,
    title: "Merge Resolve",
    desc: "Resolve conflicts visually — pick ours, theirs, or let AI suggest the merge.",
    color: "text-cyan-400",
    glow: "group-hover:shadow-[0_0_30px_rgba(34,211,238,0.12)]",
    border: "group-hover:border-cyan-500/20",
  },
  {
    icon: Cloud,
    title: "GitHub Integration",
    desc: "OAuth, push/pull, create PRs, and trigger CI/CD — all without leaving the TUI.",
    color: "text-indigo-400",
    glow: "group-hover:shadow-[0_0_30px_rgba(129,140,248,0.12)]",
    border: "group-hover:border-indigo-500/20",
  },
];

export default function Features() {
  return (
    <section id="features" className="py-32 relative">
      <div className="max-w-6xl mx-auto px-6">
        {/* Header */}
        <div className="text-center mb-20">
          <motion.p
            initial={{ opacity: 0 }}
            whileInView={{ opacity: 1 }}
            viewport={{ once: true }}
            className="text-sm font-semibold text-orange-400 tracking-widest uppercase mb-4"
          >
            Features
          </motion.p>
          <motion.h2
            initial={{ opacity: 0, y: 16 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            transition={{ delay: 0.1 }}
            className="text-4xl md:text-5xl font-black tracking-tight mb-5"
          >
            Everything you need.{" "}
            <span className="text-white/25">Nothing you don't.</span>
          </motion.h2>
          <motion.p
            initial={{ opacity: 0 }}
            whileInView={{ opacity: 1 }}
            viewport={{ once: true }}
            transition={{ delay: 0.2 }}
            className="text-white/40 text-lg max-w-xl mx-auto"
          >
            14 Git features. One binary. Zero GUI bloat.
          </motion.p>
        </div>

        {/* Grid */}
        <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
          {features.map((f, i) => {
            const Icon = f.icon;
            return (
              <motion.div
                key={f.title}
                initial={{ opacity: 0, y: 20 }}
                whileInView={{ opacity: 1, y: 0 }}
                viewport={{ once: true }}
                transition={{ delay: i * 0.07 }}
                className={`group relative p-6 rounded-2xl glass border border-white/5 transition-all duration-300 cursor-default ${f.glow} ${f.border} hover:-translate-y-1`}
              >
                <div className={`${f.color} mb-4 opacity-80 group-hover:opacity-100 transition-opacity`}>
                  <Icon size={22} />
                </div>
                <h3 className="text-white font-bold text-sm mb-2">{f.title}</h3>
                <p className="text-white/35 text-xs leading-relaxed">{f.desc}</p>
              </motion.div>
            );
          })}
        </div>

        {/* Bottom extras row */}
        <motion.div
          initial={{ opacity: 0 }}
          whileInView={{ opacity: 1 }}
          viewport={{ once: true }}
          transition={{ delay: 0.4 }}
          className="mt-6 flex flex-wrap items-center justify-center gap-3"
        >
          {["Commit Timeline", "Git Bisect", "Cherry Pick", "Workflow Builder", "Reflog Recovery"].map((tag) => (
            <span
              key={tag}
              className="px-3 py-1.5 rounded-full text-xs text-white/30 border border-white/5 glass font-mono"
            >
              + {tag}
            </span>
          ))}
        </motion.div>
      </div>
    </section>
  );
}
