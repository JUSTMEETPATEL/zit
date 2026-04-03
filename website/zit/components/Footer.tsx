"use client";

import { Terminal, Github, Star } from "lucide-react";
import Link from "next/link";

export default function Footer() {
  return (
    <footer className="border-t border-white/5 bg-[#060606]">
      <div className="max-w-6xl mx-auto px-6 py-12 flex flex-col md:flex-row items-center justify-between gap-6">
        {/* Brand */}
        <div className="flex items-center gap-3">
          <span className="bg-orange-500/10 border border-orange-500/20 text-orange-400 p-1.5 rounded-lg">
            <Terminal size={16} />
          </span>
          <span className="font-black text-white text-lg tracking-tight">zit</span>
          <span className="text-white/20 text-sm">· AI-Powered Git Assistant</span>
        </div>

        {/* Center */}
        <p className="text-white/20 text-xs font-mono text-center">
          Built with Rust & ❤️ ·{" "}
          <a
            href="https://github.com/JUSTMEETPATEL/zit/blob/main/LICENSE"
            target="_blank"
            className="hover:text-white/50 transition-colors"
          >
            MIT License
          </a>
        </p>

        {/* Links */}
        <div className="flex items-center gap-4">
          <Link
            href="https://github.com/JUSTMEETPATEL/zit"
            target="_blank"
            className="flex items-center gap-1.5 text-white/30 hover:text-white transition-colors text-sm"
          >
            <Github size={16} />
            GitHub
          </Link>
          <Link
            href="https://github.com/JUSTMEETPATEL/zit"
            target="_blank"
            className="flex items-center gap-1.5 text-white/30 hover:text-yellow-400 transition-colors text-sm"
          >
            <Star size={16} />
            Star
          </Link>
        </div>
      </div>
    </footer>
  );
}
