import React, { useState } from "react";
import { useTranslation } from "react-i18next";
import { Cpu, Trophy, Star } from "lucide-react";
import { MatchmakingModal, CreateRoomModal } from "sdkwork-gameengine-pc-commons";

// Imported split components
import DashboardHero from "../components/Dashboard/DashboardHero";
import StatsOverview from "../components/Dashboard/StatsOverview";
import LiveMatchesPreview from "../components/Dashboard/LiveMatchesPreview";
import DailyMissions from "../components/Dashboard/DailyMissions";
import LeaderboardPreview from "../components/Dashboard/LeaderboardPreview";

interface DashboardProps {
  setCurrentView?: (view: string) => void;
}

export default function Dashboard({ setCurrentView }: DashboardProps) {
  const { t } = useTranslation();
  const [isMatchmakingOpen, setIsMatchmakingOpen] = useState(false);
  const [isCreateRoomOpen, setIsCreateRoomOpen] = useState(false);
  const [leaderboardTab, setLeaderboardTab] = useState("season");

  const rankingsData: Record<string, any[]> = {
    daily: [
      { rank: 1, name: "快手_阿飞", type: "Human", score: 1250, trend: "up" },
      { rank: 2, name: "Bot_Model_X", type: "AI", score: 1180, trend: "up" },
      { rank: 3, name: "夜猫�?, type: "Human", score: 1025, trend: "down" },
      { rank: 4, name: "算力节点_01", type: "AI", score: 960, trend: "up" },
      { rank: 5, name: "雀神附�?, type: "Human", score: 945, trend: "up" },
      { rank: 6, name: "AlphaGo_Lite", type: "AI", score: 920, trend: "down" },
      { rank: 7, name: "摸鱼达人", type: "Human", score: 890, trend: "up" },
      { rank: 8, name: "DeepAgent_Mini", type: "AI", score: 875, trend: "up" },
      { rank: 9, name: "Player_112", type: "Human", score: 860, trend: "down" },
      { rank: 10, name: "AI_Tester", type: "AI", score: 855, trend: "up" },
    ],
    season: [
      { rank: 1, name: "AlphaGo_V4", type: "AI", score: 12500, trend: "up" },
      { rank: 2, name: "人类_柯洁", type: "Human", score: 11820, trend: "up" },
      { rank: 3, name: "DeepAgent", type: "AI", score: 10250, trend: "down" },
      { rank: 4, name: "赌神高进", type: "Human", score: 9600, trend: "up" },
      { rank: 5, name: "Libratus", type: "AI", score: 9450, trend: "up" },
      { rank: 6, name: "四川麻将�?, type: "Human", score: 9200, trend: "down" },
      { rank: 7, name: "AI猎手_007", type: "Human", score: 8900, trend: "up" },
      { rank: 8, name: "Bot_Model_3", type: "AI", score: 8750, trend: "up" },
      { rank: 9, name: "Player_8848", type: "Human", score: 8600, trend: "down" },
      { rank: 10, name: "算力节点_99", type: "AI", score: 8550, trend: "up" },
    ],
    allTime: [
      { rank: 1, name: "AlphaZero", type: "AI", score: 99999, trend: "up" },
      { rank: 2, name: "人类_李世�?, type: "Human", score: 88500, trend: "up" },
      { rank: 3, name: "Pluribus", type: "AI", score: 85200, trend: "down" },
      { rank: 4, name: "棋圣_聂卫�?, type: "Human", score: 82100, trend: "up" },
      { rank: 5, name: "DeepBlue", type: "AI", score: 79000, trend: "up" },
      { rank: 6, name: "雀�?, type: "Human", score: 75000, trend: "down" },
      { rank: 7, name: "AI_Master", type: "AI", score: 72000, trend: "up" },
      { rank: 8, name: "Bot_Omega", type: "AI", score: 68000, trend: "up" },
      { rank: 9, name: "Player_1", type: "Human", score: 65000, trend: "down" },
      { rank: 10, name: "算力巅峰", type: "AI", score: 62000, trend: "up" },
    ]
  };

  const topRankings = rankingsData[leaderboardTab] || rankingsData.season;

  const liveMatches = [
    {
      id: 1,
      game: t('go'),
      player1: "AlphaGo_V4 (S�?",
      player2: "人类_柯洁",
      status: t('in_battle'),
      viewers: "125k",
    },
    {
      id: 2,
      game: t('doudizhu'),
      player1: "DeepAgent (A�?",
      player2: "人类_赌神",
      status: t('endgame'),
      viewers: "45k",
    },
  ];

  const dailyMissions = [
    { id: 1, title: t('mission_1'), progress: 2, total: 3, reward: t('50_points'), icon: <Cpu size={16} /> },
    { id: 2, title: t('mission_2'), progress: 0, total: 1, reward: t('100_points'), icon: <Trophy size={16} /> },
    { id: 3, title: t('mission_3'), progress: 1, total: 1, reward: t('exclusive_title'), icon: <Star size={16} />, completed: true },
  ];

  const handleNavigate = (view: string) => {
    if (setCurrentView) {
      setCurrentView(view);
    }
  };

  return (
    <div className="space-y-8 pb-12">
      {/* Hero Section */}
      <DashboardHero 
        onQuickMatch={() => setIsMatchmakingOpen(true)}
        onCreateRoom={() => setIsCreateRoomOpen(true)}
        onNavigate={handleNavigate}
      />

      <div className="grid grid-cols-1 xl:grid-cols-3 gap-8">
        {/* Left Column: Live Matches & Missions */}
        <div className="xl:col-span-2 space-y-8">
          
          {/* Stats Overview */}
          <StatsOverview />

          {/* Live Matches */}
          <LiveMatchesPreview matches={liveMatches} />

          {/* Daily Missions */}
          <DailyMissions missions={dailyMissions} />
        </div>

        {/* Right Column: Leaderboard Preview & Quick Access */}
        <LeaderboardPreview 
          topRankings={topRankings}
          leaderboardTab={leaderboardTab}
          setLeaderboardTab={setLeaderboardTab}
          onNavigate={handleNavigate}
        />
      </div>

      <MatchmakingModal 
        isOpen={isMatchmakingOpen} 
        onClose={() => setIsMatchmakingOpen(false)} 
      />
      <CreateRoomModal 
        isOpen={isCreateRoomOpen} 
        onClose={() => setIsCreateRoomOpen(false)} 
      />
    </div>
  );
}
