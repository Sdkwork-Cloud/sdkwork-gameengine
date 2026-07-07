import React, { useEffect, useRef, useState } from 'react';
import { ChevronRight, Globe, LogOut, Monitor, Moon, Sun, Trophy } from 'lucide-react';
import { AnimatePresence, motion } from 'motion/react';
import { useTheme } from 'next-themes';
import { useTranslation } from 'react-i18next';
import { useConfigStore, useUserStore } from 'sdkwork-gameengine-pc-core';

interface TopbarProps {
  onLogout?: () => Promise<void> | void;
  setCurrentView?: (view: string) => void;
}

export default function Topbar({ onLogout, setCurrentView }: TopbarProps) {
  const { t } = useTranslation();
  const { region, language, setRegion, setLanguage } = useConfigStore();
  const { profile } = useUserStore();
  const { theme, setTheme } = useTheme();
  const [isDropdownOpen, setIsDropdownOpen] = useState(false);
  const [isLoggingOut, setIsLoggingOut] = useState(false);
  const dropdownRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    function handleClickOutside(event: MouseEvent) {
      if (dropdownRef.current && !dropdownRef.current.contains(event.target as Node)) {
        setIsDropdownOpen(false);
      }
    }
    document.addEventListener('mousedown', handleClickOutside);
    return () => document.removeEventListener('mousedown', handleClickOutside);
  }, []);

  const toggleLanguage = () => {
    setLanguage(language === 'zh' ? 'en' : 'zh');
  };

  const toggleRegion = () => {
    setRegion(region === 'CN' ? 'GLOBAL' : 'CN');
  };

  const cycleTheme = () => {
    if (theme === 'dark') {
      setTheme('light');
      return;
    }
    if (theme === 'light') {
      setTheme('system');
      return;
    }
    setTheme('dark');
  };

  const handleLogout = async () => {
    if (!onLogout || isLoggingOut) {
      return;
    }

    setIsLoggingOut(true);
    try {
      await onLogout();
    } catch (error) {
      console.error('Failed to sign out through IAM runtime', error);
    } finally {
      setIsLoggingOut(false);
      setIsDropdownOpen(false);
    }
  };

  return (
    <header className="h-20 border-b border-zinc-200 dark:border-zinc-800/50 bg-white/80 dark:bg-zinc-950/80 backdrop-blur-xl flex items-center justify-between px-8 sticky top-0 z-50 transition-colors duration-300">
      <div className="text-sm font-bold text-zinc-500 dark:text-zinc-400">
        {t('app_name_short', 'Game Center')}
      </div>

      <div className="flex items-center space-x-6">
        <button
          onClick={toggleRegion}
          className="flex items-center gap-1 text-zinc-500 dark:text-zinc-400 hover:text-zinc-900 dark:hover:text-zinc-200 transition-colors text-sm font-medium"
        >
          <Globe size={18} />
          <span className="hidden sm:inline">
            {region === 'CN' ? t('region_cn') : t('region_global')}
          </span>
        </button>
        <button
          onClick={toggleLanguage}
          className="text-zinc-500 dark:text-zinc-400 hover:text-zinc-900 dark:hover:text-zinc-200 transition-colors text-sm font-bold"
        >
          {language === 'zh' ? 'EN' : 'ZH'}
        </button>
        <button
          onClick={cycleTheme}
          className="text-zinc-500 dark:text-zinc-400 hover:text-zinc-900 dark:hover:text-zinc-200 transition-colors"
        >
          {theme === 'dark' ? (
            <Moon size={20} />
          ) : theme === 'light' ? (
            <Sun size={20} />
          ) : (
            <Monitor size={20} />
          )}
        </button>

        <div className="h-8 w-px bg-zinc-200 dark:bg-zinc-800 mx-2" />

        <div className="relative" ref={dropdownRef}>
          <div
            className="flex items-center space-x-3 cursor-pointer hover:bg-zinc-100 dark:hover:bg-zinc-900 p-2 rounded-xl transition-colors border border-transparent hover:border-zinc-200 dark:hover:border-zinc-800"
            onClick={() => setIsDropdownOpen(!isDropdownOpen)}
          >
            <div className="text-right hidden md:block">
              <p className="text-sm font-bold text-zinc-900 dark:text-zinc-200">
                {profile?.username || t('signed_in_user', 'Signed-in user')}
              </p>
              <p className="text-xs text-zinc-500 dark:text-zinc-400 font-medium">
                {profile?.tenantId || t('tenant_context_ready', 'Tenant context ready')}
              </p>
            </div>
            <div className="relative">
              <div className="w-10 h-10 rounded-full bg-zinc-200 dark:bg-zinc-800 border-2 border-white dark:border-zinc-800 p-0.5">
                <div className="w-full h-full rounded-full bg-zinc-100 dark:bg-zinc-900 flex items-center justify-center overflow-hidden">
                  <img
                    src={profile?.avatar || 'https://api.dicebear.com/7.x/initials/svg?seed=SDKWORK'}
                    alt={t('user_avatar', 'User avatar')}
                    className="w-full h-full object-cover"
                  />
                </div>
              </div>
            </div>
          </div>

          <AnimatePresence>
            {isDropdownOpen && (
              <motion.div
                initial={{ opacity: 0, y: 10, scale: 0.95 }}
                animate={{ opacity: 1, y: 0, scale: 1 }}
                exit={{ opacity: 0, y: 10, scale: 0.95 }}
                transition={{ duration: 0.15 }}
                className="absolute right-0 mt-2 w-72 bg-white/95 dark:bg-zinc-900/95 backdrop-blur-xl border border-zinc-200 dark:border-zinc-800 rounded-2xl shadow-2xl overflow-hidden z-50"
              >
                <div className="p-5 border-b border-zinc-200 dark:border-zinc-800/50 bg-gradient-to-b from-zinc-50 dark:from-zinc-800/30 to-transparent">
                  <div className="flex items-center space-x-3">
                    <div className="w-12 h-12 rounded-full bg-zinc-200 dark:bg-zinc-800 p-0.5">
                      <img
                        src={profile?.avatar || 'https://api.dicebear.com/7.x/initials/svg?seed=SDKWORK'}
                        alt={t('user_avatar', 'User avatar')}
                        className="w-full h-full rounded-full object-cover border-2 border-white dark:border-zinc-900"
                      />
                    </div>
                    <div className="min-w-0">
                      <h3 className="font-bold text-zinc-900 dark:text-white truncate">
                        {profile?.username || t('signed_in_user', 'Signed-in user')}
                      </h3>
                      <p className="text-xs text-zinc-500 dark:text-zinc-400 truncate">
                        ID: {profile?.id || '---'}
                      </p>
                      {profile?.sessionId && (
                        <p className="text-xs text-zinc-500 dark:text-zinc-400 truncate">
                          {profile.sessionId}
                        </p>
                      )}
                    </div>
                  </div>
                </div>

                <div className="p-2 space-y-1">
                  <button
                    onClick={() => {
                      setCurrentView?.('leaderboard');
                      setIsDropdownOpen(false);
                    }}
                    className="w-full flex items-center justify-between p-3 rounded-xl hover:bg-zinc-100 dark:hover:bg-zinc-800/50 text-zinc-700 dark:text-zinc-300 hover:text-zinc-900 dark:hover:text-white transition-colors group"
                  >
                    <div className="flex items-center space-x-3">
                      <Trophy
                        size={16}
                        className="text-zinc-400 dark:text-zinc-500 group-hover:text-yellow-500 transition-colors"
                      />
                      <span className="text-sm font-medium">{t('my_ranking')}</span>
                    </div>
                    <ChevronRight
                      size={14}
                      className="text-zinc-400 dark:text-zinc-600 group-hover:text-zinc-600 dark:group-hover:text-zinc-400"
                    />
                  </button>
                </div>

                <div className="p-2 border-t border-zinc-200 dark:border-zinc-800/50">
                  <button
                    disabled={isLoggingOut || !onLogout}
                    onClick={handleLogout}
                    className="w-full flex items-center space-x-3 p-3 rounded-xl hover:bg-rose-50 dark:hover:bg-rose-500/10 text-zinc-500 dark:text-zinc-400 hover:text-rose-600 dark:hover:text-rose-500 transition-colors disabled:cursor-not-allowed disabled:opacity-60"
                  >
                    <LogOut size={16} />
                    <span className="text-sm font-medium">
                      {isLoggingOut ? t('logging_out', 'Logging out') : t('logout')}
                    </span>
                  </button>
                </div>
              </motion.div>
            )}
          </AnimatePresence>
        </div>
      </div>
    </header>
  );
}
