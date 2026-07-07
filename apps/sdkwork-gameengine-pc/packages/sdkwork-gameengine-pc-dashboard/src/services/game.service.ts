import { isBlank } from '@sdkwork/utils';
import {
  getGamesCatalogService,
  getGamesRoomService,
  type CreateRoomRequest,
  type GameCatalogItem,
  type GameRoomItem,
} from 'sdkwork-gameengine-pc-core';

import type { Game, GetGamesParams, GameListResponse, LiveMatch } from '../types/game.types';

function mapCatalogItemToGame(item: GameCatalogItem): Game {
  return {
    id: item.id,
    name: item.gameCode,
    title: item.title,
    category: item.genre ?? 'action',
    desc: item.summary ?? item.title,
    status: item.status,
    tags: [item.genre ?? 'game', item.status],
  };
}

function mapSortBy(sortBy?: GetGamesParams['sortBy']): 'recommended' | 'title' | 'newest' | undefined {
  if (sortBy === 'newest') {
    return 'newest';
  }
  if (sortBy === 'title') {
    return 'title';
  }
  if (sortBy === 'recommended') {
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

    return {
      games: page.items.map(mapCatalogItemToGame),
      total: page.total,
    };
  }

  static async createRoom(input: CreateRoomRequest): Promise<GameRoomItem> {
    if (isBlank(input.roomCode)) {
      throw new Error('Room name is required.');
    }
    return getGamesRoomService().createRoom({
      ...input,
      roomCode: input.roomCode.trim(),
    });
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
