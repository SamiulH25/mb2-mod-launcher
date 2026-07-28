let dismissTimer: ReturnType<typeof setTimeout> | undefined;

export function scheduleStatusDismiss(
  clear: () => void,
  delayMs = 3500,
): void {
  if (dismissTimer !== undefined) {
    clearTimeout(dismissTimer);
  }
  dismissTimer = setTimeout(() => {
    clear();
    dismissTimer = undefined;
  }, delayMs);
}

export function cancelStatusDismiss(): void {
  if (dismissTimer !== undefined) {
    clearTimeout(dismissTimer);
    dismissTimer = undefined;
  }
}
