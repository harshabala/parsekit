/**
 * Simple dialog focus trap: autofocus primary control, Tab cycles inside root.
 * Restore previous focus via `restoreFocusOnDestroy` or handle at the opener.
 */

const FOCUSABLE_SELECTOR = [
  "a[href]",
  "button:not([disabled])",
  "input:not([disabled]):not([type='hidden'])",
  "select:not([disabled])",
  "textarea:not([disabled])",
  "[tabindex]:not([tabindex='-1'])",
].join(", ");

export type FocusTrapOptions = {
  /** CSS selector for initial focus (default: .settings-back-btn) */
  initialSelector?: string;
  /** When true (default), restore previously focused element on destroy if still in DOM */
  restoreFocus?: boolean;
};

function isVisible(el: HTMLElement): boolean {
  return !!(el.offsetWidth || el.offsetHeight || el.getClientRects().length);
}

export function getFocusableElements(root: HTMLElement): HTMLElement[] {
  return Array.from(root.querySelectorAll<HTMLElement>(FOCUSABLE_SELECTOR)).filter(
    (el) => !el.hasAttribute("disabled") && el.tabIndex !== -1 && isVisible(el),
  );
}

/**
 * Svelte action: `use:focusTrap` or `use:focusTrap={{ restoreFocus: false }}`.
 */
export function focusTrap(node: HTMLElement, options: FocusTrapOptions = {}) {
  let opts = options;
  const previouslyFocused =
    document.activeElement instanceof HTMLElement ? document.activeElement : null;

  function focusInitial() {
    const selector = opts.initialSelector ?? ".settings-back-btn";
    const preferred = node.querySelector<HTMLElement>(selector);
    const target = preferred && isVisible(preferred) ? preferred : getFocusableElements(node)[0];
    target?.focus({ preventScroll: true });
  }

  function onKeydown(e: KeyboardEvent) {
    if (e.key !== "Tab") return;

    const focusable = getFocusableElements(node);
    if (focusable.length === 0) {
      e.preventDefault();
      return;
    }

    const first = focusable[0];
    const last = focusable[focusable.length - 1];
    const active = document.activeElement;

    if (e.shiftKey) {
      if (active === first || !node.contains(active)) {
        e.preventDefault();
        last.focus();
      }
    } else if (active === last || !node.contains(active)) {
      e.preventDefault();
      first.focus();
    }
  }

  const frame = requestAnimationFrame(() => {
    focusInitial();
  });
  node.addEventListener("keydown", onKeydown);

  return {
    update(next: FocusTrapOptions = {}) {
      opts = next;
    },
    destroy() {
      cancelAnimationFrame(frame);
      node.removeEventListener("keydown", onKeydown);
      const shouldRestore = opts.restoreFocus !== false;
      if (
        shouldRestore &&
        previouslyFocused &&
        document.contains(previouslyFocused) &&
        typeof previouslyFocused.focus === "function"
      ) {
        previouslyFocused.focus({ preventScroll: true });
      }
    },
  };
}
