"use client";

import { AlertTriangle, AlertCircle, Terminal, Github, ArrowRight } from "lucide-react";
import Link from "next/link";

const issues = [
  {
    title: "Windows: linker link.exe not found",
    steps: [
      'Install Visual Studio Build Tools from visualstudio.microsoft.com/visual-cpp-build-tools',
      'Select the "Desktop development with C++" workload and install.',
      'Restart your terminal and try again.',
    ],
    icon: <AlertTriangle size={20} className="text-amber-500" />,
    borderColor: "border-amber-500/20",
    bgColor: "bg-amber-500/5",
  },
  {
    title: "AI Mentor not responding",
    steps: [
      'Use Health Check in the AI Mentor panel: press `a` → select "Health Check".',
      'Verify config: cat ~/.config/zit/config.toml — ensure [ai] section is present with endpoint + api_key.',
      'Check env vars: echo $ZIT_AI_ENDPOINT $ZIT_AI_API_KEY',
      'Check Lambda logs: aws logs tail /aws/lambda/zit-ai-mentor-dev --region ap-south-1',
    ],
    icon: <AlertCircle size={20} className="text-red-500" />,
    borderColor: "border-red-500/20",
    bgColor: "bg-red-500/5",
  },
];

export default function Troubleshooting() {
  return (
    <>
      <section id="troubleshooting" className="py-20 bg-zinc-900/20 border-t border-white/5">
        <div className="max-w-4xl mx-auto px-4 sm:px-6 lg:px-8">
          <h2 className="text-3xl font-extrabold mb-2 text-center text-white">Troubleshooting</h2>
          <p className="text-center text-gray-500 mb-10">Common issues and how to resolve them.</p>

          <div className="space-y-4">
            {issues.map((issue, i) => (
              <div
                key={i}
                className={`rounded-2xl border ${issue.borderColor} ${issue.bgColor} p-6`}
              >
                <div className="flex items-center gap-3 mb-4">
                  {issue.icon}
                  <h4 className="font-bold text-white">{issue.title}</h4>
                </div>
                <ol className="space-y-2">
                  {issue.steps.map((step, j) => (
                    <li key={j} className="flex items-start gap-3 text-sm text-gray-400">
                      <span className="min-w-[20px] h-5 rounded-full bg-white/5 border border-white/10 text-[10px] text-gray-500 flex items-center justify-center font-bold shrink-0 mt-0.5">
                        {j + 1}
                      </span>
                      <code className="text-gray-300 font-mono text-xs leading-relaxed">{step}</code>
                    </li>
                  ))}
                </ol>
              </div>
            ))}
          </div>
        </div>
      </section>

      {/* CTA Section */}
      <section className="py-24 bg-black relative overflow-hidden">
        <div className="absolute inset-0 bg-[radial-gradient(ellipse_60%_70%_at_50%_50%,rgba(249,115,22,0.1),transparent)] pointer-events-none"></div>
        <div className="absolute top-0 left-0 right-0 h-px bg-gradient-to-r from-transparent via-orange-500/30 to-transparent"></div>

        <div className="max-w-4xl mx-auto px-4 sm:px-6 lg:px-8 text-center relative z-10">
          <div className="inline-flex items-center gap-2 px-3 py-1.5 mb-6 rounded-full bg-orange-500/10 border border-orange-500/20 text-orange-400 text-xs font-bold tracking-wide">
            <Terminal size={12} />
            Ready to level up your Git workflow?
          </div>

          <h2 className="text-5xl md:text-6xl font-extrabold tracking-tight mb-6">
            Start using{" "}
            <span className="text-transparent bg-clip-text bg-gradient-to-r from-orange-400 to-amber-500">
              zit
            </span>{" "}
            today.
          </h2>

          <p className="text-gray-400 text-xl mb-10 max-w-2xl mx-auto leading-relaxed">
            14 Git features, AI mentorship, and a beautiful TUI — all in one Rust binary. Free & open source.
          </p>

          <div className="flex flex-col sm:flex-row gap-4 justify-center">
            <Link
              href="#installation"
              className="group inline-flex items-center justify-center gap-2 px-8 py-4 rounded-xl bg-gradient-to-r from-orange-500 to-orange-600 text-white font-bold text-base shadow-[0_0_30px_rgba(249,115,22,0.4)] hover:shadow-[0_0_50px_rgba(249,115,22,0.6)] transition-all hover:-translate-y-1"
            >
              Install Now
              <ArrowRight size={18} className="group-hover:translate-x-1 transition-transform" />
            </Link>
            <Link
              href="https://github.com/JUSTMEETPATEL/zit"
              target="_blank"
              className="inline-flex items-center justify-center gap-2 px-8 py-4 rounded-xl bg-white/5 hover:bg-white/10 text-white font-semibold text-base border border-white/10 hover:border-white/20 transition-all hover:-translate-y-1"
            >
              <Github size={18} />
              View on GitHub
            </Link>
          </div>

          <div className="mt-8 font-mono text-sm text-gray-600">
            brew tap JUSTMEETPATEL/zit && brew install zit
          </div>
        </div>
      </section>
    </>
  );
}
