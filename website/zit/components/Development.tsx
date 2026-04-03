"use client";

import { motion } from "framer-motion";
import { FolderTree, Wrench } from "lucide-react";

const structure = [
  { name: "src/main.rs", desc: "Entry point, terminal setup, render loop" },
  { name: "src/app.rs", desc: "App state, view routing, AI dispatch" },
  { name: "src/config.rs", desc: "Config loading (~/.config/zit/config.toml)" },
  { name: "src/event.rs", desc: "Keyboard & tick event handling" },
  { name: "src/keychain.rs", desc: "macOS Keychain integration" },
  { name: "src/ai/", desc: "AI client, prompts, provider abstraction" },
  { name: "src/git/", desc: "All git ops — runner, diff, log, branch…" },
  { name: "src/ui/", desc: "14 TUI views + help overlay + utils" },
  { name: "aws/", desc: "Lambda backend & SAM/CloudFormation infra" },
  { name: "website/", desc: "Next.js marketing site (this page!)" },
];

const commands = [
  { cmd: "cargo build", comment: "# Build debug binary" },
  { cmd: "cargo run", comment: "# Run in debug mode" },
  { cmd: "make check", comment: "# Format + clippy + test (CI gate)" },
  { cmd: "cargo test --all-targets", comment: "# 178 Rust tests (143 unit + 35 integration)" },
  { cmd: "cd aws && python3 -m pytest tests/ -v", comment: "# 27 Lambda tests" },
  { cmd: "cargo clippy --all-targets -- -D warnings", comment: "# Lint" },
  { cmd: "cargo fmt --all", comment: "# Format" },
  { cmd: "cargo build --release", comment: "# Release build (stripped, LTO)" },
  { cmd: "make help", comment: "# See all make targets" },
];

export default function Development() {
  return (
    <section id="development" className="py-24 border-t border-white/5 relative">
      <div className="absolute inset-0 bg-[radial-gradient(ellipse_60%_40%_at_50%_100%,rgba(249,115,22,0.04),transparent)] pointer-events-none"></div>

      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 relative z-10">
        <div className="text-center mb-14">
          <h2 className="text-4xl md:text-5xl font-extrabold mb-4 tracking-tight">
            Development &{" "}
            <span className="text-transparent bg-clip-text bg-gradient-to-r from-orange-400 to-amber-500">
              Structure
            </span>
          </h2>
          <p className="text-gray-400 text-lg max-w-xl mx-auto">
            Built with Rust for reliability and performance. Contributions welcome!
          </p>
        </div>

        <div className="grid md:grid-cols-2 gap-8">
          {/* Project Structure */}
          <motion.div
            initial={{ opacity: 0, x: -20 }}
            whileInView={{ opacity: 1, x: 0 }}
            viewport={{ once: true }}
            className="bg-[#0c0c0c] border border-white/10 rounded-2xl overflow-hidden"
          >
            <div className="bg-zinc-900 px-5 py-3.5 border-b border-white/5 flex items-center gap-2.5">
              <FolderTree size={15} className="text-orange-400" />
              <span className="text-sm font-semibold text-white">Project Structure</span>
            </div>
            <div className="p-5">
              <ul className="space-y-1 font-mono text-sm">
                {structure.map((item, i) => (
                  <motion.li
                    key={i}
                    initial={{ opacity: 0 }}
                    whileInView={{ opacity: 1 }}
                    viewport={{ once: true }}
                    transition={{ delay: i * 0.04 }}
                    className="flex items-center gap-2 text-gray-400 hover:text-white group transition-colors py-1.5 px-2 rounded-lg hover:bg-white/3"
                  >
                    <span className="text-orange-500/50 text-xs select-none">›</span>
                    <span className="text-gray-200 shrink-0 group-hover:text-orange-400 transition-colors">{item.name}</span>
                    <span className="flex-1 border-b border-dashed border-white/5"></span>
                    <span className="text-gray-600 text-xs text-right max-w-[160px] truncate">{item.desc}</span>
                  </motion.li>
                ))}
              </ul>
            </div>
          </motion.div>

          {/* Dev commands */}
          <motion.div
            initial={{ opacity: 0, x: 20 }}
            whileInView={{ opacity: 1, x: 0 }}
            viewport={{ once: true }}
            className="bg-[#0c0c0c] border border-white/10 rounded-2xl overflow-hidden"
          >
            <div className="bg-zinc-900 px-5 py-3.5 border-b border-white/5 flex items-center gap-2.5">
              <Wrench size={15} className="text-orange-400" />
              <span className="text-sm font-semibold text-white">Dev Workflow Commands</span>
            </div>
            <div className="divide-y divide-white/5 font-mono text-sm">
              {commands.map((c, i) => (
                <motion.div
                  key={i}
                  initial={{ opacity: 0 }}
                  whileInView={{ opacity: 1 }}
                  viewport={{ once: true }}
                  transition={{ delay: i * 0.05 }}
                  className="group flex flex-col gap-0.5 px-5 py-3 hover:bg-white/3 transition-colors"
                >
                  <div className="flex items-center gap-2">
                    <span className="text-green-400 select-none shrink-0">$</span>
                    <code className="text-gray-200 group-hover:text-white transition-colors break-all">{c.cmd}</code>
                  </div>
                  <span className="text-xs text-gray-600 pl-4 group-hover:text-gray-500 transition-colors">{c.comment}</span>
                </motion.div>
              ))}
            </div>
          </motion.div>
        </div>

        {/* Prereqs row */}
        <div className="mt-8 grid grid-cols-2 md:grid-cols-4 gap-4">
          {[
            { name: "Rust", detail: "Stable toolchain (rustup)", color: "text-orange-400" },
            { name: "Git", detail: "Any recent version", color: "text-green-400" },
            { name: "Python 3.12", detail: "For Lambda tests", color: "text-blue-400" },
            { name: "C++ Tools", detail: "Windows only", color: "text-gray-400" },
          ].map((req) => (
            <div
              key={req.name}
              className="p-4 bg-white/3 border border-white/5 rounded-xl text-center hover:border-white/10 transition-all"
            >
              <div className={`font-bold text-sm ${req.color} mb-0.5`}>{req.name}</div>
              <div className="text-gray-600 text-xs">{req.detail}</div>
            </div>
          ))}
        </div>
      </div>
    </section>
  );
}
