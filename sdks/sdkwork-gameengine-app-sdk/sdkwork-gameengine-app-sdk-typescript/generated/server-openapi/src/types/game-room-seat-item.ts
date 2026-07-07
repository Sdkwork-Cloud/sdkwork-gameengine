export interface GameRoomSeatItem {
  id: string;
  roomId: string;
  seatNo: number;
  teamNo?: number;
  userId?: string;
  displayNameSnapshot?: string;
  status: 'empty' | 'reserved' | 'joined' | 'ready' | 'playing' | 'left';
  version: string;
}
