import { ref, provide, inject, type InjectionKey, type Ref, watch } from "vue";
import type { AppSnapshot } from "../types";

export interface SelectionStore {
  selectedSkillId: Ref<string | null>;
  selectedProjectId: Ref<string | null>;
  selectedAgentId: Ref<string | null>;
  setSelectedSkillId: (id: string | null) => void;
  setSelectedProjectId: (id: string | null) => void;
  setSelectedAgentId: (id: string | null) => void;
}

export const SelectionStoreKey: InjectionKey<SelectionStore> = Symbol("SelectionStore");

export function createSelectionStore(snapshotRef: Ref<AppSnapshot | null>) {
  const selectedSkillId = ref<string | null>(null);
  const selectedProjectId = ref<string | null>(null);
  const selectedAgentId = ref<string | null>(null);

  function setSelectedSkillId(id: string | null) {
    selectedSkillId.value = id;
  }
  function setSelectedProjectId(id: string | null) {
    selectedProjectId.value = id;
  }
  function setSelectedAgentId(id: string | null) {
    selectedAgentId.value = id;
  }

  // Watch snapshot updates to sync selections
  watch(
    snapshotRef,
    (next) => {
      if (!next) return;
      if (!selectedSkillId.value || !next.state.skills.some((skill) => skill.id === selectedSkillId.value)) {
        selectedSkillId.value = next.state.skills[0]?.id ?? null;
      }
      if (!selectedProjectId.value || !next.state.projects.some((project) => project.id === selectedProjectId.value)) {
        selectedProjectId.value = next.state.currentProjectId ?? next.state.projects[0]?.id ?? null;
      }
      if (!selectedAgentId.value || !next.state.agents?.some((agent) => agent.id === selectedAgentId.value)) {
        selectedAgentId.value = next.state.agents?.[0]?.id ?? null;
      }
    },
    { immediate: true }
  );

  const store: SelectionStore = {
    selectedSkillId,
    selectedProjectId,
    selectedAgentId,
    setSelectedSkillId,
    setSelectedProjectId,
    setSelectedAgentId,
  };

  provide(SelectionStoreKey, store);
  return store;
}

export function useSelectionStore() {
  const store = inject(SelectionStoreKey);
  if (!store) {
    throw new Error("useSelectionStore must be used after createSelectionStore is called");
  }
  return store;
}
