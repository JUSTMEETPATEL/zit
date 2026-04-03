import type { Metadata } from "next";
import DocsLayout from "./DocsLayout";

export const metadata: Metadata = {
  title: "Documentation — zit | AI-Powered Git Assistant",
  description:
    "Complete documentation for zit — learn how to install, configure, and use the AI-powered terminal Git assistant built in Rust.",
};

export default function DocsPage() {
  return <DocsLayout />;
}
