"use client";

import { motion } from "framer-motion";

const layers = [
  {
    title: "AI Guidance Layer",
    subtitle: "Amazon Bedrock — Claude 3 Sonnet",
    desc: "Mentorship, auto error explanation, AI commit messages",
    color: "border-violet-500/30 bg-violet-500/5 shadow-[0_0_30px_rgba(139,92,246,0.08)]",
    badge: "text-violet-400",
    dot: "bg-violet-500",
    items: ["Explain Repo", "Ask a Question", "Safety Recommendations", "Health Check"],
  },
  {
    title: "Zit TUI Application",
    subtitle: "Built with Rust + ratatui + crossterm",
    desc: "All 14 Git features, state management, event loop, safety guardrails",
    color: "border-orange-500/40 bg-orange-500/5 shadow-[0_0_40px_rgba(249,115,22,0.12)] scale-[1.03]",
    badge: "text-orange-400",
    dot: "bg-orange-500",
    items: ["Staging", "Commits", "Branches", "Timeline", "Time Travel", "Merge Resolve", "Bisect"],
    youAreHere: true,
  },
  {
    title: "Native Git CLI",
    subtitle: "Your system's git binary",
    desc: "100% compatible — no reimplemented internals, runs real git commands",
    color: "border-white/10 bg-white/2 opacity-75",
    badge: "text-gray-400",
    dot: "bg-gray-500",
    items: ["git status", "git add", "git commit", "git push", "git log"],
  },
];

export default function Architecture() {
  return (
    <section id="architecture" className="py-24 relative overflow-hidden">
      <div className="absolute top-0 left-0 w-full h-full bg-[radial-gradient(ellipse_50%_40%_at_50%_50%,rgba(249,115,22,0.06),transparent)] pointer-events-none"></div>

      <div className="max-w-5xl mx-auto px-4 sm:px-6 lg:px-8 relative z-10">
        <div className="text-center mb-16">
          <h2 className="text-4xl md:text-5xl font-extrabold mb-4 tracking-tight">
            How{" "}
            <span className="text-transparent bg-clip-text bg-gradient-to-r from-orange-400 to-amber-500">
              zit works
            </span>
          </h2>
          <p className="text-gray-400 text-lg max-w-2xl mx-auto">
            Three clean layers — AI on top, the Rust TUI in the middle, native Git at the bottom. No magic, no bloat.
          </p>
        </div>

        <div className="space-y-4 relative">
          {/* Connecting line */}
          <div className="absolute left-8 top-12 bottom-12 w-px bg-gradient-to-b from-violet-500/30 via-orange-500/50 to-white/10 hidden md:block"></div>

          {layers.map((layer, i) => (
            <motion.div
              key={layer.title}
              initial={{ opacity: 0, x: -20 }}
              whileInView={{ opacity: 1, x: 0 }}
              viewport={{ once: true }}
              transition={{ delay: i * 0.15 }}
              className={`relative rounded-2xl border p-6 md:p-8 backdrop-blur-sm transition-all ${layer.color}`}
            >
              {layer.youAreHere && (
                <div className="absolute top-4 right-4 flex items-center gap-1.5 bg-orange-500 text-black text-[10px] font-black rounded-full px-2.5 py-1 tracking-wider">
                  <span className="w-1.5 h-1.5 rounded-full bg-black animate-pulse"></span>
                  YOU ARE HERE
                </div>
              )}
              <div className="flex items-start gap-4 md:gap-8">
                {/* Layer indicator */}
                <div className="hidden md:flex flex-col items-center gap-2 shrink-0 w-6 mt-1">
                  <div className={`w-3 h-3 rounded-full ${layer.dot} shrink-0 shadow-[0_0_10px_currentColor]`}></div>
                </div>

                <div className="flex-1 min-w-0">
                  <div className="flex flex-col md:flex-row md:items-center gap-2 mb-1">
                    <h3 className={`text-xl font-bold text-white`}>{layer.title}</h3>
                    <span className={`text-xs font-mono ${layer.badge} bg-white/5 px-2 py-0.5 rounded-full border border-white/10 self-start`}>
                      {layer.subtitle}
                    </span>
                  </div>
                  <p className="text-gray-400 text-sm mb-4">{layer.desc}</p>
                  <div className="flex flex-wrap gap-2">
                    {layer.items.map((item) => (
                      <span
                        key={item}
                        className="text-xs font-mono text-gray-400 bg-black/30 border border-white/5 px-2.5 py-1 rounded-lg"
                      >
                        {item}
                      </span>
                    ))}
                  </div>
                </div>
              </div>
            </motion.div>
          ))}
        </div>

        {/* Key design decisions */}
        <div className="mt-14 grid grid-cols-1 md:grid-cols-2 gap-4">
          {[
            { title: "Shell-based Git", desc: "Runs real `git` — never reimplements git internals for 100% compatibility." },
            { title: "AI is Optional", desc: "Degrades gracefully to static help when AI is unconfigured or unavailable." },
            { title: "Non-blocking AI", desc: "All AI calls run in background threads to keep the TUI perfectly responsive." },
            { title: "Retry with Backoff", desc: "AI client retries transient failures — 2 retries with exponential backoff." },
          ].map((d) => (
            <div key={d.title} className="bg-white/3 border border-white/5 rounded-xl p-5">
              <h4 className="text-white font-bold mb-1 text-sm">⚡ {d.title}</h4>
              <p className="text-gray-500 text-xs leading-relaxed">{d.desc}</p>
            </div>
          ))}
        </div>
      </div>
    </section>
  );
}
