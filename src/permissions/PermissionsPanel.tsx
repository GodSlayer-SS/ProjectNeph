/**
 * src/permissions/PermissionsPanel.tsx
 *
 * Blueprint §4: apps/desktop/src/permissions/ — capability prompts.
 *
 * This component handles the yellow/red confirmation UI for tool executions.
 * Phase 1: Inline confirm-card and confirm-modal (currently in PalettePanel).
 * Phase 2: Full permissions flow with domain context, audit trail, and
 *          capability token display (shows the user exactly what Nephis can do).
 *
 * Currently re-exports the confirmation UI types so other panels can import
 * the PendingConfirmation shape from one place.
 */

export type { PendingConfirmation } from "../state/paletteStore";

/**
 * Permission capability labels — maps domain + capability to a human-readable string.
 * Used in Phase 2 to explain to the user what the action will do.
 */
export const CAPABILITY_LABELS: Record<string, string> = {
  "workspace:write": "Write files to your Nephis workspace",
  "personal:write": "Modify files in your Documents / Downloads / Desktop",
  "personal:delete": "Delete files (moves to Recycle Bin)",
  "system:read": "Read system information",
  "browser-research:navigate": "Open URLs in isolated research browser",
  "browser-personal:navigate": "Open URLs in YOUR personal browser (logged-in sessions)",
  "shell-safe:exec": "Run built-in text utilities (no subprocess)",
  "shell-sandboxed:exec": "Run code in a sandboxed subprocess",
  "shell-native:exec": "Run a native system command (full access)",
  "network:egress": "Make an outbound HTTP request",
};
