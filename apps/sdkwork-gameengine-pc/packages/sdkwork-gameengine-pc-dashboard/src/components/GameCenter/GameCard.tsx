import React from 'react';
import { useTranslation } from 'react-i18next';
import { Activity, Gamepad2, Users } from 'lucide-react';
import { Game } from '../../types/game.types';

interface GameCardProps {
  categoryName?: string;
  game: Game;
  onCreateRoom: (game: Game) => void;
}

export default function GameCard({ categoryName, game, onCreateRoom }: GameCardProps) {
  const { t } = useTranslation();
  const displayName = game.title || t(game.name);

  return (
    <div className="group relative bg-white dark:bg-zinc-900/80 backdrop-blur-xl rounded-[2rem] border border-zinc-200/50 dark:border-zinc-800/50 overflow-hidden hover:shadow-2xl hover:shadow-emerald-500/10 hover:border-emerald-500/30 transition-all duration-500 flex flex-col">
      <div className="absolute inset-0 bg-gradient-to-br from-emerald-500/5 to-transparent opacity-0 group-hover:opacity-100 transition-opacity duration-500 pointer-events-none" />
      <div className="h-48 relative overflow-hidden shrink-0 bg-zinc-100 dark:bg-zinc-800">
        {game.img ? (
          <img
            src={game.img}
            alt={displayName}
            className="w-full h-full object-cover group-hover:scale-105 transition-transform duration-700 ease-out"
          />
        ) : (
          <div className="w-full h-full flex items-center justify-center">
            <Gamepad2 size={56} className="text-zinc-300 dark:text-zinc-600" />
          </div>
        )}
        <div className="absolute inset-0 bg-gradient-to-t from-zinc-900 via-zinc-900/10 to-transparent opacity-80" />

        <div className="absolute top-4 left-4 flex flex-col gap-2">
          <div className="bg-zinc-900/75 text-emerald-300 text-[10px] font-black px-2.5 py-1 rounded-md shadow-lg flex items-center gap-1 w-max backdrop-blur-md uppercase tracking-wider">
            <Activity size={12} />
            {game.status}
          </div>
        </div>

        <div className="absolute bottom-4 left-5 right-5">
          <h3 className="text-2xl font-black text-white tracking-tight drop-shadow-lg mb-2 group-hover:text-emerald-400 transition-colors">
            {displayName}
          </h3>
          <div className="flex flex-wrap gap-1.5">
            {game.tags.slice(0, 3).map((tag) => (
              <span
                key={tag}
                className="text-[10px] font-bold px-2 py-0.5 rounded-md bg-white/10 text-zinc-200 border border-white/10 backdrop-blur-md group-hover:border-emerald-500/30 group-hover:text-emerald-300 transition-colors"
              >
                {t(tag)}
              </span>
            ))}
          </div>
        </div>
      </div>

      <div className="p-5 flex-1 flex flex-col justify-between bg-white dark:bg-zinc-900/50 relative z-10">
        <div className="flex items-center justify-between mb-4">
          <div className="flex items-center gap-1.5 text-[10px] font-black text-emerald-600 dark:text-emerald-400 bg-emerald-50 dark:bg-emerald-500/10 px-2.5 py-1 rounded-md uppercase tracking-wider">
            <Gamepad2 size={12} />
            {game.category}
          </div>
          <span className="text-[10px] font-bold text-zinc-400 uppercase tracking-wider">
            {categoryName}
          </span>
        </div>

        <p className="text-sm text-zinc-600 dark:text-zinc-400 font-medium leading-relaxed line-clamp-2 mb-6">
          {game.desc}
        </p>

        <button
          onClick={() => onCreateRoom(game)}
          className="mt-auto w-full py-3.5 bg-zinc-900 dark:bg-white hover:bg-emerald-600 dark:hover:bg-emerald-500 text-white dark:text-zinc-900 hover:text-white rounded-xl font-black flex items-center justify-center space-x-2 transition-all duration-300 shadow-md hover:shadow-[0_0_20px_rgba(16,185,129,0.3)] active:scale-95 group-hover:bg-emerald-600 dark:group-hover:bg-emerald-500 group-hover:text-white"
        >
          <Users size={16} />
          <span>{t('create_room')}</span>
        </button>
      </div>
    </div>
  );
}
