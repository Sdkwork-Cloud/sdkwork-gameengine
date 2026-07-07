export interface CreateRoomRequest {
  gameId: string;
  modeId?: string;
  rulesetId?: string;
  roomCode: string;
  visibility?: 'public' | 'private';
  joinPolicy?: 'open' | 'invite' | 'password';
  maxPlayers: number;
}
