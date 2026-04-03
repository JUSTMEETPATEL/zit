import Navbar from "@/components/Navbar";
import Hero from "@/components/Hero";
import Features from "@/components/Features";
import AiFeatures from "@/components/AiFeatures";
import Installation from "@/components/Installation";
import Keybindings from "@/components/Keybindings";
import Cta from "@/components/Cta";
import Footer from "@/components/Footer";

export default function Home() {
  return (
    <main className="min-h-screen relative bg-[var(--background)] text-[var(--foreground)] overflow-x-hidden selection:bg-[var(--primary)]/30 selection:text-white">
      <Navbar />
      <Hero />
      <Features />
      <AiFeatures />
      <Installation />
      <Keybindings />
      <Cta />
      <Footer />
    </main>
  );
}
