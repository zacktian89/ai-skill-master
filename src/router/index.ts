import { createRouter, createWebHashHistory } from "vue-router";
import SkillsView from "../views/skills/SkillsView.vue";
import ProjectsView from "../views/projects/ProjectsView.vue";
import AgentsView from "../views/agents/AgentsView.vue";
import SettingsView from "../views/settings/SettingsView.vue";

const routes = [
  {
    path: "/",
    redirect: "/skills",
  },
  {
    path: "/skills",
    name: "skills",
    component: SkillsView,
  },
  {
    path: "/projects",
    name: "projects",
    component: ProjectsView,
  },
  {
    path: "/agents",
    name: "agents",
    component: AgentsView,
  },
  {
    path: "/plugins",
    name: "plugins",
    component: () => import("../views/plugins/PluginsView.vue"),
  },
  {
    path: "/settings",
    name: "settings",
    component: SettingsView,
  },
];

export const router = createRouter({
  history: createWebHashHistory(),
  routes,
});
