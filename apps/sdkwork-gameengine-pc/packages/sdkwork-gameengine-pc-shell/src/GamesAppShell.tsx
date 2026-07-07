import React, { useState } from "react";
import { motion, AnimatePresence } from "motion/react";
import { Sidebar, Topbar } from "sdkwork-gameengine-pc-commons";
import { GameCenter, Leaderboard } from "sdkwork-gameengine-pc-dashboard";

export interface GamesAppShellProps {
  onLogout?: () => Promise<void> | void;
}

export default function GamesAppShell({ onLogout }: GamesAppShellProps) {
  const [currentView, setCurrentView] = useState("games");

  const renderView = () => {
    switch (currentView) {
      case "leaderboard":
        return <Leaderboard />;
      case "games":
        return <GameCenter setCurrentView={setCurrentView} />;
      default:
        return <GameCenter setCurrentView={setCurrentView} />;
    }
  };

  return (
    <div className="flex h-screen bg-zinc-50 dark:bg-zinc-950 text-zinc-900 dark:text-zinc-50 overflow-hidden font-sans selection:bg-rose-500/30 transition-colors duration-300">
      <Sidebar currentView={currentView} setCurrentView={setCurrentView} />
      <div className="flex-1 flex flex-col relative">
        <Topbar onLogout={onLogout} setCurrentView={setCurrentView} />
        <main className="flex-1 overflow-y-auto p-6 relative z-0">
          <AnimatePresence mode="wait">
            <motion.div
              key={currentView}
              initial={{ opacity: 0, y: 10 }}
              animate={{ opacity: 1, y: 0 }}
              exit={{ opacity: 0, y: -10 }}
              transition={{ duration: 0.2 }}
              className="h-full"
            >
              {renderView()}
            </motion.div>
          </AnimatePresence>
        </main>
      </div>
    </div>
  );
}
