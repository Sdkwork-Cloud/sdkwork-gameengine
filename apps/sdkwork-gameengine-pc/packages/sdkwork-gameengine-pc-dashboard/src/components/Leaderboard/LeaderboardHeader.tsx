import React from "react";
import { useTranslation } from "react-i18next";
import { Flame } from "lucide-react";

export default function LeaderboardHeader() {
  const { t } = useTranslation();

  return (
    <div className="flex flex-col md:flex-row md:items-end justify-between gap-4 bg-white/80 dark:bg-zinc-900/50 p-6 rounded-3xl border border-zinc-200 dark:border-zinc-800 relative overflow-hidden shrink-0 shadow-sm dark:shadow-none">
      <div className="absolute top-0 right-0 w-64 h-64 bg-rose-600/10 rounded-full blur-3xl pointer-events-none"></div>
      <div className="relative z-10">
        <div className="flex items-center space-x-3 mb-2">
          <Flame className="text-rose-500" size={28} />
          <h1 className="text-4xl font-black text-transparent bg-clip-text bg-gradient-to-r from-zinc-900 to-zinc-600 dark:from-white dark:to-zinc-400 tracking-tight">
            {t('leaderboard_title')}
          </h1>
        </div>
        <p className="text-zinc-600 dark:text-zinc-400 font-medium font-sans">
          {t('leaderboard_desc')}
        </p>
      </div>
    </div>
  );
}
