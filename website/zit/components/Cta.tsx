"use client";

import { motion } from "framer-motion";
import { ArrowRight, Github } from "lucide-react";
import Link from "next/link";

export default function Cta() {
  return (
    <section className="py-32 relative overflow-hidden">
      {/* Big center glow */}
      <div className="absolute inset-0 bg-[radial-gradient(ellipse_70%_60%_at_50%_50%,rgba(249,115,22,0.09),transparent)] pointer-events-none" />
      <div className="absolute top-0 left-0 right-0 h-px bg-gradient-to-r from-transparent via-orange-500/20 to-transparent" />

      <div className="relative z-10 max-w-4xl mx-auto px-6 text-center">
        <motion.div
          initial={{ opacity: 0, y: 24 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          transition={{ duration: 0.7 }}
        >
          <div className="inline-flex items-center gap-2 mb-8 px-4 py-1.5 rounded-full border border-orange-500/20 bg-orange-500/5 text-orange-400 text-xs font-semibold tracking-wide">
            Free & Open Source · MIT License
          </div>

          <h2 className="text-5xl md:text-6xl lg:text-7xl font-black tracking-tight leading-[1.02] mb-6">
            Ready to master
            <br />
            <span className="text-orange">Git?</span>
          </h2>

          <p className="text-white/40 text-lg md:text-xl max-w-xl mx-auto mb-12 leading-relaxed">
            14 features. AI mentorship. Beautiful TUI.
            <br />
            One Rust binary.
          </p>

          <div className="flex flex-col sm:flex-row items-center justify-center gap-4">
            <Link
              href="#installation"
              className="group flex items-center gap-2.5 px-9 py-4 rounded-xl bg-orange-500 hover:bg-orange-400 text-black font-bold text-base transition-all shadow-[0_0_40px_rgba(249,115,22,0.35)] hover:shadow-[0_0_60px_rgba(249,115,22,0.55)] hover:-translate-y-0.5"
            >
              Install Now
              <ArrowRight size={18} className="group-hover:translate-x-1 transition-transform" />
            </Link>
            <Link
              href="https://github.com/JUSTMEETPATEL/zit"
              target="_blank"
              className="flex items-center gap-2.5 px-9 py-4 rounded-xl glass border border-white/8 text-white/60 hover:text-white font-semibold text-base transition-all hover:-translate-y-0.5"
            >
              <Github size={18} />
              Star on GitHub
            </Link>
          </div>

          <div className="mt-10 font-mono text-xs text-white/15">
            brew tap JUSTMEETPATEL/zit && brew install zit
          </div>
        </motion.div>
      </div>
    </section>
  );
}
