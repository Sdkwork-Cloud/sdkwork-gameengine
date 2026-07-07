import React, { useEffect, useState } from 'react';
import { isBlank } from '@sdkwork/utils';
import { Gamepad2, Settings, Shield, Unlock, Users, X } from 'lucide-react';
import { AnimatePresence, motion } from 'motion/react';
import { useTranslation } from 'react-i18next';

export interface CreateRoomFormValues {
  joinPolicy: 'open' | 'invite';
  maxPlayers: number;
  modeId?: string;
  roomCode: string;
  visibility: 'public' | 'private';
}

interface CreateRoomModalProps {
  defaultRoomName?: string;
  isOpen: boolean;
  onClose: () => void;
  onCreateRoom: (values: CreateRoomFormValues) => Promise<void> | void;
}

export default function CreateRoomModal({
  defaultRoomName,
  isOpen,
  onClose,
  onCreateRoom,
}: CreateRoomModalProps) {
  const { t } = useTranslation();
  const [roomName, setRoomName] = useState(defaultRoomName ?? '');
  const [isPrivate, setIsPrivate] = useState(false);
  const [gameMode, setGameMode] = useState('standard');
  const [playerLimit, setPlayerLimit] = useState(2);
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [submitError, setSubmitError] = useState<string | null>(null);

  useEffect(() => {
    if (isOpen) {
      setRoomName(defaultRoomName ?? '');
      setSubmitError(null);
    }
  }, [defaultRoomName, isOpen]);

  if (!isOpen) return null;

  const handleSubmit = async () => {
    if (isBlank(roomName)) {
      setSubmitError(t('room_name_required', 'Room name is required'));
      return;
    }

    setIsSubmitting(true);
    setSubmitError(null);
    try {
      await onCreateRoom({
        roomCode: roomName.trim(),
        modeId: gameMode,
        maxPlayers: playerLimit,
        visibility: isPrivate ? 'private' : 'public',
        joinPolicy: isPrivate ? 'invite' : 'open',
      });
      onClose();
    } catch (error) {
      const message = error instanceof Error ? error.message : t('create_room_failed');
      setSubmitError(message || t('create_room_failed'));
    } finally {
      setIsSubmitting(false);
    }
  };

  return (
    <AnimatePresence>
      <div className="fixed inset-0 z-50 flex items-center justify-center px-4">
        <motion.div
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          exit={{ opacity: 0 }}
          className="absolute inset-0 bg-white/80 dark:bg-zinc-950/80 backdrop-blur-md"
          onClick={isSubmitting ? undefined : onClose}
        />

        <motion.div
          initial={{ opacity: 0, scale: 0.95, y: 20 }}
          animate={{ opacity: 1, scale: 1, y: 0 }}
          exit={{ opacity: 0, scale: 0.95, y: 20 }}
          className="relative w-full max-w-lg bg-white dark:bg-zinc-900 border border-zinc-200 dark:border-zinc-800 rounded-3xl shadow-2xl overflow-hidden"
        >
          <div className="flex items-center justify-between p-6 border-b border-zinc-200 dark:border-zinc-800 bg-zinc-50/50 dark:bg-zinc-950/50">
            <h2 className="text-2xl font-black text-zinc-900 dark:text-white flex items-center gap-2">
              <Users className="text-orange-500" />
              {t('create_room')}
            </h2>
            <button
              disabled={isSubmitting}
              onClick={onClose}
              className="p-2 bg-zinc-100 dark:bg-zinc-800/50 hover:bg-zinc-200 dark:hover:bg-zinc-700 text-zinc-500 dark:text-zinc-400 hover:text-zinc-900 dark:hover:text-white rounded-full transition-colors disabled:cursor-not-allowed disabled:opacity-60"
            >
              <X size={20} />
            </button>
          </div>

          <div className="p-6 space-y-6">
            <div className="space-y-2">
              <label className="text-sm font-bold text-zinc-500 dark:text-zinc-400 flex items-center gap-2">
                <Settings size={16} /> {t('room_name')}
              </label>
              <input
                type="text"
                value={roomName}
                onChange={(event) => setRoomName(event.target.value)}
                className="w-full bg-zinc-50 dark:bg-zinc-950 border border-zinc-200 dark:border-zinc-800 rounded-xl px-4 py-3 text-zinc-900 dark:text-white focus:outline-none focus:border-orange-500 transition-colors"
                placeholder={t('enter_room_name')}
              />
            </div>

            <div className="space-y-2">
              <label className="text-sm font-bold text-zinc-500 dark:text-zinc-400 flex items-center gap-2">
                <Gamepad2 size={16} /> {t('game_mode')}
              </label>
              <div className="grid grid-cols-2 gap-3">
                {[
                  { id: 'standard', label: t('standard_competitive') },
                  { id: 'casual', label: t('casual_entertainment') },
                ].map((mode) => (
                  <button
                    key={mode.id}
                    onClick={() => setGameMode(mode.id)}
                    className={`py-3 px-4 rounded-xl border font-bold text-sm transition-all ${
                      gameMode === mode.id
                        ? 'bg-orange-50 dark:bg-orange-500/10 border-orange-500 text-orange-600 dark:text-orange-500'
                        : 'bg-zinc-50 dark:bg-zinc-950 border-zinc-200 dark:border-zinc-800 text-zinc-500 dark:text-zinc-400 hover:border-zinc-300 dark:hover:border-zinc-700'
                    }`}
                  >
                    {mode.label}
                  </button>
                ))}
              </div>
            </div>

            <div className="space-y-2">
              <label className="text-sm font-bold text-zinc-500 dark:text-zinc-400 flex items-center gap-2">
                <Users size={16} /> {t('player_count')}
              </label>
              <div className="flex items-center gap-4 bg-zinc-50 dark:bg-zinc-950 border border-zinc-200 dark:border-zinc-800 rounded-xl p-2">
                {[2, 3, 4, 8].map((num) => (
                  <button
                    key={num}
                    onClick={() => setPlayerLimit(num)}
                    className={`flex-1 py-2 rounded-lg font-bold text-sm transition-colors ${
                      playerLimit === num
                        ? 'bg-white dark:bg-zinc-800 text-zinc-900 dark:text-white shadow-sm dark:shadow-none'
                        : 'text-zinc-500 hover:text-zinc-700 dark:hover:text-zinc-300'
                    }`}
                  >
                    {t('players_count', { count: num })}
                  </button>
                ))}
              </div>
            </div>

            <div className="space-y-4 pt-4 border-t border-zinc-200 dark:border-zinc-800">
              <div className="flex items-center justify-between">
                <label className="text-sm font-bold text-zinc-500 dark:text-zinc-400 flex items-center gap-2">
                  <Shield size={16} /> {t('private_room')}
                </label>
                <button
                  onClick={() => setIsPrivate(!isPrivate)}
                  className={`w-12 h-6 rounded-full transition-colors relative ${
                    isPrivate ? 'bg-orange-500' : 'bg-zinc-200 dark:bg-zinc-800'
                  }`}
                >
                  <motion.div
                    layout
                    className="w-4 h-4 bg-white rounded-full absolute top-1 left-1 shadow-sm"
                    animate={{ x: isPrivate ? 24 : 0 }}
                    transition={{ type: 'spring', stiffness: 500, damping: 30 }}
                  />
                </button>
              </div>

              <div className="flex items-center gap-2 text-xs text-zinc-500 dark:text-zinc-400">
                <Unlock size={14} />
                <span>
                  {isPrivate
                    ? t('invite_only_room', 'Private rooms require invited participants.')
                    : t('open_join_room', 'Public rooms can be joined by eligible players.')}
                </span>
              </div>
            </div>

            {submitError && (
              <div className="rounded-xl border border-rose-200 dark:border-rose-500/30 bg-rose-50 dark:bg-rose-500/10 px-4 py-3 text-sm font-medium text-rose-700 dark:text-rose-300">
                {submitError}
              </div>
            )}
          </div>

          <div className="p-6 bg-zinc-50/50 dark:bg-zinc-950/50 border-t border-zinc-200 dark:border-zinc-800 flex gap-4">
            <button
              disabled={isSubmitting}
              onClick={onClose}
              className="flex-1 py-3.5 bg-zinc-100 dark:bg-zinc-800 hover:bg-zinc-200 dark:hover:bg-zinc-700 text-zinc-900 dark:text-white rounded-xl font-bold transition-colors disabled:cursor-not-allowed disabled:opacity-60"
            >
              {t('cancel')}
            </button>
            <button
              disabled={isSubmitting}
              onClick={handleSubmit}
              className="flex-1 py-3.5 bg-gradient-to-r from-orange-600 to-rose-600 hover:from-orange-500 hover:to-rose-500 text-white rounded-xl font-bold transition-colors shadow-[0_0_20px_rgba(249,115,22,0.3)] disabled:cursor-not-allowed disabled:opacity-60"
            >
              {isSubmitting ? t('creating_room', 'Creating') : t('create_room')}
            </button>
          </div>
        </motion.div>
      </div>
    </AnimatePresence>
  );
}
