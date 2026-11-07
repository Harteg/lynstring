import type { TranslationSession } from '$lib/stores/translation-history';

/**
 * Format a duration in milliseconds for display.
 * - Under 60s: "9.5s", "19.0s", "59.9s" (always 1 decimal)
 * - 60s+: with decimalsWhenMinutes: "1m20.5s"; without: "1m20s"
 */
export function formatDuration(ms: number, decimalsWhenMinutes = true): string {
  if (ms < 0 || !Number.isFinite(ms)) return '0.0s';
  const seconds = ms / 1000;
  const whole = Math.floor(seconds);
  const frac = Math.floor((seconds % 1) * 10); /* 0–9 only, avoid rounding to 10 */
  if (seconds < 60) {
    return `${whole}.${frac}s`;
  }
  const mins = Math.floor(whole / 60);
  const secs = whole % 60;
  if (decimalsWhenMinutes) {
    return `${mins}m${secs.toString().padStart(2, '0')}.${frac}s`;
  }
  return `${mins}m${secs.toString().padStart(2, '0')}s`;
}

/**
 * Format a timestamp as a relative time string (e.g., "2m ago", "Yesterday", "3 days ago")
 */
export function formatRelativeTime(timestamp: number): string {
  const now = Date.now();
  const diff = now - timestamp;
  const seconds = Math.floor(diff / 1000);
  const minutes = Math.floor(seconds / 60);
  const hours = Math.floor(minutes / 60);
  const days = Math.floor(hours / 24);

  if (days === 1) {
    return 'Yesterday';
  } else if (days > 1) {
    return `${days} days ago`;
  } else if (hours > 0) {
    return `${hours} hour${hours > 1 ? 's' : ''} ago`;
  } else if (minutes > 0) {
    return `${minutes} minute${minutes > 1 ? 's' : ''} ago`;
  } else if (seconds > 0) {
    return `${seconds} second${seconds > 1 ? 's' : ''} ago`;
  } else {
    return 'Just now';
  }
}

/**
 * Format a timestamp as a short relative time string (e.g., "6m ago", "2h ago")
 * Used for compact displays like the last translation card
 * Never shows seconds - shows "Less than a minute ago" for < 1 minute
 */
export function formatRelativeTimeShort(timestamp: number): string {
  const now = Date.now();
  const diff = now - timestamp;
  const minutes = Math.floor(diff / 60000);
  const hours = Math.floor(minutes / 60);
  const days = Math.floor(hours / 24);

  if (days === 1) {
    return 'Yesterday';
  } else if (days > 1) {
    return `${days} days ago`;
  } else if (hours > 0) {
    return `${hours} hour${hours > 1 ? 's' : ''} ago`;
  } else if (minutes > 0) {
    return `${minutes} minute${minutes > 1 ? 's' : ''} ago`;
  } else {
    return 'Less than a minute ago';
  }
}

/**
 * Get a summary string for a translation session (e.g., "2 strings → 3 languages")
 * For cancelled sessions with progress, includes progress info (e.g., "Cancelled. 132/1239 translations generated")
 */
export function getTranslationSummary(session: TranslationSession): string {
  const stringCount = session.sourceStrings.length;
  const langCount = session.targetLanguages.length;
  
  // For cancelled sessions with progress, show progress badge
  if (session.status === 'cancelled' && session.progress.completedStrings > 0 && session.progress.totalStrings > 0) {
    const baseSummary = session.mode === 'single_string_multi_language'
      ? `1 string → ${langCount} language${langCount > 1 ? 's' : ''}`
      : `${stringCount} string${stringCount > 1 ? 's' : ''} → ${langCount} language${langCount > 1 ? 's' : ''}`;
    return `${baseSummary} • Cancelled. ${session.progress.completedStrings}/${session.progress.totalStrings} translations generated`;
  }
  
  if (session.mode === 'single_string_multi_language') {
    return `1 string → ${langCount} language${langCount > 1 ? 's' : ''}`;
  } else {
    return `${stringCount} string${stringCount > 1 ? 's' : ''} → ${langCount} language${langCount > 1 ? 's' : ''}`;
  }
}

