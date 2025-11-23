/**
 * History Manager for Undo/Redo Functionality
 *
 * ## Why This Exists
 *
 * The HL7 message editor needs undo/redo that works across multiple edit sources:
 * raw text typing, form field changes, wizards, and segment additions. A centralised
 * history manager ensures consistent behaviour regardless of how the message is modified.
 *
 * ## Design Decision: Function vs Effect
 *
 * We use an explicit `updateMessage()` function pattern rather than a Svelte `$effect`
 * watching the message variable. This is intentional because:
 *
 * 1. **Undo/redo would cause infinite loops** - An effect would fire when restoring
 *    from history, pushing the restored state back onto the stack.
 *
 * 2. **Different edit types need different handling** - Typing should coalesce into
 *    single undo entries, while wizard changes should be discrete. An effect can't
 *    distinguish between these.
 *
 * 3. **File operations need to skip history** - Opening a file or creating a new
 *    message should clear history, not record the change.
 *
 * ## Coalescing Behavior
 *
 * When `coalesce: true` is passed, rapid changes within 500ms are merged into a
 * single history entry. This prevents each keystroke from being a separate undo step.
 * The saved state is from BEFORE the typing session started, so undoing restores
 * to the pre-typing state.
 *
 * Coalescing is used for:
 * - Regular keyboard typing in the message editor
 *
 * Discrete entries (coalesce: false) are created for:
 * - Paste operations
 * - Form field changes (segment tabs)
 * - Wizard applications
 * - Adding new segments
 *
 * ## Usage
 *
 * ```typescript
 * const history = createHistoryManager();
 *
 * // In updateMessage function (called before changing message state):
 * history.push(currentMessage, coalesce);
 * message = newMessage;
 *
 * // Undo (pass current state, receive previous state):
 * const previous = history.undo(message);
 * if (previous !== null) message = previous;
 *
 * // Redo (pass current state, receive next state):
 * const next = history.redo(message);
 * if (next !== null) message = next;
 *
 * // Clear on file new/open:
 * history.clear();
 * ```
 *
 * ## Reactive State
 *
 * `canUndo` and `canRedo` are Svelte 5 derived values that automatically update
 * when the stacks change, enabling reactive UI binding for toolbar buttons and
 * menu item states.
 */

const COALESCE_DELAY_MS = 500;

export function createHistoryManager() {
  let undoStack: string[] = $state([]);
  let redoStack: string[] = $state([]);
  let coalesceTimer: ReturnType<typeof setTimeout> | null = null;
  let pendingState: string | null = null;

  const canUndo = $derived(undoStack.length > 0);
  const canRedo = $derived(redoStack.length > 0);

  /**
   * Flush any pending coalesced state to the undo stack.
   * Called when a non-coalesced change occurs or before undo/redo.
   */
  function flushPending() {
    if (coalesceTimer !== null) {
      clearTimeout(coalesceTimer);
      coalesceTimer = null;
    }
    if (pendingState !== null) {
      undoStack.push(pendingState);
      pendingState = null;
    }
  }

  /**
   * Push the current message state to history before making a change.
   *
   * @param currentMessage - The current message state (before the change)
   * @param coalesce - If true, rapid calls within COALESCE_DELAY_MS will be merged
   *                   into a single history entry. Use for typing; false for discrete changes.
   */
  function push(currentMessage: string, coalesce: boolean = false) {
    // Clear redo stack on any new change
    redoStack = [];

    if (coalesce) {
      // For coalesced changes, we only want to save the state from BEFORE
      // the typing session started. If there's no pending state, this is
      // the start of a new typing session.
      if (pendingState === null) {
        pendingState = currentMessage;
      }

      // Reset the timer - if user keeps typing, we keep delaying
      if (coalesceTimer !== null) {
        clearTimeout(coalesceTimer);
      }

      coalesceTimer = setTimeout(() => {
        flushPending();
      }, COALESCE_DELAY_MS);
    } else {
      // Non-coalesced change: flush any pending state first, then push current
      flushPending();
      undoStack.push(currentMessage);
    }
  }

  /**
   * Undo the last change.
   *
   * @param currentMessage - The current message state (will be pushed to redo stack)
   * @returns The previous message state to restore, or null if nothing to undo
   */
  function undo(currentMessage: string): string | null {
    // Flush any pending coalesced state before undoing
    flushPending();

    if (undoStack.length === 0) {
      return null;
    }

    const previous = undoStack.pop()!;
    redoStack.push(currentMessage);
    return previous;
  }

  /**
   * Redo a previously undone change.
   *
   * @param currentMessage - The current message state (will be pushed to undo stack)
   * @returns The next message state to restore, or null if nothing to redo
   */
  function redo(currentMessage: string): string | null {
    if (redoStack.length === 0) {
      return null;
    }

    const next = redoStack.pop()!;
    undoStack.push(currentMessage);
    return next;
  }

  /**
   * Clear all history. Call when opening a new file or creating a new message.
   */
  function clear() {
    if (coalesceTimer !== null) {
      clearTimeout(coalesceTimer);
      coalesceTimer = null;
    }
    pendingState = null;
    undoStack = [];
    redoStack = [];
  }

  return {
    get canUndo() {
      return canUndo;
    },
    get canRedo() {
      return canRedo;
    },
    push,
    undo,
    redo,
    clear,
  };
}

export type HistoryManager = ReturnType<typeof createHistoryManager>;
