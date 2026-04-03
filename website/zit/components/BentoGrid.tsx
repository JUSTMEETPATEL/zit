"use client";

import { motion } from "framer-motion";
import {
  GitCommit,
  GitPullRequest,
  LayoutDashboard,
  History,
  Undo2,
  Wand2,
  FileDiff,
  Cloud,
  ChevronRight,
  Archive,
  GitMerge,
  Bug,
  MousePointerClick,
  Workflow,
  Clock,
  PlaySquare
} from "lucide-react";
import Link from "next/link";

export default function BentoGrid() {
  return (
    <section
      id="features"
      className="py-24 relative max-w-7xl mx-auto px-4 sm:px-6 lg:px-8"
    >
      <div className="absolute inset-0 bg-[url('https://grainy-gradients.vercel.app/noise.svg')] opacity-5 mix-blend-overlay pointer-events-none"></div>
      
      <div className="text-center mb-16 relative z-10">
        <h2 className="text-4xl md:text-6xl font-extrabold mb-4 tracking-tight">
          Every Git Feature. <span className="text-transparent bg-clip-text bg-gradient-to-r from-orange-500 to-amber-500">Zero GUI Bloat.</span>
        </h2>
        <p className="text-gray-400 max-w-2xl mx-auto text-lg">
          A terminal user interface that feels like a modern application. Unmatched speed, absolute control, and 14 powerful features built right in.
        </p>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-4 lg:grid-cols-4 auto-rows-[minmax(180px,auto)] gap-5 relative z-10">
        {/* Row 1 & 2 */}
        {/* Box 1: Repository Dashboard (Large 2x2) */}
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          className="col-span-1 md:col-span-2 row-span-2 rounded-2xl p-6 bg-zinc-900/40 border border-white/5 hover:border-orange-500/30 transition-all duration-300 group overflow-hidden relative backdrop-blur-sm"
        >
          <div className="absolute top-0 right-0 p-6 opacity-5 group-hover:opacity-10 transition-opacity duration-500 transform group-hover:scale-110">
            <LayoutDashboard size={140} />
          </div>
          <div className="relative z-10 h-full flex flex-col justify-between">
            <div>
              <div className="p-3 bg-orange-500/10 w-fit rounded-xl mb-4 text-orange-500 ring-1 ring-orange-500/20 shadow-[0_0_15px_rgba(249,115,22,0.1)]">
                <LayoutDashboard />
              </div>
              <h3 className="text-2xl font-bold text-white mb-2">
                Repository Dashboard
              </h3>
              <p className="text-gray-400 max-w-sm">
                Your mission control center. See dirty state, recent commits,
                and branch status instantly upon launch.
              </p>
            </div>

            <div className="mt-8 bg-black/60 rounded-xl border border-white/5 p-4 font-mono text-xs text-gray-300 shadow-2xl translate-y-2 group-hover:translate-y-0 transition-transform duration-300 backdrop-blur-md">
              <div className="flex justify-between border-b border-white/5 pb-2 mb-2 items-center">
                <div className="flex gap-2">
                  <div className="w-2.5 h-2.5 rounded-full bg-red-500"></div>
                  <div className="w-2.5 h-2.5 rounded-full bg-yellow-500"></div>
                  <div className="w-2.5 h-2.5 rounded-full bg-green-500"></div>
                </div>
                <span className="font-bold text-white/80">zit dashboard</span>
              </div>
              <div className="grid grid-cols-2 gap-4">
                <div>
                  <div className="text-gray-500 mb-1">BRANCH</div>
                  <div className="text-green-400 font-bold text-lg flex items-center gap-1">
                    <GitPullRequest size={14} /> feat/ai-mentor
                  </div>
                </div>
                <div>
                  <div className="text-gray-500 mb-1">STATUS</div>
                  <div className="text-yellow-400">3 modified files</div>
                </div>
              </div>
            </div>
          </div>
        </motion.div>

        {/* Box 2: Visual Branching (Tall 1x2) */}
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          transition={{ delay: 0.1 }}
          className="col-span-1 md:col-span-2 lg:col-span-1 row-span-2 rounded-2xl p-6 bg-zinc-900/40 border border-white/5 hover:border-purple-500/30 transition-all duration-300 group relative overflow-hidden backdrop-blur-sm"
        >
          <div className="p-3 bg-purple-500/10 w-fit rounded-xl mb-4 text-purple-400 ring-1 ring-purple-500/20 shadow-[0_0_15px_rgba(168,85,247,0.1)]">
            <GitPullRequest />
          </div>
          <h3 className="text-xl font-bold text-white mb-2">
            Visual Branching
          </h3>
          <p className="text-gray-400 text-sm mb-6 relative z-10">
            Create, switch, delete, and rename branches visually.
          </p>

          <div className="absolute bottom-0 left-6 right-6 top-40 flex flex-col gap-3 font-mono text-xs opacity-60 group-hover:opacity-100 transition-opacity duration-300">
            <div className="flex items-center gap-2">
              <div className="w-2 h-2 rounded-full bg-gray-500 shadow-[0_0_5px_rgba(156,163,175,0.5)]"></div>
              <span className="text-gray-400">main</span>
            </div>
            <div className="flex items-center gap-2 ml-4">
              <div className="w-2 h-8 border-l mb-[-16px] border-b border-gray-700 rounded-bl-lg"></div>
            </div>
            <div className="flex items-center gap-2 ml-4">
              <div className="w-2 h-2 rounded-full bg-green-500 shadow-[0_0_8px_rgba(34,197,94,0.6)]"></div>
              <span className="text-green-400 font-bold bg-green-500/10 px-2 py-0.5 border border-green-500/20 rounded">
                dev
              </span>
            </div>
            <div className="flex items-center gap-2 ml-8">
              <div className="w-2 h-8 border-l mb-[-16px] border-b border-gray-700 rounded-bl-lg"></div>
            </div>
            <div className="flex items-center gap-2 ml-8">
              <div className="w-2 h-2 rounded-full bg-blue-400 shadow-[0_0_8px_rgba(96,165,250,0.6)]"></div>
              <span className="text-blue-300">feature/ui</span>
            </div>
          </div>
        </motion.div>

        {/* Box 3: Commit Timeline (Tall 1x2) */}
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          transition={{ delay: 0.15 }}
          className="col-span-1 md:col-span-2 lg:col-span-1 row-span-2 rounded-2xl p-6 bg-zinc-900/40 border border-white/5 hover:border-blue-500/30 transition-all duration-300 group relative overflow-hidden backdrop-blur-sm"
        >
          <div className="p-3 bg-blue-500/10 w-fit rounded-xl mb-4 text-blue-400 ring-1 ring-blue-500/20 shadow-[0_0_15px_rgba(59,130,246,0.1)]">
            <Clock />
          </div>
          <h3 className="text-xl font-bold text-white mb-2">
            Commit Timeline
          </h3>
          <p className="text-gray-400 text-sm">
            Browse git log with a visual commit graph and robust search.
          </p>
          <div className="mt-6 flex flex-col gap-2 font-mono text-[10px] text-gray-500 group-hover:text-gray-400 transition-colors">
            <div className="flex items-start gap-2">
              <div className="text-blue-400">*</div>
              <div>Fix bento grid layout issues</div>
            </div>
            <div className="flex items-start gap-2">
              <div className="text-red-400">| \</div>
            </div>
            <div className="flex items-start gap-2">
              <div className="text-blue-400">| *</div>
              <div>Add AI mentor setup</div>
            </div>
            <div className="flex items-start gap-2">
              <div className="text-green-400">* |</div>
              <div>Implement stash manager</div>
            </div>
          </div>
        </motion.div>

        {/* Row 3 */}
        {/* Box 4: AI Mentor (Wide 2x1) */}
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          transition={{ delay: 0.2 }}
          className="col-span-1 md:col-span-4 lg:col-span-2 rounded-2xl p-6 bg-gradient-to-br from-[var(--accent)]/10 via-zinc-900/80 to-black border border-[var(--accent)]/30 hover:shadow-[0_0_30px_rgba(139,92,246,0.15)] transition-all duration-300 group relative overflow-hidden"
        >
          {/* Animated glow background */}
          <div className="absolute -inset-20 bg-[var(--accent)]/5 blur-3xl rounded-full opacity-0 group-hover:opacity-100 transition-opacity duration-700"></div>
          
          <div className="flex justify-between items-start relative z-10 h-full">
            <div className="flex flex-col justify-between h-full">
              <div>
                <div className="p-3 bg-[var(--accent)]/20 w-fit rounded-xl mb-4 text-[var(--accent)] ring-1 ring-[var(--accent)]/30 backdrop-blur-md">
                  <Wand2 />
                </div>
                <h3 className="text-2xl font-bold text-white mb-2">AI Mentor</h3>
                <p className="text-gray-300 text-sm max-w-sm">
                  Explanations, recommendations, and automatic error help right in your terminal.
                </p>
              </div>
              <div className="mt-4 font-mono text-xs text-[var(--accent)]/80 flex items-center gap-2">
                <span className="w-2 h-2 rounded-full bg-[var(--accent)] animate-pulse"></span>
                Amazon Bedrock Claude 3 Powered
              </div>
            </div>
            <Link
              href="#ai-mentor"
              className="mt-2 bg-white/5 hover:bg-[var(--accent)]/20 p-3 rounded-full border border-white/10 hover:border-[var(--accent)]/50 transition-all text-white/50 hover:text-white"
            >
              <ChevronRight size={20} />
            </Link>
          </div>
        </motion.div>

        {/* Box 5: Smart Staging */}
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          transition={{ delay: 0.3 }}
          className="col-span-1 rounded-2xl p-6 bg-zinc-900/40 border border-white/5 hover:border-emerald-500/30 hover:-translate-y-1 transition-all duration-300 backdrop-blur-sm"
        >
          <div className="p-3 bg-emerald-500/10 w-fit rounded-xl mb-4 text-emerald-400 ring-1 ring-emerald-500/20">
            <FileDiff />
          </div>
          <h3 className="font-bold text-white mb-2">Smart Staging</h3>
          <p className="text-gray-400 text-sm">
            Stop `git add .`. Review diffs, stage hunks interactively.
          </p>
        </motion.div>

        {/* Box 6: Guided Commits */}
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          transition={{ delay: 0.4 }}
          className="col-span-1 rounded-2xl p-6 bg-zinc-900/40 border border-white/5 hover:border-amber-500/30 hover:-translate-y-1 transition-all duration-300 backdrop-blur-sm"
        >
          <div className="p-3 bg-amber-500/10 w-fit rounded-xl mb-4 text-amber-400 ring-1 ring-amber-500/20">
            <GitCommit />
          </div>
          <h3 className="font-bold text-white mb-2">Guided Commits</h3>
          <p className="text-gray-400 text-sm">
            Validate subjects, write bodies, or auto-generate via AI.
          </p>
        </motion.div>

        {/* Row 4 */}
        {/* Box 7: Time Travel */}
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          transition={{ delay: 0.5 }}
          className="col-span-1 rounded-2xl p-6 bg-zinc-900/40 border border-white/5 hover:border-red-500/30 hover:-translate-y-1 transition-all duration-300 backdrop-blur-sm"
        >
          <div className="p-3 bg-red-500/10 w-fit rounded-xl mb-4 text-red-500 ring-1 ring-red-500/20">
            <Undo2 />
          </div>
          <h3 className="font-bold text-white mb-2">Time Travel</h3>
          <p className="text-gray-400 text-sm">
            Safe reset/restore (soft, mixed, hard) with clear confirmations.
          </p>
        </motion.div>

        {/* Box 8: Merge Resolve */}
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          transition={{ delay: 0.6 }}
          className="col-span-1 rounded-2xl p-6 bg-zinc-900/40 border border-white/5 hover:border-cyan-500/30 hover:-translate-y-1 transition-all duration-300 backdrop-blur-sm"
        >
          <div className="p-3 bg-cyan-500/10 w-fit rounded-xl mb-4 text-cyan-400 ring-1 ring-cyan-500/20">
            <GitMerge />
          </div>
          <h3 className="font-bold text-white mb-2">Merge Resolve</h3>
          <p className="text-gray-400 text-sm">
            Visual conflict resolution with ours/theirs or AI-assist.
          </p>
        </motion.div>

        {/* Box 9: Git Bisect (Wide) */}
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          transition={{ delay: 0.7 }}
          className="col-span-1 md:col-span-2 rounded-2xl p-6 bg-zinc-900/40 border border-white/5 hover:border-rose-500/30 hover:-translate-y-1 transition-all duration-300 flex items-center gap-6 backdrop-blur-sm"
        >
          <div className="p-4 bg-rose-500/10 rounded-xl text-rose-400 ring-1 ring-rose-500/20 shrink-0">
            <Bug size={32} />
          </div>
          <div>
            <h3 className="font-bold text-white mb-1">Git Bisect</h3>
            <p className="text-gray-400 text-sm">
              Interactive binary search for bug-introducing commits without leaving the UI.
            </p>
          </div>
        </motion.div>

        {/* Row 5 */}
        {/* Box 10: Stash Manager */}
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          transition={{ delay: 0.8 }}
          className="col-span-1 rounded-2xl p-6 bg-zinc-900/40 border border-white/5 hover:border-indigo-500/30 hover:-translate-y-1 transition-all duration-300 backdrop-blur-sm"
        >
          <div className="p-3 bg-indigo-500/10 w-fit rounded-xl mb-4 text-indigo-400 ring-1 ring-indigo-500/20">
            <Archive />
          </div>
          <h3 className="font-bold text-white mb-2">Stash Manager</h3>
          <p className="text-gray-400 text-sm">
            Save, pop, apply, drop, and clear stashes intuitively.
          </p>
        </motion.div>

        {/* Box 11: Cherry Pick */}
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          transition={{ delay: 0.9 }}
          className="col-span-1 rounded-2xl p-6 bg-zinc-900/40 border border-white/5 hover:border-pink-500/30 hover:-translate-y-1 transition-all duration-300 backdrop-blur-sm"
        >
          <div className="p-3 bg-pink-500/10 w-fit rounded-xl mb-4 text-pink-400 ring-1 ring-pink-500/20">
            <MousePointerClick />
          </div>
          <h3 className="font-bold text-white mb-2">Cherry Pick</h3>
          <p className="text-gray-400 text-sm">
            Pick commits from other branches with multi-select capabilities.
          </p>
        </motion.div>

        {/* Box 12: Workflow Builder */}
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          transition={{ delay: 1.0 }}
          className="col-span-1 md:col-span-2 rounded-2xl p-6 bg-zinc-900/40 border border-white/5 hover:border-emerald-400/30 hover:-translate-y-1 transition-all duration-300 flex items-center gap-6 backdrop-blur-sm"
        >
          <div className="p-4 bg-emerald-400/10 rounded-xl text-emerald-400 ring-1 ring-emerald-400/20 shrink-0">
            <Workflow size={32} />
          </div>
          <div>
            <h3 className="font-bold text-white mb-1">Workflow Builder</h3>
            <p className="text-gray-400 text-sm">
              Visually compose multi-step git workflows to automate repetitive team processes.
            </p>
          </div>
        </motion.div>

        {/* Box 13: GitHub & Reflog (Wide bottom span) */}
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          transition={{ delay: 1.1 }}
          className="col-span-1 md:col-span-4 rounded-2xl p-6 bg-zinc-900/30 border border-white/5 flex flex-col md:flex-row justify-around items-center gap-6"
        >
          <div className="flex items-center gap-4 text-gray-400 hover:text-white transition-colors group">
            <div className="p-3 bg-white/5 rounded-full group-hover:bg-white/10 transition-colors">
              <Cloud size={24} />
            </div>
            <div>
              <span className="block font-bold text-white">GitHub Integration</span>
              <span className="text-sm">PRs, Actions, Push/Pull natively</span>
            </div>
          </div>
          <div className="hidden md:block w-px h-12 bg-white/10"></div>
          <div className="flex items-center gap-4 text-gray-400 hover:text-white transition-colors group">
            <div className="p-3 bg-white/5 rounded-full group-hover:bg-white/10 transition-colors">
              <History size={24} />
            </div>
            <div>
              <span className="block font-bold text-white">Reflog Recovery</span>
              <span className="text-sm">Recover lost commits easily</span>
            </div>
          </div>
        </motion.div>
      </div>
    </section>
  );
}
