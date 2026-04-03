"use client";

import { Check, Copy } from "lucide-react";
import { useState } from "react";
import { motion } from "framer-motion";

type OS = "brew" | "cargo";

export default function Installation() {
  const [copied, setCopied] = useState("");
  const [os, setOs] = useState<OS>("brew");

  const commands: Record<OS, { lines: string[]; hint: string }> = {
    brew: {
      lines: ["brew tap JUSTMEETPATEL/zit", "brew install zit"],
      hint: "Recommended for macOS. Auto-updates with brew upgrade.",
    },
    cargo: {
      lines: ["cargo install --git https://github.com/JUSTMEETPATEL/zit"],
      hint: "Requires Rust toolchain. Works on macOS, Linux, Windows.",
    },
  };

  const handleCopy = (key: string) => {
    navigator.clipboard.writeText(commands[key as OS].lines.join("\n"));
    setCopied(key);
    setTimeout(() => setCopied(""), 2000);
  };

  return (
    <section id="installation" className="py-32 relative">
      <div className="absolute bottom-0 left-1/2 -translate-x-1/2 w-[500px] h-[200px] bg-orange-500/5 blur-[100px] rounded-full pointer-events-none" />

      <div className="max-w-4xl mx-auto px-6 relative z-10">
        {/* Header */}
        <div className="text-center mb-16">
          <motion.p
            initial={{ opacity: 0 }}
            whileInView={{ opacity: 1 }}
            viewport={{ once: true }}
            className="text-sm font-semibold text-orange-400 tracking-widest uppercase mb-4"
          >
            Install
          </motion.p>
          <motion.h2
            initial={{ opacity: 0, y: 16 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            transition={{ delay: 0.1 }}
            className="text-4xl md:text-5xl font-black tracking-tight mb-5"
          >
            Up and running{" "}
            <span className="text-orange">in 30 seconds.</span>
          </motion.h2>
          <motion.p
            initial={{ opacity: 0 }}
            whileInView={{ opacity: 1 }}
            viewport={{ once: true }}
            transition={{ delay: 0.2 }}
            className="text-white/40 text-lg"
          >
            No configuration required. Works on macOS, Linux, and Windows.
          </motion.p>
        </div>

        {/* Tabs */}
        <motion.div
          initial={{ opacity: 0, y: 16 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          transition={{ delay: 0.3 }}
        >
          <div className="flex gap-2 mb-4 justify-center">
            {(["brew", "cargo"] as OS[]).map((key) => (
              <button
                key={key}
                onClick={() => setOs(key)}
                className={`px-5 py-2 rounded-full text-sm font-semibold transition-all ${
                  os === key
                    ? "bg-orange-500 text-black shadow-[0_0_20px_rgba(249,115,22,0.3)]"
                    : "glass text-white/40 hover:text-white/70"
                }`}
              >
                {key === "brew" ? "🍺 Homebrew" : "📦 Cargo"}
              </button>
            ))}
          </div>

          {/* Terminal block */}
          <motion.div
            key={os}
            initial={{ opacity: 0, y: 8 }}
            animate={{ opacity: 1, y: 0 }}
            className="relative group"
          >
            <div className="bg-[#0d0d0d] rounded-2xl border border-white/8 overflow-hidden">
              {/* Titlebar */}
              <div className="flex items-center gap-2 px-5 py-3.5 border-b border-white/5">
                <div className="flex gap-1.5">
                  <div className="w-3 h-3 rounded-full bg-red-500/60" />
                  <div className="w-3 h-3 rounded-full bg-yellow-500/60" />
                  <div className="w-3 h-3 rounded-full bg-green-500/60" />
                </div>
                <span className="ml-3 text-xs text-white/20 font-mono">zsh</span>
                <button
                  onClick={() => handleCopy(os)}
                  className="ml-auto flex items-center gap-1.5 text-xs text-white/30 hover:text-white/70 transition-colors px-3 py-1.5 rounded-lg glass"
                >
                  {copied === os ? (
                    <><Check size={12} className="text-green-400" /><span className="text-green-400">Copied</span></>
                  ) : (
                    <><Copy size={12} />Copy</>
                  )}
                </button>
              </div>

              {/* Commands */}
              <div className="p-6 font-mono text-sm space-y-2">
                {commands[os].lines.map((line, i) => (
                  <div key={i} className="flex items-center gap-3">
                    <span className="text-orange-500 select-none">❯</span>
                    <code className="text-white/80">{line}</code>
                  </div>
                ))}
                <div className="flex items-center gap-3 pt-2 border-t border-white/5 mt-2">
                  <span className="text-orange-500 select-none">❯</span>
                  <code className="text-white/80">cd my-repo <span className="text-white/30">&amp;&amp;</span> <span className="text-orange-400 font-bold">zit</span></code>
                </div>
              </div>
            </div>
          </motion.div>

          {/* Hint */}
          <p className="text-center text-white/25 text-xs mt-5 font-mono">{commands[os].hint}</p>
        </motion.div>
      </div>
    </section>
  );
}
