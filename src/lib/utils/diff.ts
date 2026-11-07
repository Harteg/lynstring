/**
 * Diff segment types
 */
export type DiffSegment = 
  | { type: 'equal'; text: string }
  | { type: 'delete'; text: string }
  | { type: 'insert'; text: string };

/**
 * Compute word-based diff between two strings
 * Returns an array of segments representing the differences
 * Uses LCS (Longest Common Subsequence) algorithm
 */
export function computeDiff(oldText: string, newText: string): DiffSegment[] {
  // If texts are identical, return single equal segment
  if (oldText === newText) {
    return [{ type: 'equal', text: oldText }];
  }
  
  // Split into words (preserving whitespace)
  const oldWords = splitIntoWords(oldText);
  const newWords = splitIntoWords(newText);
  
  // Compute LCS to find matching parts
  const lcs = computeLCS(oldWords, newWords);
  
  // Build diff segments by aligning to LCS
  const segments: DiffSegment[] = [];
  let oldIdx = 0;
  let newIdx = 0;
  let lcsIdx = 0;
  
  while (oldIdx < oldWords.length || newIdx < newWords.length) {
    // Collect deletions (words in old but not in current LCS position)
    const deletions: string[] = [];
    while (oldIdx < oldWords.length && (lcsIdx >= lcs.length || oldWords[oldIdx] !== lcs[lcsIdx])) {
      deletions.push(oldWords[oldIdx]);
      oldIdx++;
    }
    
    // Collect insertions (words in new but not in current LCS position)
    const insertions: string[] = [];
    while (newIdx < newWords.length && (lcsIdx >= lcs.length || newWords[newIdx] !== lcs[lcsIdx])) {
      insertions.push(newWords[newIdx]);
      newIdx++;
    }
    
    // Add deletions and insertions
    if (deletions.length > 0) {
      segments.push({ type: 'delete', text: deletions.join('') });
    }
    if (insertions.length > 0) {
      segments.push({ type: 'insert', text: insertions.join('') });
    }
    
    // Add equal segment if we have a match
    if (lcsIdx < lcs.length && oldIdx < oldWords.length && newIdx < newWords.length &&
        oldWords[oldIdx] === lcs[lcsIdx] && newWords[newIdx] === lcs[lcsIdx]) {
      segments.push({ type: 'equal', text: lcs[lcsIdx] });
      oldIdx++;
      newIdx++;
      lcsIdx++;
    } else if (lcsIdx >= lcs.length) {
      // No more LCS matches, add remaining as deletions/insertions
      if (oldIdx < oldWords.length) {
        segments.push({ type: 'delete', text: oldWords.slice(oldIdx).join('') });
      }
      if (newIdx < newWords.length) {
        segments.push({ type: 'insert', text: newWords.slice(newIdx).join('') });
      }
      break;
    }
  }
  
  // Merge adjacent segments of the same type
  return mergeSegments(segments);
}

/**
 * Split text into words while preserving whitespace
 */
function splitIntoWords(text: string): string[] {
  const words: string[] = [];
  let currentWord = '';
  
  for (let i = 0; i < text.length; i++) {
    const char = text[i];
    const isWhitespace = /\s/.test(char);
    
    if (isWhitespace) {
      if (currentWord) {
        words.push(currentWord);
        currentWord = '';
      }
      words.push(char);
    } else {
      currentWord += char;
    }
  }
  
  if (currentWord) {
    words.push(currentWord);
  }
  
  return words;
}

/**
 * Compute Longest Common Subsequence (LCS) between two arrays
 */
function computeLCS<T>(arr1: T[], arr2: T[]): T[] {
  const m = arr1.length;
  const n = arr2.length;
  
  // DP table: dp[i][j] = length of LCS of arr1[0..i-1] and arr2[0..j-1]
  const dp: number[][] = Array(m + 1)
    .fill(null)
    .map(() => Array(n + 1).fill(0));
  
  // Fill DP table
  for (let i = 1; i <= m; i++) {
    for (let j = 1; j <= n; j++) {
      if (arr1[i - 1] === arr2[j - 1]) {
        dp[i][j] = dp[i - 1][j - 1] + 1;
      } else {
        dp[i][j] = Math.max(dp[i - 1][j], dp[i][j - 1]);
      }
    }
  }
  
  // Reconstruct LCS
  const lcs: T[] = [];
  let i = m;
  let j = n;
  
  while (i > 0 && j > 0) {
    if (arr1[i - 1] === arr2[j - 1]) {
      lcs.unshift(arr1[i - 1]);
      i--;
      j--;
    } else if (dp[i - 1][j] > dp[i][j - 1]) {
      i--;
    } else {
      j--;
    }
  }
  
  return lcs;
}

/**
 * Merge adjacent segments of the same type
 */
function mergeSegments(segments: DiffSegment[]): DiffSegment[] {
  if (segments.length === 0) return [];
  
  const merged: DiffSegment[] = [];
  let current = segments[0];
  
  for (let i = 1; i < segments.length; i++) {
    const next = segments[i];
    
    if (current.type === next.type) {
      // Merge segments of the same type
      current = { ...current, text: current.text + next.text };
    } else {
      // Push current and start new segment
      merged.push(current);
      current = next;
    }
  }
  
  merged.push(current);
  return merged;
}

/**
 * Compute separate diffs for old and new text
 * This is useful for side-by-side display
 */
export function computeSeparateDiffs(oldText: string, newText: string): {
  oldSegments: DiffSegment[];
  newSegments: DiffSegment[];
} {
  const diff = computeDiff(oldText, newText);
  
  const oldSegments: DiffSegment[] = [];
  const newSegments: DiffSegment[] = [];
  
  for (const segment of diff) {
    if (segment.type === 'equal') {
      oldSegments.push(segment);
      newSegments.push(segment);
    } else if (segment.type === 'delete') {
      oldSegments.push(segment);
    } else if (segment.type === 'insert') {
      newSegments.push(segment);
    }
  }
  
  return { oldSegments, newSegments };
}

