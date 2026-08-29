/**
 * 只重排目前可見的釘選專案，保留暫時不可見 key 原本所在的槽位。
 */
export function rebuildPinnedProjectOrder(
  currentKeys: string[],
  currentVisibleKeys: string[],
  nextVisibleKeys: string[],
): string[] {
  const visibleKeys = new Set(currentVisibleKeys);
  const nextKeys = new Set(nextVisibleKeys);

  if (
    visibleKeys.size !== currentVisibleKeys.length ||
    nextKeys.size !== nextVisibleKeys.length ||
    nextVisibleKeys.length !== currentVisibleKeys.length ||
    nextVisibleKeys.some((key) => !visibleKeys.has(key))
  ) {
    return currentKeys;
  }

  let nextVisibleIndex = 0;
  return currentKeys.map((key) => {
    if (!visibleKeys.has(key)) return key;
    return nextVisibleKeys[nextVisibleIndex++];
  });
}
