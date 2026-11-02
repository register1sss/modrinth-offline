// Only packs that can be installed are listed: the catalog leaves out packs incompatible with
// 26.3, too large, or local archives that were never imported.
export function listedPacks(data) {
  return (data?.entries ?? []).filter((entry) => entry?.available === true && !entry.local)
}
