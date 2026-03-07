import type { Metadata } from "next";
import { Geist, Geist_Mono } from "next/font/google";
import "./globals.css";

const geistSans = Geist({
  variable: "--font-geist-sans",
  subsets: ["latin"],
});

const geistMono = Geist_Mono({
  variable: "--font-geist-mono",
  subsets: ["latin"],
});

export const metadata: Metadata = {
  title: "zit - Your AI-Powered Git Assistant",
  description: "zit is a terminal-based Git assistant that makes version control simple and intuitive. With interactive staging, guided commits, visual history, and an AI mentor that teaches you while you work, zit is the ultimate tool for mastering Git.",
  openGraph: {
    title: "zit - Your AI-Powered Git Assistant",
    description:
      "Terminal-based Git assistant with interactive staging, guided commits, visual history, and AI mentor.",
    url: "https://zitcli.com",
    siteName: "zit",
    type: "website",
    locale: "en_US",
  },
  twitter: {
    card: "summary_large_image",
    title: "zit - Your AI-Powered Git Assistant",
    description:
      "Terminal-based Git assistant with interactive staging, guided commits, visual history, and AI mentor.",
  },
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en">
      <body
        className={`${geistSans.variable} ${geistMono.variable} antialiased`}
      >
        {children}
      </body>
    </html>
  );
}
