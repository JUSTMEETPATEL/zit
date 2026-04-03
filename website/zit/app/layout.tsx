import type { Metadata } from "next";
import { Inter, JetBrains_Mono } from "next/font/google";
import "./globals.css";

const inter = Inter({
  variable: "--font-inter",
  subsets: ["latin"],
  display: "swap",
});

const jetbrainsMono = JetBrains_Mono({
  variable: "--font-mono",
  subsets: ["latin"],
  display: "swap",
});

export const metadata: Metadata = {
  title: "zit — AI-Powered Git Assistant for the Terminal",
  description:
    "zit is a Rust-powered TUI that replaces your entire Git workflow — interactive staging, visual branching, AI commit messages, conflict resolution, and an AI mentor built in. 14+ features, zero GUI bloat.",
  openGraph: {
    title: "zit — AI-Powered Git Assistant for the Terminal",
    description:
      "Terminal-based Git assistant with 14+ features: interactive staging, guided commits, visual branching, AI mentor, GitHub integration, and more. Built in Rust.",
    url: "https://zitcli.com",
    siteName: "zit",
    type: "website",
    locale: "en_US",
  },
  twitter: {
    card: "summary_large_image",
    title: "zit — AI-Powered Git Assistant for the Terminal",
    description:
      "14+ Git features. AI mentorship. Beautiful TUI. Built in Rust. Free & open source.",
  },
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en" className={`${inter.variable} ${jetbrainsMono.variable}`}>
      <body className="antialiased">{children}</body>
    </html>
  );
}
