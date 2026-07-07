import React, { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { Cpu, Trophy, Star } from "lucide-react";
import { MatchmakingModal, CreateRoomModal } from "sdkwork-gameengine-pc-commons";

import DashboardHero from "../components/Dashboard/DashboardHero";
import StatsOverview from "../components/Dashboard/StatsOverview";
import LiveMatchesPreview from "../components/Dashboard/LiveMatchesPreview";
import DailyMissions from "../components/Dashboard/DailyMissions";
import LeaderboardPreview from "../components/Dashboard/LeaderboardPreview";
import { GameService } from "../services/game.service";
import { LeaderboardService } from "../services/leaderboard.service";

interface DashboardProps {
  setCurrentView?: (view: string) => void;
}

interface DashboardLiveMatch {
  id: string | number;
  game: string;
  player1: string;
  player2: string;
  status: string;
  viewers: string;
}

interface DashboardRankingPreview {
  rank: number;
  name: string;
  type: string;
  score: number;
  trend: string;
}

export default function Dashboard({ setCurrentView }: DashboardProps) {
  const { t } = useTranslation();
  const [isMatchmakingOpen, setIsMatchmakingOpen] = useState(false);
  const [isCreateRoomOpen, setIsCreateRoomOpen] = useState(false);
  const [leaderboardTab, setLeaderboardTab] = useState("season");
  const [topRankings, setTopRankings] = useState<DashboardRankingPreview[]>([]);
  const [liveMatches, setLiveMatches] = useState<DashboardLiveMatch[]>([]);

  useEffect(() => {
    let isMounted = true;

    void LeaderboardService.listRankings({ page: 1, pageSize: 10 })
      .then((page) => {
        if (!isMounted) {
          return;
        }
        setTopRankings(
          page.items.map((item) => ({
            rank: item.rank,
            name: item.name,
            type: item.type,
            score: item.score,
            trend: item.trend.startsWith('-') ? 'down' : 'up',
          })),
        );
      })
      .catch(() => {
        if (isMounted) {
          setTopRankings([]);
        }
      });

    void GameService.getLiveMatches()
      .then((matches) => {
        if (!isMounted) {
          return;
        }
        setLiveMatches(
          matches.slice(0, 5).map((match) => ({
            id: match.id,
            game: match.gameNameKey,
            player1: match.teams[0]?.nameKey ?? t('player_one', 'Player 1'),
            player2: match.teams[1]?.nameKey ?? t('player_two', 'Player 2'),
            status: match.status === 'live' ? t('in_battle') : t('waiting', 'Waiting'),
            viewers: String(match.spectators),
          })),
        );
      })
      .catch(() => {
        if (isMounted) {
          setLiveMatches([]);
        }
      });

    return () => {
      isMounted = false;
    };
  }, [t]);

  const dailyMissions = [
    {
      id: 1,
      title: t('mission_1'),
      progress: 0,
      total: 3,
      reward: t('50_points'),
      icon: <Cpu size={16} />,
      completed: false,
    },
    {
      id: 2,
      title: t('mission_2'),
      progress: 0,
      total: 1,
      reward: t('100_points'),
      icon: <Trophy size={16} />,
      completed: false,
    },
    {
      id: 3,
      title: t('mission_3'),
      progress: 0,
      total: 1,
      reward: t('exclusive_title'),
      icon: <Star size={16} />,
      completed: false,
    },
  ];

  const handleNavigate = (view: string) => {
    if (setCurrentView) {
      setCurrentView(view);
    }
  };

  return (
    <div className="space-y-8 pb-12">
      <DashboardHero
        onQuickMatch={() => setIsMatchmakingOpen(true)}
        onCreateRoom={() => setIsCreateRoomOpen(true)}
        onNavigate={handleNavigate}
      />

      <div className="grid grid-cols-1 xl:grid-cols-3 gap-8">
        <div className="xl:col-span-2 space-y-8">
          <StatsOverview />
          <LiveMatchesPreview matches={liveMatches} />
          <DailyMissions missions={dailyMissions} />
        </div>

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
