import Navbar from "@/components/Navbar";
import Hero from "@/components/Hero";
import BentoGrid from "@/components/BentoGrid";
import AiFeatures from "@/components/AiFeatures";
import Installation from "@/components/Installation";
import Keybindings from "@/components/Keybindings";
import Configuration from "@/components/Configuration";
import Development from "@/components/Development";
import Architecture from "@/components/Architecture";
import Troubleshooting from "@/components/Troubleshooting";
import Footer from "@/components/Footer";

export default function Home() {
  return (
    <main className="min-h-screen relative bg-[var(--background)] text-[var(--foreground)] overflow-x-hidden selection:bg-[var(--primary)] selection:text-white">
      <Navbar />
      <Hero />
      <BentoGrid />
      <AiFeatures />
      <Installation />
      <Keybindings />
      <Configuration />
      <Architecture />
      <Development />
      <Troubleshooting />
      <Footer />
    </main>
  );
}
