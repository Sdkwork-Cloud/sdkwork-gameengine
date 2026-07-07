import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../..');

test('pc core room service exposes generated app SDK room lifecycle facade', () => {
  const sourcePath = path.join(
    root,
    'apps/sdkwork-gameengine-pc/packages/sdkwork-gameengine-pc-core/src/roomService.ts',
  );
  const source = fs.readFileSync(sourcePath, 'utf8');

  for (const method of [
    'listRooms',
    'createRoom',
    'retrieveRoom',
    'listRoomSeats',
    'joinRoom',
    'leaveRoom',
    'setRoomReady',
    'startRoom',
    'closeRoom',
  ]) {
    assert.match(source, new RegExp(`\\b${method}\\b`), `missing ${method} facade method`);
  }

  assert.match(source, /client\.rooms\.games\.rooms\.create\(/);
  assert.match(source, /client\.rooms\.games\.rooms\.retrieve\(/);
  assert.match(source, /client\.rooms\.games\.rooms\.seats\.list\(/);
  assert.match(source, /client\.rooms\.games\.rooms\.join\(/);
  assert.match(source, /client\.rooms\.games\.rooms\.leave\(/);
  assert.match(source, /client\.rooms\.games\.rooms\.ready\(/);
  assert.match(source, /client\.rooms\.games\.rooms\.start\(/);
  assert.match(source, /client\.rooms\.games\.rooms\.close\(/);
  assert.doesNotMatch(source, /\bfetch\s*\(/);
  assert.doesNotMatch(source, /\baxios\b/);
});
