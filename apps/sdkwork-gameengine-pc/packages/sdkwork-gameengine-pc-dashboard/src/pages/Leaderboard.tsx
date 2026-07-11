import React, { useCallback, useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import {
  ChevronLeft,
  ChevronRight,
  ChevronsLeft,
  ChevronsRight,
  Crown,
  Loader2,
  Medal,
  TrendingUp,
  Trophy,
  User,
} from "lucide-react";
import { AnimatePresence, motion } from "motion/react";

import LeaderboardHeader from "../components/Leaderboard/LeaderboardHeader";
import { LeaderboardService, type LeaderboardRow } from "../services/leaderboard.service";

const ITEMS_PER_PAGE = 20;

export default function Leaderboard() {
  const { t } = useTranslation();
  const [currentPage, setCurrentPage] = useState(1);
  const [rankings, setRankings] = useState<LeaderboardRow[]>([]);
  const [totalRankings, setTotalRankings] = useState(0);
  const [loading, setLoading] = useState(true);
  const [loadError, setLoadError] = useState<string | null>(null);
  const [myRanking, setMyRanking] = useState<LeaderboardRow | null>(null);

  const loadRankings = useCallback(async () => {
    setLoading(true);
    setLoadError(null);
    try {
      const page = await LeaderboardService.listRankings({
        page: currentPage,
        pageSize: ITEMS_PER_PAGE,
      });
      setRankings(page.items);
      setTotalRankings(page.total);
    } catch {
      setLoadError(t('leaderboard_load_failed', 'Failed to load leaderboard.'));
      setRankings([]);
      setTotalRankings(0);
    } finally {
      setLoading(false);
    }
  }, [currentPage, t]);

  useEffect(() => {
    void loadRankings();
  }, [loadRankings]);

  useEffect(() => {
    let isMounted = true;
    void LeaderboardService.retrieveMyRanking()
      .then((item) => {
        if (isMounted) {
          setMyRanking(item);
        }
      })
      .catch(() => {
        if (isMounted) {
          setMyRanking(null);
        }
      });
    return () => {
      isMounted = false;
    };
  }, []);

  const totalPages = Math.max(1, Math.ceil(totalRankings / ITEMS_PER_PAGE));

  const handlePageChange = (page: number) => {
    if (page >= 1 && page <= totalPages) {
      setCurrentPage(page);
    }
  };

  return (
    <div className="space-y-6 pb-12 h-full flex flex-col">
      <LeaderboardHeader />

      <div className="flex items-center justify-between gap-4 rounded-2xl border border-zinc-200/50 bg-white/60 p-4 shadow-sm dark:border-zinc-800/50 dark:bg-zinc-900/50">
        <div className="flex items-center gap-3">
          <div className="flex h-10 w-10 items-center justify-center rounded-xl bg-rose-50 text-rose-600 dark:bg-rose-500/10 dark:text-rose-400">
            <Trophy size={20} />
          </div>
          <div>
            <div className="text-sm font-black text-zinc-900 dark:text-white">
              {t('global_leaderboard')}
            </div>
            <div className="text-xs font-medium text-zinc-500 dark:text-zinc-400">
              {t('showing_pagination_total', { total: totalRankings })}
            </div>
          </div>
        </div>
      </div>

      <div className="flex-1 bg-white/80 dark:bg-zinc-900/80 backdrop-blur-sm rounded-3xl border border-zinc-200 dark:border-zinc-800 overflow-hidden flex flex-col shadow-lg dark:shadow-2xl relative min-h-[500px]">
        <div className="grid grid-cols-12 gap-4 p-4 border-b border-zinc-200 dark:border-zinc-800 text-xs font-black tracking-wider text-zinc-500 uppercase bg-zinc-50/80 dark:bg-zinc-950/80 shrink-0">
          <div className="col-span-1 text-center">{t('rank')}</div>
          <div className="col-span-4">{t('player', 'Player')}</div>
          <div className="col-span-2 text-center">{t('title')}</div>
          <div className="col-span-2 text-center">{t('win_rate')}</div>
          <div className="col-span-3 text-right">{t('points')}</div>
        </div>

        <div className="flex-1 overflow-y-auto p-2 space-y-2 pb-36 scrollbar-hide">
          <AnimatePresence mode="wait">
            {loading ? (
              <div className="flex h-48 items-center justify-center text-zinc-500">
                <Loader2 className="mr-2 animate-spin" size={20} />
                <span>{t('loading', 'Loading...')}</span>
              </div>
            ) : loadError ? (
              <div className="flex h-48 items-center justify-center px-6 text-center text-sm font-medium text-rose-500">
                {loadError}
              </div>
            ) : rankings.length === 0 ? (
              <div className="flex h-48 items-center justify-center text-sm font-medium text-zinc-500">
                {t('leaderboard_empty', 'No rankings yet.')}
              </div>
            ) : (
              <motion.div
                key={currentPage}
                initial={{ opacity: 0, y: 10 }}
                animate={{ opacity: 1, y: 0 }}
                exit={{ opacity: 0, y: -10 }}
                transition={{ duration: 0.2 }}
                className="space-y-2"
              >
                {rankings.map((user) => (
                  <div
                    key={`${user.rank}-${user.userId}`}
                    className={`grid grid-cols-12 gap-4 px-4 py-3 items-center rounded-xl border transition-all hover:scale-[1.005] ${
                      user.rank === 1
                        ? "bg-gradient-to-r from-yellow-900/30 to-amber-900/10 border-yellow-500/50 shadow-[0_0_15px_rgba(234,179,8,0.1)]"
                        : user.rank === 2
                          ? "bg-gradient-to-r from-zinc-400/20 to-zinc-800/10 border-zinc-400/40"
                          : user.rank === 3
                            ? "bg-gradient-to-r from-orange-900/30 to-amber-900/10 border-orange-700/40"
                            : user.rank <= 10
                              ? "bg-zinc-100 dark:bg-zinc-900 border-zinc-200/50 dark:border-zinc-700/50 hover:bg-zinc-200"
                              : "bg-zinc-50/50 dark:bg-zinc-950/50 border-zinc-100/50 dark:border-zinc-800/50 hover:bg-zinc-100"
                    }`}
                  >
                    <div className="col-span-1 flex justify-center">
                      {user.rank === 1 ? (
                        <Crown className="text-yellow-500 drop-shadow-[0_0_8px_rgba(234,179,8,0.8)]" size={24} />
                      ) : user.rank === 2 ? (
                        <Medal className="text-zinc-400 drop-shadow-[0_0_5px_rgba(161,161,170,0.5)]" size={22} />
                      ) : user.rank === 3 ? (
                        <Medal className="text-orange-500 drop-shadow-[0_0_5px_rgba(249,115,22,0.5)]" size={22} />
                      ) : (
                        <span className={`text-lg font-black ${user.rank <= 10 ? 'text-rose-500/80' : 'text-zinc-500 dark:text-zinc-600'}`}>
                          {user.rank}
                        </span>
                      )}
                    </div>

                    <div className="col-span-4 flex items-center space-x-3">
                      <div className="relative">
                        <img
                          src={user.avatar}
                          alt={user.name}
                          className="w-10 h-10 rounded-xl object-cover border-2 border-orange-500"
                        />
                        <div className="absolute -bottom-1.5 -right-1.5 w-5 h-5 rounded-md flex items-center justify-center border-2 border-white dark:border-zinc-900 bg-orange-600">
                          <User size={10} className="text-white" />
                        </div>
                      </div>
                      <div className="truncate">
                        <span className={`font-black text-base block truncate ${
                          user.rank === 1
                            ? "text-transparent bg-clip-text bg-gradient-to-r from-yellow-500 to-amber-600 dark:from-yellow-300 dark:to-amber-500"
                            : user.rank === 2
                              ? "text-transparent bg-clip-text bg-gradient-to-r from-zinc-500 to-zinc-700 dark:from-zinc-200 dark:to-zinc-400"
                              : user.rank === 3
                                ? "text-transparent bg-clip-text bg-gradient-to-r from-orange-500 to-orange-700 dark:from-orange-300 dark:to-orange-500"
                                : user.rank <= 10
                                  ? "text-zinc-900 dark:text-white"
                                  : "text-zinc-700 dark:text-zinc-200"
                        }`}>
                          {user.name}
                        </span>
                      </div>
                    </div>

                    <div className="col-span-2 flex justify-center">
                      <span className="px-2.5 py-1 text-[10px] font-black tracking-wider rounded-md border bg-orange-50 dark:bg-orange-500/10 text-orange-600 dark:text-orange-400 border-orange-200 dark:border-orange-500/20">
                        {user.title}
                      </span>
                    </div>

                    <div className="col-span-2 text-center">
                      <div className="inline-flex items-center space-x-1 bg-zinc-50 dark:bg-zinc-950 px-2 py-1 rounded-md border border-zinc-200 dark:border-zinc-800">
                        <span className="font-mono text-emerald-600 dark:text-emerald-400 font-bold text-xs">
                          {user.winRate}
                        </span>
                      </div>
                    </div>

                    <div className="col-span-3 flex items-center justify-end">
                      <div className={`font-mono font-black text-xl ${
                        user.rank <= 3
                          ? "text-transparent bg-clip-text bg-gradient-to-br from-zinc-900 to-zinc-500 dark:from-white dark:to-zinc-400"
                          : "text-zinc-700 dark:text-zinc-300"
                      }`}>
                        {user.score.toLocaleString()}
                      </div>
                    </div>
                  </div>
                ))}
              </motion.div>
            )}
          </AnimatePresence>
        </div>

        <div className="absolute bottom-[88px] left-0 right-0 p-3 bg-white/95 dark:bg-zinc-950/95 backdrop-blur-md border-t border-zinc-200 dark:border-zinc-800 flex flex-col sm:flex-row items-center justify-between z-20 gap-2 shadow-[0_-10px_20px_rgba(0,0,0,0.05)] dark:shadow-none">
          <div className="text-sm text-zinc-500 dark:text-zinc-400 font-medium px-4">
            {t('showing_pagination', {
              start: totalRankings === 0 ? 0 : (currentPage - 1) * ITEMS_PER_PAGE + 1,
              end: Math.min(currentPage * ITEMS_PER_PAGE, totalRankings),
              total: totalRankings,
            })}
          </div>
          <div className="flex items-center space-x-2 px-4">
            <button
              onClick={() => handlePageChange(1)}
              disabled={currentPage === 1}
              className="p-2 rounded-lg bg-zinc-50 dark:bg-zinc-900 border border-zinc-200 dark:border-zinc-800 text-zinc-500 dark:text-zinc-400 hover:text-zinc-900 dark:hover:text-white hover:bg-zinc-100 dark:hover:bg-zinc-800 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
            >
              <ChevronsLeft size={16} />
            </button>
            <button
              onClick={() => handlePageChange(currentPage - 1)}
              disabled={currentPage === 1}
              className="p-2 rounded-lg bg-zinc-50 dark:bg-zinc-900 border border-zinc-200 dark:border-zinc-800 text-zinc-500 dark:text-zinc-400 hover:text-zinc-900 dark:hover:text-white hover:bg-zinc-100 dark:hover:bg-zinc-800 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
            >
              <ChevronLeft size={16} />
            </button>

            <div className="flex space-x-1 px-2">
              {Array.from({ length: Math.min(5, totalPages) }).map((_, idx) => {
                let pageNum = currentPage;
                if (currentPage <= 3) pageNum = idx + 1;
                else if (currentPage >= totalPages - 2) pageNum = totalPages - 4 + idx;
                else pageNum = currentPage - 2 + idx;

                if (pageNum < 1 || pageNum > totalPages) return null;

                return (
                  <button
                    key={pageNum}
                    onClick={() => handlePageChange(pageNum)}
                    className={`w-8 h-8 rounded-lg text-sm font-bold flex items-center justify-center transition-colors ${
                      currentPage === pageNum
                        ? "bg-rose-600 text-white shadow-[0_0_10px_rgba(225,29,72,0.4)]"
                        : "bg-zinc-50 dark:bg-zinc-900 border border-zinc-200 dark:border-zinc-800 text-zinc-600 dark:text-zinc-400 hover:bg-zinc-100 dark:hover:bg-zinc-800 hover:text-zinc-900 dark:hover:text-white"
                    }`}
                  >
                    {pageNum}
                  </button>
                );
              })}
            </div>

            <button
              onClick={() => handlePageChange(currentPage + 1)}
              disabled={currentPage === totalPages}
              className="p-2 rounded-lg bg-zinc-50 dark:bg-zinc-900 border border-zinc-200 dark:border-zinc-800 text-zinc-500 dark:text-zinc-400 hover:text-zinc-900 dark:hover:text-white hover:bg-zinc-100 dark:hover:bg-zinc-800 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
            >
              <ChevronRight size={16} />
            </button>
            <button
              onClick={() => handlePageChange(totalPages)}
              disabled={currentPage === totalPages}
              className="p-2 rounded-lg bg-zinc-50 dark:bg-zinc-900 border border-zinc-200 dark:border-zinc-800 text-zinc-500 dark:text-zinc-400 hover:text-zinc-900 dark:hover:text-white hover:bg-zinc-100 dark:hover:bg-zinc-800 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
            >
              <ChevronsRight size={16} />
            </button>
          </div>
        </div>

        <div className="absolute bottom-0 left-0 right-0 p-4 bg-white/95 dark:bg-zinc-950/95 backdrop-blur-xl border-t border-rose-500/30 shadow-[0_-10px_30px_rgba(0,0,0,0.1)] dark:shadow-[0_-20px_40px_rgba(0,0,0,0.5)] z-30">
          <div className="grid grid-cols-12 gap-4 items-center max-w-full">
            <div className="col-span-1 flex justify-center">
              <span className="text-xl font-black text-zinc-400 dark:text-zinc-500">
                {myRanking?.rank ?? "?"}
              </span>
            </div>
            <div className="col-span-4 flex items-center space-x-4">
              <div className="relative">
                <img
                  src={
                    myRanking?.avatar
                    ?? `https://api.dicebear.com/7.x/initials/svg?seed=${encodeURIComponent(myRanking?.name ?? 'guest')}`
                  }
                  alt={t('me')}
                  className="w-12 h-12 rounded-xl object-cover border-2 border-rose-500/50"
                />
                <div className="absolute -bottom-2 -right-2 w-6 h-6 rounded-lg flex items-center justify-center border-2 border-white dark:border-zinc-900 bg-orange-600">
                  <User size={12} className="text-white" />
                </div>
              </div>
              <div>
                <span className="font-black text-lg text-zinc-900 dark:text-white block">
                  {t('me')} ({myRanking?.name ?? t('guest', 'Guest')})
                </span>
                <span className="text-xs text-rose-500 dark:text-rose-400 font-medium">
                  {myRanking
                    ? t('current_score', { score: myRanking.score.toLocaleString() })
                    : t('not_ranked_yet', 'Not ranked yet')}
                </span>
              </div>
            </div>
            <div className="col-span-2 flex justify-center">
              <span className="px-3 py-1 text-xs font-black tracking-wider rounded-lg border bg-zinc-100 dark:bg-zinc-800/50 text-zinc-600 dark:text-zinc-400 border-zinc-200 dark:border-zinc-700">
                {myRanking?.title ?? t('player', 'Player')}
              </span>
            </div>
            <div className="col-span-2 text-center">
              <div className="inline-flex items-center space-x-1 bg-zinc-50 dark:bg-zinc-900 px-3 py-1 rounded-lg border border-zinc-200 dark:border-zinc-800">
                <span className="font-mono text-emerald-600 dark:text-emerald-400 font-bold">
                  {myRanking?.winRate ?? "-"}
                </span>
              </div>
            </div>
            <div className="col-span-3 flex items-center justify-end space-x-4 pr-4">
              <div className="flex items-center space-x-1 text-xs font-bold text-emerald-600 dark:text-emerald-500">
                <TrendingUp size={14} />
                <span>{myRanking?.trend ?? "0"}</span>
              </div>
              <div className="font-mono font-black text-2xl text-transparent bg-clip-text bg-gradient-to-br from-zinc-900 to-zinc-500 dark:from-white dark:to-zinc-400">
                {(myRanking?.score ?? 0).toLocaleString()}
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
