import type { GameCatalogItem } from './game-catalog-item';

export interface GameCatalogPage {
  items: GameCatalogItem[];
  total: number;
  page: number;
  pageSize: number;
}
