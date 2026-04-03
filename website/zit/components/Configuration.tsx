"use client";

import { motion } from "framer-motion";
import { Settings, Lock, Key, FileCog } from "lucide-react";
import { useState } from "react";

type Tab = "general" | "ai" | "env";

const tabs: { id: Tab; label: string; color: string; activeClass: string }[] = [
  { id: "general", label: "General", color: "orange", activeClass: "bg-orange-500/10 text-orange-400 border-b-2 border-orange-500" },
  { id: "ai", label: "AI Config", color: "violet", activeClass: "bg-violet-500/10 text-violet-400 border-b-2 border-violet-500" },
  { id: "env", label: "Env Vars", color: "blue", activeClass: "bg-blue-500/10 text-blue-400 border-b-2 border-blue-500" },
];

const generalConfig = `[general]
tick_rate_ms = 2000          # UI refresh interval
confirm_destructive = true   # Confirmation for risky ops

[ui]
color_scheme = "default"
show_help_hints = true

[github]
# pat = "ghp_..."            # Or use OAuth device flow`;

const aiConfig = `[ai]
enabled = true
endpoint = "https://your-api.execute-api.region.amazonaws.com/dev/mentor"
api_key = "your-api-key"
timeout_secs = 30

# Tip: zit migrates keys to OS keychain on first run.
# Plaintext values are removed from this file after migration.`;

const envContent = `# Option B — Environment Variables
export ZIT_AI_ENDPOINT="https://your-api.execute-api.amazonaws.com/dev/mentor"
export ZIT_AI_API_KEY="your-api-key"

# Config file takes precedence over env vars.`;

const tabContent: Record<Tab, string> = {
  general: generalConfig,
  ai: aiConfig,
  env: envContent,
};

export default function Configuration() {
  const [activeTab, setActiveTab] = useState<Tab>("general");

  return (
    <section id="configuration" className="py-24 relative bg-zinc-900/20 border-y border-white/5">
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
        <div className="grid md:grid-cols-5 gap-12 items-start">
          {/* Left */}
          <div className="md:col-span-2">
            <div className="flex items-center gap-3 mb-4">
              <div className="p-2.5 bg-violet-500/10 rounded-xl text-violet-400 ring-1 ring-violet-500/20">
                <Settings size={22} />
              </div>
              <h2 className="text-3xl font-extrabold text-white">Configuration</h2>
            </div>
            <p className="text-gray-400 mb-8 leading-relaxed">
              All settings live in{" "}
              <code className="text-orange-400 bg-orange-500/10 px-1.5 py-0.5 rounded text-sm">
                ~/.config/zit/config.toml
              </code>
              . Zero mandatory configuration — sane defaults work out of the box.
            </p>

            <div className="space-y-5">
              <div className="flex items-start gap-3 p-4 rounded-xl bg-white/3 border border-white/5">
                <FileCog size={20} className="text-orange-400 shrink-0 mt-0.5" />
                <div>
                  <h4 className="text-white font-semibold text-sm mb-0.5">TOML Format</h4>
                  <p className="text-gray-500 text-xs leading-relaxed">Simple, human-readable configuration. Sections for general, UI, GitHub, and AI.</p>
                </div>
              </div>
              <div className="flex items-start gap-3 p-4 rounded-xl bg-white/3 border border-white/5">
                <Key size={20} className="text-violet-400 shrink-0 mt-0.5" />
                <div>
                  <h4 className="text-white font-semibold text-sm mb-0.5">AI Backend Setup</h4>
                  <p className="text-gray-500 text-xs leading-relaxed">AI features require deploying the AWS Lambda backend. See <code className="text-gray-300">aws/README.md</code>.</p>
                </div>
              </div>
              <div className="flex items-start gap-3 p-4 rounded-xl bg-white/3 border border-white/5">
                <Lock size={20} className="text-blue-400 shrink-0 mt-0.5" />
                <div>
                  <h4 className="text-white font-semibold text-sm mb-0.5">OS Keychain Security</h4>
                  <p className="text-gray-500 text-xs leading-relaxed">GitHub tokens & AI keys are automatically migrated to macOS Keychain / Windows Credential Manager on first run.</p>
                </div>
              </div>
            </div>
          </div>

          {/* Right */}
          <div className="md:col-span-3">
            <div className="bg-[#0c0c0c] border border-white/10 rounded-2xl overflow-hidden shadow-2xl">
              {/* Titlebar */}
              <div className="bg-zinc-900 px-4 py-2.5 flex items-center gap-2 border-b border-white/5">
                <div className="flex gap-1.5">
                  <div className="w-3 h-3 rounded-full bg-red-500/70"></div>
                  <div className="w-3 h-3 rounded-full bg-yellow-500/70"></div>
                  <div className="w-3 h-3 rounded-full bg-green-500/70"></div>
                </div>
                <span className="ml-3 text-xs text-gray-500 font-mono">~/.config/zit/config.toml</span>
              </div>

              {/* Tabs */}
              <div className="flex border-b border-white/5 overflow-x-auto">
                {tabs.map((tab) => (
                  <button
                    key={tab.id}
                    onClick={() => setActiveTab(tab.id)}
                    className={`px-5 py-3 text-sm font-medium transition-all whitespace-nowrap ${
                      activeTab === tab.id
                        ? tab.activeClass
                        : "text-gray-500 hover:text-white hover:bg-white/5"
                    }`}
                  >
                    {tab.label}
                  </button>
                ))}
              </div>

              {/* Code */}
              <div className="p-6">
                <motion.pre
                  key={activeTab}
                  initial={{ opacity: 0, y: 5 }}
                  animate={{ opacity: 1, y: 0 }}
                  transition={{ duration: 0.2 }}
                  className="font-mono text-sm text-gray-300 overflow-x-auto leading-relaxed"
                >
                  <code>{tabContent[activeTab]}</code>
                </motion.pre>
              </div>
            </div>

            <p className="mt-4 text-xs text-gray-600 text-center">
              AI is fully optional — all core features work without any AI configuration.
            </p>
          </div>
        </div>
      </div>
    </section>
  );
}
