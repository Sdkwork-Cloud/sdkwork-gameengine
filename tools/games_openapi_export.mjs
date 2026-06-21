#!/usr/bin/env node
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { isBlank } from '@sdkwork/utils';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const outputDir = path.join(root, 'generated/openapi');

const OWNER = 'sdkwork-games';
const DOMAIN = 'game';

const sharedComponents = {
  securitySchemes: {
    AuthToken: {
      type: 'http',
      scheme: 'bearer',
      bearerFormat: 'SDKWork-Auth-Token',
      description: 'SDKWork dual-token auth principal (Authorization bearer).',
    },
    AccessToken: {
      type: 'apiKey',
      in: 'header',
      name: 'Access-Token',
      description: 'SDKWork dual-token access credential header.',
    },
  },
  parameters: {
    GameIdPath: {
      name: 'gameId',
      in: 'path',
      required: true,
      schema: { type: 'string' },
    },
    PageQuery: {
      name: 'page',
      in: 'query',
      required: false,
      schema: { type: 'integer', minimum: 1, default: 1 },
    },
    PageSizeQuery: {
      name: 'page_size',
      in: 'query',
      required: false,
      schema: { type: 'integer', minimum: 1, maximum: 200, default: 20 },
    },
    StatusQuery: {
      name: 'status',
      in: 'query',
      required: false,
      schema: { type: 'string' },
    },
  },
  responses: {
    ProblemDetailResponse: {
      description: 'RFC 9457 problem details.',
      content: {
        'application/problem+json': {
          schema: { $ref: '#/components/schemas/ProblemDetail' },
        },
      },
    },
  },
  schemas: {
    ProblemDetail: {
      type: 'object',
      additionalProperties: true,
      required: ['type', 'title', 'status'],
      properties: {
        type: { type: 'string', format: 'uri' },
        title: { type: 'string' },
        status: { type: 'integer' },
        detail: { type: 'string' },
        instance: { type: 'string' },
      },
    },
    GamesApiResult: {
      type: 'object',
      additionalProperties: false,
      required: ['code', 'message', 'data'],
      properties: {
        code: { type: 'string' },
        message: { type: 'string' },
        data: {},
      },
    },
    GamesHealthResponse: {
      type: 'object',
      additionalProperties: false,
      required: ['status', 'service'],
      properties: {
        status: { type: 'string' },
        service: { type: 'string' },
      },
    },
    GameCatalogItem: {
      type: 'object',
      additionalProperties: false,
      required: ['id', 'gameCode', 'title', 'status'],
      properties: {
        id: { type: 'string' },
        gameCode: { type: 'string' },
        title: { type: 'string' },
        summary: { type: 'string' },
        genre: { type: 'string' },
        status: { type: 'string' },
      },
    },
    GameCatalogPage: {
      type: 'object',
      additionalProperties: false,
      required: ['items', 'total', 'page', 'pageSize'],
      properties: {
        items: {
          type: 'array',
          items: { $ref: '#/components/schemas/GameCatalogItem' },
        },
        total: { type: 'integer', minimum: 0 },
        page: { type: 'integer', minimum: 1 },
        pageSize: { type: 'integer', minimum: 1 },
      },
    },
  },
};

const dualTokenSecurity = [{ AuthToken: [], AccessToken: [] }];

function operation(
  operationId,
  tags,
  apiSurface,
  options = {},
) {
  const { publicRoute = false, parameters = [], responseSchema = 'GamesApiResult' } = options;
  return {
    operationId,
    tags,
    'x-sdkwork-request-context': 'WebRequestContext',
    'x-sdkwork-api-surface': apiSurface,
    'x-sdkwork-owner': OWNER,
    'x-sdkwork-domain': DOMAIN,
    ...(publicRoute ? { security: [] } : { security: dualTokenSecurity }),
    ...(parameters.length > 0 ? { parameters } : {}),
    responses: {
      200: {
        description: 'OK',
        content: {
          'application/json': {
            schema: { $ref: `#/components/schemas/${responseSchema}` },
          },
        },
      },
      default: { $ref: '#/components/responses/ProblemDetailResponse' },
    },
  };
}

function buildOpenApi(title, serverUrl, operations) {
  return {
    openapi: '3.1.2',
    jsonSchemaDialect: 'https://json-schema.org/draft/2020-12/schema',
    info: {
      title,
      version: '0.1.0',
      description: 'SDKWork games platform HTTP contract.',
      'x-sdkwork-owner': OWNER,
      'x-sdkwork-domain': DOMAIN,
    },
    servers: [{ url: serverUrl, description: 'SDKWork API root' }],
    paths: operations,
    components: sharedComponents,
  };
}

const appOperations = {
  '/app/v3/api/system/health': {
    get: operation('games.health.check', ['health'], 'app-api', {
      publicRoute: true,
      responseSchema: 'GamesHealthResponse',
    }),
  },
  '/app/v3/api/system/ready': {
    get: operation('games.ready.check', ['health'], 'app-api', {
      publicRoute: true,
      responseSchema: 'GamesHealthResponse',
    }),
  },
  '/app/v3/api/games': {
    get: operation('games.catalog.list', ['games'], 'app-api', {
      parameters: [
        { $ref: '#/components/parameters/PageQuery' },
        { $ref: '#/components/parameters/PageSizeQuery' },
        { $ref: '#/components/parameters/StatusQuery' },
      ],
    }),
  },
  '/app/v3/api/games/{gameId}': {
    get: operation('games.catalog.retrieve', ['games'], 'app-api', {
      parameters: [{ $ref: '#/components/parameters/GameIdPath' }],
    }),
  },
};

const backendOperations = {
  '/backend/v3/api/games': {
    get: operation('backend.games.catalog.list', ['games'], 'backend-api', {
      parameters: [
        { $ref: '#/components/parameters/PageQuery' },
        { $ref: '#/components/parameters/PageSizeQuery' },
        { $ref: '#/components/parameters/StatusQuery' },
      ],
    }),
  },
};

if (isBlank(OWNER)) {
  throw new Error('owner must be non-empty');
}

fs.mkdirSync(path.join(root, 'apis/app-api/game'), { recursive: true });
fs.mkdirSync(path.join(root, 'apis/backend-api/game'), { recursive: true });
fs.mkdirSync(outputDir, { recursive: true });

const appDoc = buildOpenApi('SDKWork Games App API', '/app/v3/api', appOperations);
const backendDoc = buildOpenApi('SDKWork Games Backend API', '/backend/v3/api', backendOperations);

const targets = [
  ['apis/app-api/game/games-app-api.openapi.json', appDoc],
  ['apis/backend-api/game/games-backend-api.openapi.json', backendDoc],
  ['generated/openapi/games-app-api.openapi.json', appDoc],
  ['generated/openapi/games-backend-api.openapi.json', backendDoc],
];

for (const [relativePath, doc] of targets) {
  fs.writeFileSync(path.join(root, relativePath), `${JSON.stringify(doc, null, 2)}\n`);
}

process.stdout.write('[games-openapi] exported L2 app and backend OpenAPI documents\n');
