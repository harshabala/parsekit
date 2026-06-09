/** Middle-truncate long paths for narrow popover UI. */
export function truncatePath(path: string, maxLen = 36): string {
  if (!path || path.length <= maxLen) return path;
  const keep = Math.floor((maxLen - 1) / 2);
  return `${path.slice(0, keep)}…${path.slice(-keep)}`;
}