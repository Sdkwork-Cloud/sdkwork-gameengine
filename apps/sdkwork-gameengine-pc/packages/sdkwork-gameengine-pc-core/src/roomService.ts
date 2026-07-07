import type {
  CreateRoomRequest,
  ExpectedVersionRequest,
  GameRoomItem,
  GameRoomSeatItem,
  JoinRoomRequest,
  ReadyRoomRequest,
  SdkWorkPageData,
} from '@sdkwork/gameengine-app-sdk';
import type { SdkworkGameengineAppClient } from '@sdkwork/gameengine-app-sdk';

export type {
  CreateRoomRequest,
  ExpectedVersionRequest,
  GameRoomItem,
  GameRoomSeatItem,
  JoinRoomRequest,
  ReadyRoomRequest,
};

export interface GameRoomPage {
  items: GameRoomItem[];
  total: number;
  page: number;
  pageSize: number;
}

export interface GameRoomSeatPage {
  items: GameRoomSeatItem[];
  total: number;
  page: number;
  pageSize: number;
}

export interface GamesRoomListParams {
  gameId?: string;
  status?: 'open' | 'in_progress' | 'closed';
  page?: number;
  pageSize?: number;
}

export interface GamesRoomService {
  listRooms(params?: GamesRoomListParams): Promise<GameRoomPage>;
  createRoom(input: CreateRoomRequest): Promise<GameRoomItem>;
  retrieveRoom(roomId: string): Promise<GameRoomItem>;
  listRoomSeats(roomId: string): Promise<GameRoomSeatPage>;
  joinRoom(roomId: string, input?: JoinRoomRequest): Promise<GameRoomItem>;
  leaveRoom(roomId: string, input?: ExpectedVersionRequest): Promise<GameRoomItem>;
  setRoomReady(roomId: string, input: ReadyRoomRequest): Promise<GameRoomItem>;
  startRoom(roomId: string, input?: ExpectedVersionRequest): Promise<GameRoomItem>;
  closeRoom(roomId: string, input?: ExpectedVersionRequest): Promise<GameRoomItem>;
}

let configuredRoomService: GamesRoomService | null = null;

export function configureGamesRoomService(service: GamesRoomService): void {
  configuredRoomService = service;
}

export function getGamesRoomService(): GamesRoomService {
  if (!configuredRoomService) {
    throw new Error(
      'Games room service is not configured. Bootstrap sdkwork-gameengine-pc runtime first.',
    );
  }
  return configuredRoomService;
}

function mapRoomPageData(page: SdkWorkPageData): GameRoomPage {
  return {
    items: page.items as unknown as GameRoomItem[],
    total: Number(page.pageInfo.totalItems ?? page.items.length),
    page: page.pageInfo.page ?? 1,
    pageSize: page.pageInfo.pageSize ?? 20,
  };
}

function mapSeatPageData(page: SdkWorkPageData): GameRoomSeatPage {
  return {
    items: page.items as unknown as GameRoomSeatItem[],
    total: Number(page.pageInfo.totalItems ?? page.items.length),
    page: page.pageInfo.page ?? 1,
    pageSize: page.pageInfo.pageSize ?? 200,
  };
}

function mapRoomItem(item: Record<string, unknown>): GameRoomItem {
  return item as unknown as GameRoomItem;
}

export function createGamesRoomService(client: SdkworkGameengineAppClient): GamesRoomService {
  return {
    async listRooms(params) {
      const page = await client.rooms.games.rooms.list(params);
      return mapRoomPageData(page);
    },
    async createRoom(input) {
      const item = await client.rooms.games.rooms.create(input);
      return mapRoomItem(item);
    },
    async retrieveRoom(roomId) {
      const item = await client.rooms.games.rooms.retrieve(roomId);
      return mapRoomItem(item);
    },
    async listRoomSeats(roomId) {
      const page = await client.rooms.games.rooms.seats.list(roomId);
      return mapSeatPageData(page);
    },
    async joinRoom(roomId, input = {}) {
      const item = await client.rooms.games.rooms.join(roomId, input);
      return mapRoomItem(item);
    },
    async leaveRoom(roomId, input = {}) {
      const item = await client.rooms.games.rooms.leave(roomId, input);
      return mapRoomItem(item);
    },
    async setRoomReady(roomId, input) {
      const item = await client.rooms.games.rooms.ready(roomId, input);
      return mapRoomItem(item);
    },
    async startRoom(roomId, input = {}) {
      const item = await client.rooms.games.rooms.start(roomId, input);
      return mapRoomItem(item);
    },
    async closeRoom(roomId, input = {}) {
      const item = await client.rooms.games.rooms.close(roomId, input);
      return mapRoomItem(item);
    },
  };
}
