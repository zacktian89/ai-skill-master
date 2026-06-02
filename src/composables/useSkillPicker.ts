import { ref, computed } from "vue";
import type { Skill } from "../types";

export function useSkillPicker(getSkills: () => Skill[]) {
  const addSkillQuery = ref("");
  const selectedSkillIds = ref<string[]>([]);

  const filteredLibrarySkills = computed(() => {
    const normalized = addSkillQuery.value.trim().toLowerCase();
    const allSkills = getSkills();
    if (!normalized) return allSkills;
    return allSkills.filter(
      (skill) =>
        skill.name.toLowerCase().includes(normalized) ||
        skill.id.toLowerCase().includes(normalized) ||
        (skill.description && skill.description.toLowerCase().includes(normalized))
    );
  });

  function toggleAllLibrarySkills(checked: boolean) {
    if (checked) {
      selectedSkillIds.value = filteredLibrarySkills.value.map((s) => s.id);
    } else {
      selectedSkillIds.value = [];
    }
  }

  function resetPicker() {
    addSkillQuery.value = "";
    selectedSkillIds.value = [];
  }

  return {
    addSkillQuery,
    selectedSkillIds,
    filteredLibrarySkills,
    toggleAllLibrarySkills,
    resetPicker,
  };
}
