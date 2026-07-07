import type { GamesHealthResponse } from './games-health-response';

export interface GamesHealthRetrieveResponse {
  code: 0;
  data: unknown & Record<string, unknown>;
  /** Server-owned request correlation id. */
  traceId: string;
}
