import { isBlank } from '@sdkwork/utils';
import {
  getGamesCatalogService,
  getGamesRoomService,
  type GameCatalogItem,
  type GameRoomItem,
} from 'sdkwork-gameengine-pc-core';

import type { FeatureBanner, Game, GetGamesParams, GameListResponse, LiveMatch } from '../types/game.types';

function mapCatalogItemToGame(item: GameCatalogItem): Game {
  return {
    id: item.id,
    name: item.gameCode,
    category: item.genre ?? 'action',
    desc: item.summary ?? item.title,
    img: 'https://images.unsplash.com/photo-1511512578047-dfb367046420?w=800&q=80',
    isHot: item.status === 'active',
    playersOnline: '0',
    rating: 4.5,
    aiDifficulty: 'A',
    tags: [item.genre ?? 'game', item.status],
  };
}

function mapSortBy(sortBy?: GetGamesParams['sortBy']): 'recommended' | 'title' | 'newest' | undefined {
  if (sortBy === 'newest') {
    return 'newest';
  }
  if (sortBy === 'recommended' || sortBy === 'popular' || sortBy === 'rating') {
    return 'recommended';
  }
  return undefined;
}

function mapRoomToLiveMatch(room: GameRoomItem): LiveMatch {
  return {
    id: room.id,
    gameId: room.gameId,
    gameNameKey: room.gameId,
    spectators: String(room.currentPlayers),
    status: 'live',
    teams: [
      { id: `${room.id}-host`, nameKey: room.roomCode, avatarSeed: room.roomCode, type: 'human' },
      {
        id: `${room.id}-guest`,
        nameKey: 'waiting_opponent',
        avatarSeed: room.id,
        type: 'human',
      },
    ],
  };
}

function mapCatalogToBanner(item: GameCatalogItem, index: number): FeatureBanner {
  return {
    id: item.id,
    titleKey: item.title,
    subtitleKey: item.summary ?? item.gameCode,
    image: 'https://images.unsplash.com/photo-1511512578047-dfb367046420?w=1200&q=80',
    tagKey: index === 0 ? 'featured' : item.genre ?? 'game',
    color: 'from-rose-600 to-orange-600',
  };
}

export class GameService {
  static async getGames(params: GetGamesParams = {}): Promise<GameListResponse> {
    const page = await getGamesCatalogService().listCatalog({
      page: 1,
      pageSize: 20,
      status: 'active',
      genre: params.category && params.category !== 'all' && params.category !== 'featured'
        ? params.category
        : undefined,
      q: isBlank(params.searchQuery) ? undefined : params.searchQuery,
      sort: mapSortBy(params.sortBy),
    });

    let games = page.items.map(mapCatalogItemToGame);
    if (params.category === 'featured') {
      games = games.filter((game) => game.isHot || game.rating >= 4.5);
    }

    return {
      games,
      total: page.total,
    };
  }

  static async getFeaturedBanners(): Promise<FeatureBanner[]> {
    const page = await getGamesCatalogService().listCatalog({
      page: 1,
      pageSize: 3,
      status: 'active',
      sort: 'recommended',
    });
    return page.items.map(mapCatalogToBanner);
  }

  static async getRecentlyPlayed(): Promise<Game[]> {
    return [];
  }

  static async getLiveMatches(): Promise<LiveMatch[]> {
    const page = await getGamesRoomService().listRooms({
      status: 'in_progress',
      page: 1,
      pageSize: 20,
    });
    return page.items.map(mapRoomToLiveMatch);
  }
}
