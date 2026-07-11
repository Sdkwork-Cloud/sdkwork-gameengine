import React from "react";
import { useTranslation } from "react-i18next";
import { Activity, Gamepad2, Users } from "lucide-react";
import { motion } from "motion/react";

import { LiveMatch } from "../../types/game.types";

interface LiveMatchesGridProps {
  liveMatches: LiveMatch[];
}

export default function LiveMatchesGrid({ liveMatches }: LiveMatchesGridProps) {
  const { t } = useTranslation();
  const renderSeat = (room: LiveMatch, index: 0 | 1) => {
    const seat = room.teams[index];
    return (
      <div className="flex flex-col items-center gap-3 w-1/3">
        <div className="relative group-hover:scale-110 transition-transform duration-500">
          <div className="w-16 h-16 rounded-2xl bg-zinc-100 dark:bg-zinc-800 border-2 border-white dark:border-zinc-700 overflow-hidden shadow-lg z-10 relative group-hover:border-emerald-500/50 transition-colors">
            <img
              src={`https://api.dicebear.com/7.x/initials/svg?seed=${encodeURIComponent(seat.avatarSeed)}`}
              alt={t('player', 'Player')}
              className="w-full h-full object-cover"
            />
          </div>
          <div className="absolute -bottom-2 -right-2 w-6 h-6 bg-emerald-500 rounded-full border-2 border-white dark:border-zinc-900 flex items-center justify-center z-20 shadow-md">
            <span className="text-[10px] text-white font-black">P{index + 1}</span>
          </div>
        </div>
        <span className="text-sm font-black text-zinc-900 dark:text-white truncate w-full text-center group-hover:text-emerald-500 transition-colors">
          {t(seat.nameKey, index === 0 ? room.roomCode : t('waiting_opponent', 'Waiting seat'))}
        </span>
      </div>
    );
  };

  return (
    <div className="shrink-0 mt-8">
      <div className="flex items-center justify-between mb-6">
        <h2 className="text-2xl font-black text-zinc-900 dark:text-white flex items-center gap-2">
          <Activity className="text-emerald-500" />
          {t('live_rooms', 'Active Rooms')}
        </h2>
      </div>
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
        {liveMatches.map((room) => (
          <motion.div
            whileHover={{ y: -4 }}
            key={room.id}
            className="bg-white dark:bg-zinc-900/80 backdrop-blur-xl rounded-[2rem] border border-zinc-200/50 dark:border-zinc-800/50 p-6 flex flex-col gap-6 hover:shadow-2xl hover:shadow-emerald-500/10 hover:border-emerald-500/30 transition-all duration-500 group relative overflow-hidden"
          >
            <div className="absolute top-0 left-0 w-full h-1 bg-gradient-to-r from-emerald-500 via-teal-500 to-emerald-500 opacity-0 group-hover:opacity-100 transition-opacity bg-[length:200%_auto] animate-gradient" />
            <div className="absolute inset-0 bg-gradient-to-br from-emerald-500/5 to-transparent opacity-0 group-hover:opacity-100 transition-opacity duration-500 pointer-events-none" />

            <div className="flex items-center justify-between relative z-10">
              <span className="text-xs font-black px-3 py-1.5 bg-emerald-50 dark:bg-emerald-500/10 text-emerald-600 dark:text-emerald-400 rounded-lg flex items-center gap-2 border border-emerald-100 dark:border-emerald-500/20 shadow-[0_0_10px_rgba(16,185,129,0.1)] group-hover:bg-emerald-500 group-hover:text-white transition-colors">
                <span className="w-2 h-2 rounded-full bg-emerald-500 group-hover:bg-white animate-ping" />
                {t('room_status_live', 'Live')}
              </span>
              <span className="text-xs font-bold text-zinc-500 flex items-center gap-1.5 bg-zinc-50 dark:bg-zinc-800/80 px-3 py-1.5 rounded-lg border border-zinc-200/50 dark:border-zinc-700/50 group-hover:border-emerald-500/30 group-hover:text-emerald-500 transition-colors">
                <Users size={12} className="text-emerald-500" />
                {room.currentPlayers}/{room.maxPlayers}
              </span>
            </div>

            <div className="flex items-center justify-between px-2 relative z-10">
              {renderSeat(room, 0)}
              <div className="flex flex-col items-center justify-center px-2 w-1/3">
                <div className="text-2xl font-black text-zinc-300 dark:text-zinc-700 italic tracking-tighter mb-1 group-hover:scale-110 transition-transform group-hover:text-emerald-500/50">
                  {room.currentPlayers}/{room.maxPlayers}
                </div>
                <span className="text-[10px] font-black text-emerald-500 bg-emerald-50 dark:bg-emerald-500/10 px-2.5 py-1 rounded-md border border-emerald-100 dark:border-emerald-500/20 whitespace-nowrap group-hover:bg-emerald-500 group-hover:text-white transition-colors">
                  {t('active_room', 'Active room')}
                </span>
              </div>
              {renderSeat(room, 1)}
            </div>

            <div className="w-full h-1.5 bg-zinc-100 dark:bg-zinc-800 rounded-full overflow-hidden relative z-10">
              <div
                className="h-full bg-emerald-500 transition-all"
                style={{
                  width: `${Math.min(
                    100,
                    Math.max(0, (room.currentPlayers / Math.max(1, room.maxPlayers)) * 100),
                  )}%`,
                }}
              />
            </div>

            <div className="pt-4 mt-2 border-t border-zinc-100 dark:border-zinc-800/50 flex justify-between items-center relative z-10">
              <div className="flex items-center gap-2 min-w-0">
                <div className="p-1.5 bg-zinc-100 dark:bg-zinc-800 rounded-lg">
                  <Gamepad2 size={14} className="text-zinc-500 dark:text-zinc-400" />
                </div>
                <span className="text-xs font-bold text-zinc-600 dark:text-zinc-400 truncate">
                  {t('room_code', 'Room')} {room.roomCode}
                </span>
              </div>
              <span className="text-xs font-black text-zinc-500 dark:text-zinc-400 bg-zinc-100 dark:bg-zinc-800/70 px-4 py-2 rounded-xl">
                {t('room_in_progress', 'In progress')}
              </span>
            </div>
          </motion.div>
        ))}
      </div>
    </div>
  );
}
