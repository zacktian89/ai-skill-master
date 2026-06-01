import { nextTick, onBeforeUnmount, onMounted, ref, watch, type WatchSource } from "vue";

export function useScrollableList(source: WatchSource<unknown>) {
  const listStackRef = ref<HTMLElement | null>(null);
  const listStackScrollable = ref(false);
  let resizeObserver: ResizeObserver | null = null;

  function updateListStackScrollable() {
    const el = listStackRef.value;
    listStackScrollable.value = Boolean(el && el.scrollHeight > el.clientHeight + 1);
  }

  function observeListStack() {
    resizeObserver?.disconnect();
    if (listStackRef.value && typeof ResizeObserver !== "undefined") {
      resizeObserver = new ResizeObserver(updateListStackScrollable);
      resizeObserver.observe(listStackRef.value);
    }
    updateListStackScrollable();
  }

  onMounted(() => {
    observeListStack();
    window.addEventListener("resize", updateListStackScrollable);
  });

  onBeforeUnmount(() => {
    resizeObserver?.disconnect();
    window.removeEventListener("resize", updateListStackScrollable);
  });

  watch(source, async () => {
    await nextTick();
    observeListStack();
  });

  return {
    listStackRef,
    listStackScrollable,
  };
}
