import { ref } from "vue";

export function useAsyncAction(options?: {
  onError?: (err: unknown) => void;
}) {
  const busy = ref(false);

  async function run<T>(
    action: () => Promise<T>,
    onSuccess?: (res: T) => void | Promise<void>,
    onError?: (err: unknown) => void
  ): Promise<T | undefined> {
    busy.value = true;
    try {
      const res = await action();
      if (onSuccess) await onSuccess(res);
      return res;
    } catch (err) {
      if (onError) {
        onError(err);
      } else if (options?.onError) {
        options.onError(err);
      }
    } finally {
      busy.value = false;
    }
  }

  return {
    busy,
    run,
  };
}
