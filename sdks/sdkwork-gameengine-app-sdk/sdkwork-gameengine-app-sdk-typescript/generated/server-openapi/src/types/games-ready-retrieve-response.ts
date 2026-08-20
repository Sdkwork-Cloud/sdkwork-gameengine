import type { GamesHealthResponse } from './games-health-response';

export interface GamesReadyRetrieveResponse {
  code: 0;
  data: unknown & { item: GamesHealthResponse; };
  /** Server-owned request correlation id. */
  traceId: string;
}
