import assert from 'node:assert/strict';
import test from 'node:test';

import { createGamesLeaderboardService } from './leaderboardService.ts';

function createClientThatRejects(error) {
  return {
    leaderboard: {
      games: {
        leaderboard: {
          list: async () => {
            throw new Error('list should not be called');
          },
          me: {
            retrieve: async () => {
              throw error;
            },
          },
        },
      },
    },
  };
}

test('retrieveMyRanking returns null only for SDKWork not found errors', async () => {
  const service = createGamesLeaderboardService(
    createClientThatRejects({
      status: 404,
      code: 40401,
      traceId: 'trace-not-found',
    }),
  );

  await assert.doesNotReject(async () => {
    const ranking = await service.retrieveMyRanking({ gameId: 'game-1' });
    assert.equal(ranking, null);
  });
});

test('retrieveMyRanking rethrows non-not-found SDK errors', async () => {
  const error = {
    status: 500,
    code: 50001,
    traceId: 'trace-internal-error',
  };
  const service = createGamesLeaderboardService(createClientThatRejects(error));

  await assert.rejects(
    () => service.retrieveMyRanking({ gameId: 'game-1' }),
    (actual) => actual === error,
  );
});

test('retrieveMyRanking rethrows network and unknown errors', async () => {
  const error = new Error('network unavailable');
  const service = createGamesLeaderboardService(createClientThatRejects(error));

  await assert.rejects(
    () => service.retrieveMyRanking({ gameId: 'game-1' }),
    (actual) => actual === error,
  );
});
