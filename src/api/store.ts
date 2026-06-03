import type { StoreLeaderboardType, StoreSkill } from "../types";
import { resolveInvoke } from "./client";
import * as mockSkills from "./mock/skills";

export async function fetchStoreLeaderboard(board: StoreLeaderboardType): Promise<StoreSkill[]> {
  const invoke = await resolveInvoke();
  return invoke
    ? invoke<StoreSkill[]>("fetch_store_leaderboard", { board })
    : mockSkills.fetchStoreLeaderboard(board);
}

export async function searchStoreSkills(query: string, limit = 60): Promise<StoreSkill[]> {
  const invoke = await resolveInvoke();
  return invoke
    ? invoke<StoreSkill[]>("search_store_skills", { request: { query, limit } })
    : mockSkills.searchStoreSkills(query, limit);
}
