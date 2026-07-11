import i18n from 'i18next';
import { initReactI18next } from 'react-i18next';

import { common as enCommon } from './locales/en/common';
import { gameCenter as enGameCenter } from './locales/en/gameCenter';
import { leaderboard as enLeaderboard } from './locales/en/leaderboard';
import { login as enLogin } from './locales/en/login';
import { common as zhCommon } from './locales/zh/common';
import { gameCenter as zhGameCenter } from './locales/zh/gameCenter';
import { leaderboard as zhLeaderboard } from './locales/zh/leaderboard';
import { login as zhLogin } from './locales/zh/login';

const resources = {
  en: {
    translation: {
      ...enCommon,
      ...enGameCenter,
      ...enLeaderboard,
      ...enLogin,
    },
  },
  zh: {
    translation: {
      ...zhCommon,
      ...zhGameCenter,
      ...zhLeaderboard,
      ...zhLogin,
    },
  },
};

i18n
  .use(initReactI18next)
  .init({
    resources,
    lng: 'zh',
    fallbackLng: 'en',
    interpolation: {
      escapeValue: false,
    },
  });

export default i18n;
export { i18n };
