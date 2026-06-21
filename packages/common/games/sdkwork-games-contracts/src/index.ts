import { slugify } from '@sdkwork/utils';

export type GameCatalogStatus = 'draft' | 'published' | 'archived';

export interface GameCatalogSummary {
  id: string;
  gameCode: string;
  title: string;
  status: GameCatalogStatus;
}

export function normalizeGameCode(title: string): string {
  return slugify(title);
}

export const GAMES_APP_API_PREFIX = '/app/v3/api';
export const GAMES_BACKEND_API_PREFIX = '/backend/v3/api';
