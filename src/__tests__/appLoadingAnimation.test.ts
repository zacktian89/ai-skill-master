/**
 * @vitest-environment jsdom
 */
import { mount } from "@vue/test-utils";
import { describe, expect, it } from "vitest";
import AppLoadingAnimation from "../components/AppLoadingAnimation.vue";

describe("AppLoadingAnimation", () => {
  it("renders a full list skeleton placeholder", () => {
    const wrapper = mount(AppLoadingAnimation, {
      props: {
        rows: 6,
        variant: "panel",
      },
    });

    expect(wrapper.find("[data-testid='app-loading-animation']").exists()).toBe(true);
    expect(wrapper.findAll("[data-testid='loading-skeleton-row']")).toHaveLength(6);
    expect(wrapper.find(".app-loading-animation__row-title").exists()).toBe(true);
    expect(wrapper.find(".app-loading-animation__row-meta").exists()).toBe(true);
  });
});
