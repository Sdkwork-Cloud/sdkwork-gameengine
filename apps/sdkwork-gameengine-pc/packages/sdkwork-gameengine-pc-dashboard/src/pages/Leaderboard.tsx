import React, { useState, useEffect, useCallback } from "react";
import { useTranslation } from "react-i18next";
import {
  Trophy,
  Medal,
  Crown,
  User,
  Cpu,
  Users,
  Flame,
  TrendingUp,
  Timer,
  ChevronLeft,
  ChevronRight,
  ChevronsLeft,
  ChevronsRight,
  Swords,
  Loader2,
} from "lucide-react";
import { motion, AnimatePresence } from "motion/react";
import { useToast } from "sdkwork-gameengine-pc-commons";
import { useUserStore } from "sdkwork-gameengine-pc-core";

import { LeaderboardService, type LeaderboardRow } from "../services/leaderboard.service";

// Sub-components
import LeaderboardHeader from "../components/Leaderboard/LeaderboardHeader";
import ChallengeModal from "../components/Leaderboard/ChallengeModal";
import ArenaModal from "../components/Leaderboard/ArenaModal";

interface LeaderboardProps {
  onViewPlayer?: (player: any) => void;
}

export default function Leaderboard({ onViewPlayer }: LeaderboardProps) {
  const { t } = useTranslation();
  const { showToast, ToastComponent } = useToast();
  const [activeTab, setActiveTab] = useState("global");
  const [timeRange, setTimeRange] = useState("daily");
  const [currentPage, setCurrentPage] = useState(1);
  const [showChallengeModal, setShowChallengeModal] = useState(false);
  const [selectedPlayer, setSelectedPlayer] = useState<any>(null);
  const [showArenaModal, setShowArenaModal] = useState(false);
  const [wagerAmount, setWagerAmount] = useState(100);
  const itemsPerPage = 20;
  const profile = useUserStore((state) => state.profile);
  const [rankings, setRankings] = useState<LeaderboardRow[]>([]);
  const [totalRankings, setTotalRankings] = useState(0);
  const [loading, setLoading] = useState(true);
  const [myRanking, setMyRanking] = useState<LeaderboardRow | null>(null);

  const tabs = [
    { id: "global", name: t('global_leaderboard'), icon: <Trophy size={16} /> },
    { id: "human", name: t('human_peak'), icon: <User size={16} /> },
    { id: "ai", name: t('strongest_ai'), icon: <Cpu size={16} /> },
    { id: "team", name: t('team_rankings'), icon: <Users size={16} /> },
  ];

  const timeRanges = [
    { id: "hourly", name: t('hourly') },
    { id: "daily", name: t('daily') },
    { id: "weekly", name: t('weekly') },
    { id: "monthly", name: t('monthly') },
    { id: "season", name: t('season') },
    { id: "yearly", name: t('yearly') },
    { id: "all-time", name: t('all_time') },
  ];

  // Reset page when time range or tab changes
  useEffect(() => {
    setCurrentPage(1);
  }, [timeRange, activeTab]);

  const loadRankings = useCallback(async () => {
    if (activeTab !== "global") {
      setRankings([]);
      setTotalRankings(0);
      setLoading(false);
      return;
    }

    setLoading(true);
    try {
      const page = await LeaderboardService.listRankings({
        page: currentPage,
        pageSize: itemsPerPage,
      });
      setRankings(page.items);
      setTotalRankings(page.total);
    } catch {
      showToast(t('leaderboard_load_failed', 'Failed to load leaderboard.'), 'error');
      setRankings([]);
      setTotalRankings(0);
    } finally {
      setLoading(false);
    }
  }, [activeTab, currentPage, itemsPerPage, showToast, t]);

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

  const totalPages = Math.max(1, Math.ceil(totalRankings / itemsPerPage));
  const currentRankings = rankings;

  const handlePageChange = (page: number) => {
    if (page >= 1 && page <= totalPages) {
      setCurrentPage(page);
    }
  };

  const handleChallengeConfirm = (amount: number) => {
    showToast(t('challenged_alert', { name: selectedPlayer?.name, points: amount }), 'success');
    setShowChallengeModal(false);
  };

  const handleArenaPublishConfirm = (amount: number) => {
    showToast(t('arena_success_alert', { points: amount }), 'success');
    setShowArenaModal(false);
  };

  return (
    <div className="space-y-6 pb-12 h-full flex flex-col">
      <ToastComponent />
      
      {/* Header */}
      <LeaderboardHeader onSetupArena={() => setShowArenaModal(true)} />

      {/* Controls Area */}
      <div className="flex flex-col gap-4 shrink-0">
        {/* Main Categories */}
        <div className="flex space-x-2 overflow-x-auto pb-2 scrollbar-hide">
          {tabs.map((tab) => (
            <button
              key={tab.id}
              onClick={() => setActiveTab(tab.id)}
              className={`flex items-center space-x-2 px-6 py-3.5 rounded-2xl text-sm font-black tracking-wide transition-all ${
                activeTab === tab.id
                  ? "bg-gradient-to-r from-rose-600 to-orange-600 text-white shadow-[0_0_20px_rgba(225,29,72,0.3)] border border-rose-400/50"
                  : "bg-white/80 dark:bg-zinc-900/80 text-zinc-600 dark:text-zinc-400 hover:bg-zinc-50 dark:hover:bg-zinc-800 hover:text-zinc-900 dark:hover:text-zinc-200 border border-zinc-200 dark:border-zinc-800 shadow-sm dark:shadow-none"
              }`}
            >
              {tab.icon}
              <span>{tab.name}</span>
            </button>
          ))}
        </div>

        {/* Time Ranges Segmented Control & Refresh Info */}
        <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-4 bg-white/40 dark:bg-zinc-900/40 p-2 rounded-2xl border border-zinc-200/50 dark:border-zinc-800/50 backdrop-blur-sm shadow-sm dark:shadow-none">
          <div className="flex items-center space-x-1 overflow-x-auto scrollbar-hide w-full sm:w-auto">
            {timeRanges.map((range) => (
              <button
                key={range.id}
                onClick={() => setTimeRange(range.id)}
                className="relative px-5 py-2.5 text-sm font-bold rounded-xl whitespace-nowrap transition-colors flex-1 sm:flex-none"
              >
                {timeRange === range.id && (
                  <motion.div
                    layoutId="timeTabIndicator"
                    className="absolute inset-0 bg-white dark:bg-zinc-800 border border-zinc-200/50 dark:border-zinc-700/50 shadow-sm rounded-xl"
                    transition={{ type: "spring", bounce: 0.2, duration: 0.6 }}
                  />
                )}
                <span className={`relative z-10 ${timeRange === range.id ? 'text-zinc-900 dark:text-white' : 'text-zinc-500 hover:text-zinc-700 dark:hover:text-zinc-300'}`}>
                  {range.name}
                </span>
              </button>
            ))}
          </div>
          
          <div className="flex items-center justify-center space-x-2 px-4 py-2.5 bg-zinc-100/50 dark:bg-zinc-950/50 rounded-xl border border-zinc-200/50 dark:border-zinc-800/50 text-xs font-medium text-zinc-500 dark:text-zinc-400 shrink-0">
            <Timer size={14} className="text-rose-500 animate-pulse" />
            <span>{t('next_refresh_in')} <span className="text-zinc-900 dark:text-zinc-200 font-mono font-bold">14:23</span></span>
          </div>
        </div>
      </div>

      {/* Leaderboard List */}
      <div className="flex-1 bg-white/80 dark:bg-zinc-900/80 backdrop-blur-sm rounded-3xl border border-zinc-200 dark:border-zinc-800 overflow-hidden flex flex-col shadow-lg dark:shadow-2xl relative min-h-[500px]">
        {/* Table Header */}
        <div className="grid grid-cols-12 gap-4 p-4 border-b border-zinc-200 dark:border-zinc-800 text-xs font-black tracking-wider text-zinc-500 uppercase bg-zinc-50/80 dark:bg-zinc-950/80 shrink-0">
          <div className="col-span-1 text-center">{t('rank')}</div>
          <div className="col-span-3">{t('player_ai')}</div>
          <div className="col-span-2 text-center">{activeTab === "ai" ? t('provider') : t('title')}</div>
          <div className="col-span-2 text-center">{t('win_rate')}</div>
          <div className="col-span-2 text-right">{activeTab === "ai" ? t('compute_power') : t('points')}</div>
          <div className="col-span-2 text-center">{t('action')}</div>
        </div>

        {/* List Content */}
        <div className="flex-1 overflow-y-auto p-2 space-y-2 pb-36 scrollbar-hide">
          <AnimatePresence mode="wait">
            {loading ? (
              <div className="flex h-48 items-center justify-center text-zinc-500">
                <Loader2 className="mr-2 animate-spin" size={20} />
                <span>{t('loading', 'Loading...')}</span>
              </div>
            ) : activeTab !== "global" ? (
              <div className="flex h-48 items-center justify-center px-6 text-center text-sm font-medium text-zinc-500">
                {t('leaderboard_tab_coming_soon', 'This leaderboard view will be available when the platform API adds segment filters.')}
              </div>
            ) : currentRankings.length === 0 ? (
              <div className="flex h-48 items-center justify-center text-sm font-medium text-zinc-500">
                {t('leaderboard_empty', 'No rankings yet.')}
              </div>
            ) : (
            <motion.div 
              key={currentPage + timeRange + activeTab}
              initial={{ opacity: 0, y: 10 }}
              animate={{ opacity: 1, y: 0 }}
              exit={{ opacity: 0, y: -10 }}
              transition={{ duration: 0.2 }}
              className="space-y-2"
            >
              {currentRankings.map((user) => (
                <div
                  key={`${user.rank}-${user.name}`}
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
                  <div className="col-span-1 flex justify-center cursor-pointer" onClick={() => onViewPlayer && onViewPlayer(user)}>
                    {user.rank === 1 ? (
                      <Crown className="text-yellow-500 drop-shadow-[0_0_8px_rgba(234,179,8,0.8)]" size={24} />
                    ) : user.rank === 2 ? (
                      <Medal className="text-zinc-400 drop-shadow-[0_0_5px_rgba(161,161,170,0.5)]" size={22} />
                    ) : user.rank === 3 ? (
                      <Medal className="text-orange-500 drop-shadow-[0_0_5px_rgba(249,115,22,0.5)]" size={22} />
                    ) : (
                      <span className={`text-lg font-black ${user.rank <= 10 ? 'text-rose-500/80' : 'text-zinc-500 dark:text-zinc-600'}`}>{user.rank}</span>
                    )}
                  </div>

                  <div className="col-span-3 flex items-center space-x-3 cursor-pointer" onClick={() => onViewPlayer && onViewPlayer(user)}>
                    <div className="relative">
                      <img
                        src={user.avatar}
                        alt={user.name}
                        className={`w-10 h-10 rounded-xl object-cover border-2 ${
                          user.type === "AI" ? "border-rose-500" : "border-orange-500"
                        }`}
                      />
                      <div className={`absolute -bottom-1.5 -right-1.5 w-5 h-5 rounded-md flex items-center justify-center border-2 border-white dark:border-zinc-900 ${
                          user.type === "AI" ? "bg-rose-600" : "bg-orange-600"
                        }`}>
                        {user.type === "AI" ? <Cpu size={10} className="text-white" /> : <User size={10} className="text-white" />}
                      </div>
                    </div>
                    <div className="truncate">
                      <span className={`font-black text-base block truncate ${
                          user.rank === 1 ? "text-transparent bg-clip-text bg-gradient-to-r from-yellow-500 to-amber-600 dark:from-yellow-300 dark:to-amber-500" : 
                          user.rank === 2 ? "text-transparent bg-clip-text bg-gradient-to-r from-zinc-500 to-zinc-700 dark:from-zinc-200 dark:to-zinc-400" :
                          user.rank === 3 ? "text-transparent bg-clip-text bg-gradient-to-r from-orange-500 to-orange-700 dark:from-orange-300 dark:to-orange-500" :
                          user.rank <= 10 ? "text-zinc-900 dark:text-white" :
                          "text-zinc-700 dark:text-zinc-200"
                        }`}>
                        {user.name}
                      </span>
                    </div>
                  </div>

                  <div className="col-span-2 flex justify-center cursor-pointer" onClick={() => onViewPlayer && onViewPlayer(user)}>
                    {user.type === "AI" && user.provider ? (
                      <span className="px-2.5 py-1 text-[10px] font-black tracking-wider rounded-md border bg-zinc-150 dark:bg-zinc-850 text-zinc-600 dark:text-zinc-300 border-zinc-200 dark:border-zinc-700">
                        {user.provider}
                      </span>
                    ) : (
                      <span className={`px-2.5 py-1 text-[10px] font-black tracking-wider rounded-md border ${
                          user.type === "AI"
                            ? "bg-rose-50 dark:bg-rose-500/10 text-rose-600 dark:text-rose-400 border-rose-200 dark:border-rose-500/20"
                            : "bg-orange-50 dark:bg-orange-500/10 text-orange-600 dark:text-orange-400 border-orange-200 dark:border-orange-500/20"
                        }`}>
                        {user.title}
                      </span>
                    )}
                  </div>

                  <div className="col-span-2 text-center cursor-pointer" onClick={() => onViewPlayer && onViewPlayer(user)}>
                    <div className="inline-flex items-center space-x-1 bg-zinc-50 dark:bg-zinc-950 px-2 py-1 rounded-md border border-zinc-200 dark:border-zinc-800">
                      <span className="font-mono text-emerald-600 dark:text-emerald-400 font-bold text-xs">{user.winRate}</span>
                    </div>
                  </div>

                  <div className="col-span-2 flex items-center justify-end cursor-pointer" onClick={() => onViewPlayer && onViewPlayer(user)}>
                    <div className={`font-mono font-black text-xl ${
                      user.rank <= 3 ? "text-transparent bg-clip-text bg-gradient-to-br from-zinc-900 to-zinc-500 dark:from-white dark:to-zinc-400" : "text-zinc-700 dark:text-zinc-300"
                    }`}>
                      {user.score.toLocaleString()}
                    </div>
                  </div>
                  
                  <div className="col-span-2 flex items-center justify-center">
                    <button 
                      onClick={(e) => {
                        e.stopPropagation();
                        setSelectedPlayer(user);
                        setShowChallengeModal(true);
                      }}
                      className="flex items-center space-x-1 px-3 py-1.5 bg-white dark:bg-zinc-800 hover:bg-rose-600 dark:hover:bg-rose-600 text-zinc-600 dark:text-zinc-300 hover:text-white rounded-lg text-xs font-bold transition-all border border-zinc-200 dark:border-zinc-700 hover:border-transparent active:scale-95 shadow-sm"
                    >
                      <Swords size={14} />
                      <span>{t('challenge')}</span>
                    </button>
                  </div>
                </div>
              ))}
            </motion.div>
            )}
          </AnimatePresence>
        </div>

        {/* Pagination Controls */}
        <div className="absolute bottom-[88px] left-0 right-0 p-3 bg-white/95 dark:bg-zinc-950/95 backdrop-blur-md border-t border-zinc-200 dark:border-zinc-800 flex flex-col sm:flex-row items-center justify-between z-20 gap-2 shadow-[0_-10px_20px_rgba(0,0,0,0.05)] dark:shadow-none">
          <div className="text-sm text-zinc-500 dark:text-zinc-400 font-medium px-4">
            {t('showing_pagination', { start: totalRankings === 0 ? 0 : (currentPage - 1) * itemsPerPage + 1, end: Math.min(currentPage * itemsPerPage, totalRankings), total: totalRankings })}
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

        {/* Sticky Footer: My Rank */}
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
                    profile?.avatar
                    ?? myRanking?.avatar
                    ?? `https://api.dicebear.com/7.x/initials/svg?seed=${encodeURIComponent(profile?.id ?? 'guest')}`
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
                  {t('me')} ({profile?.username ?? myRanking?.name ?? t('guest', 'Guest')})
                </span>
                <span className="text-xs text-rose-500 dark:text-rose-400 font-medium">
                  {myRanking
                    ? t('points_to_next_rank', { points: Math.max(0, (myRanking.score ?? 0) - (currentRankings[0]?.score ?? 0)) })
                    : t('not_ranked_yet', 'Not ranked yet')}
                </span>
              </div>
            </div>
            <div className="col-span-2 flex justify-center">
              <span className="px-3 py-1 text-xs font-black tracking-wider rounded-lg border bg-zinc-100 dark:bg-zinc-800/50 text-zinc-600 dark:text-zinc-400 border-zinc-200 dark:border-zinc-700">
                {t('rising_star')}
              </span>
            </div>
            <div className="col-span-2 text-center">
              <div className="inline-flex items-center space-x-1 bg-zinc-50 dark:bg-zinc-900 px-3 py-1 rounded-lg border border-zinc-200 dark:border-zinc-800">
                <span className="font-mono text-emerald-600 dark:text-emerald-400 font-bold">
                  {myRanking?.winRate ?? "?"}
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

      {/* MoDals */}
      <ChallengeModal 
        isOpen={showChallengeModal} 
        onClose={() => setShowChallengeModal(false)} 
        player={selectedPlayer}
        wagerAmount={wagerAmount}
        setWagerAmount={setWagerAmount}
        onConfirm={handleChallengeConfirm}
      />

      <ArenaModal 
        isOpen={showArenaModal} 
        onClose={() => setShowArenaModal(false)} 
        wagerAmount={wagerAmount}
        setWagerAmount={setWagerAmount}
        onConfirm={handleArenaPublishConfirm}
      />
    </div>
  );
}
