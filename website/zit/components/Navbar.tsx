"use client";

import { motion, AnimatePresence } from "framer-motion";
import { Terminal, Github, Menu, X } from "lucide-react";
import Link from "next/link";
import { useState, useEffect } from "react";

const navLinks = [
  { name: "Features", href: "#features" },
  { name: "AI Mentor", href: "#ai-mentor" },
  { name: "Install", href: "#installation" },
  { name: "Keybindings", href: "#keybindings" },
];

export default function Navbar() {
  const [isOpen, setIsOpen] = useState(false);
  const [scrolled, setScrolled] = useState(false);

  useEffect(() => {
    const fn = () => setScrolled(window.scrollY > 24);
    window.addEventListener("scroll", fn, { passive: true });
    return () => window.removeEventListener("scroll", fn);
  }, []);

  return (
    <nav
      className={`fixed top-0 left-0 right-0 z-50 transition-all duration-300 ${
        scrolled ? "bg-[#080808]/80 backdrop-blur-xl border-b border-white/5" : ""
      }`}
    >
      <div className="max-w-6xl mx-auto px-6 h-16 flex items-center justify-between">
        {/* Logo */}
        <Link href="/" className="flex items-center gap-2.5" onClick={() => setIsOpen(false)}>
          <span className="bg-orange-500/10 border border-orange-500/20 text-orange-400 p-1.5 rounded-lg">
            <Terminal size={16} />
          </span>
          <span className="font-black text-white text-lg tracking-tight">zit</span>
        </Link>

        {/* Desktop */}
        <div className="hidden md:flex items-center gap-1">
          {navLinks.map((l) => (
            <Link
              key={l.name}
              href={l.href}
              className="px-3.5 py-2 text-sm text-white/40 hover:text-white transition-colors rounded-lg hover:bg-white/5"
            >
              {l.name}
            </Link>
          ))}
        </div>

        <div className="hidden md:flex items-center gap-3">
          <Link
            href="https://github.com/JUSTMEETPATEL/zit"
            target="_blank"
            className="flex items-center gap-2 text-white/40 hover:text-white text-sm transition-colors"
          >
            <Github size={17} />
          </Link>
          <Link
            href="#installation"
            className="px-5 py-2 rounded-lg bg-orange-500 hover:bg-orange-400 text-black font-bold text-sm transition-all shadow-[0_0_20px_rgba(249,115,22,0.25)] hover:shadow-[0_0_30px_rgba(249,115,22,0.4)]"
          >
            Install
          </Link>
        </div>

        {/* Mobile toggle */}
        <button
          className="md:hidden text-white/40 hover:text-white transition-colors p-1"
          onClick={() => setIsOpen(!isOpen)}
        >
          {isOpen ? <X size={22} /> : <Menu size={22} />}
        </button>
      </div>

      {/* Mobile menu */}
      <AnimatePresence>
        {isOpen && (
          <motion.div
            initial={{ opacity: 0, height: 0 }}
            animate={{ opacity: 1, height: "auto" }}
            exit={{ opacity: 0, height: 0 }}
            className="md:hidden bg-[#080808]/95 backdrop-blur-xl border-b border-white/5 overflow-hidden"
          >
            <div className="px-6 py-4 space-y-1">
              {navLinks.map((l) => (
                <Link
                  key={l.name}
                  href={l.href}
                  onClick={() => setIsOpen(false)}
                  className="block px-3 py-2.5 text-sm text-white/50 hover:text-white hover:bg-white/5 rounded-lg transition-colors"
                >
                  {l.name}
                </Link>
              ))}
              <div className="pt-3 border-t border-white/5 flex flex-col gap-2 mt-2">
                <Link
                  href="https://github.com/JUSTMEETPATEL/zit"
                  target="_blank"
                  onClick={() => setIsOpen(false)}
                  className="flex items-center gap-2 px-3 py-2.5 text-sm text-white/40 hover:text-white transition-colors"
                >
                  <Github size={16} /> GitHub
                </Link>
                <Link
                  href="#installation"
                  onClick={() => setIsOpen(false)}
                  className="px-3 py-3 rounded-xl bg-orange-500 text-black font-bold text-sm text-center"
                >
                  Install zit
                </Link>
              </div>
            </div>
          </motion.div>
        )}
      </AnimatePresence>
    </nav>
  );
}
