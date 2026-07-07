export interface GameRoomItem {
  id: string;
  gameId: string;
  modeId?: string;
  rulesetId?: string;
  roomCode: string;
  hostUserId: string;
  visibility: 'public' | 'private';
  joinPolicy: 'open' | 'invite' | 'password';
  maxPlayers: number;
  currentPlayers: number;
  status: 'open' | 'in_progress' | 'closed';
  version: string;
}
