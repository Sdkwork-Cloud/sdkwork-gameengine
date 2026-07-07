export type Tag = string;

export interface Game {
  id: string | number;
  name: string;
  title: string;
  category: string;
  desc: string;
  img?: string;
  status: string;
  tags: Tag[];
}

export interface GameCategory {
  id: string;
  nameKey: string;
  iconName: string;
}

export interface FeatureBanner {
  id: string;
  titleKey: string;
  subtitleKey: string;
  image: string;
  tagKey: string;
  color?: string;
}

export interface LiveMatchPlayer {
  id: string;
  nameKey: string;
  avatarSeed: string;
  type: 'human' | 'ai';
}

export interface LiveMatch {
  id: string;
  gameId: string | number;
  gameNameKey: string;
  spectators: string | number;
  status: 'live';
  teams: [LiveMatchPlayer, LiveMatchPlayer];
}

export interface GameListResponse {
  games: Game[];
  total: number;
}

export type SortOption = 'recommended' | 'title' | 'newest';

export interface GetGamesParams {
  category?: string;
  searchQuery?: string;
  sortBy?: SortOption;
  limit?: number;
  offset?: number;
}
