export function enumLookup(source) {
  const byType = new Map();
  const bySymbol = new Map();
  const maps = source.matchAll(/s_keys_map_([A-Za-z0-9_]+)\s*(?:=\s*)?\{([\s\S]*?)\n\s*\};\s*CONFIG_OPTION_ENUM_DEFINE_STATIC_MAPS\(\1\)/g);
  for (const match of maps) {
    const values = new Map();
    for (const pair of match[2].matchAll(/\{\s*"([^"]+)"\s*,\s*([^}\n]+)\}/g)) {
      const { symbol } = enumSymbol(pair[2]);
      if (!symbol) throw new Error(`missing enum symbol in ${pair[0]}`);
      values.set(symbol, pair[1]);
      const candidates = bySymbol.get(symbol) ?? new Set();
      candidates.add(pair[1]);
      bySymbol.set(symbol, candidates);
    }
    byType.set(match[1], values);
  }
  if (!byType.has("InputShaperType")) throw new Error("missing InputShaperType enum map");
  return { byType, bySymbol };
}

export function evaluateEnumDefault(expression, lookup) {
  const value = expression.replace(/\s+/g, " ");
  const declaredType = /ConfigOptionEnum<\s*([^>]+)>/.exec(value)?.[1].trim();
  const body = declaredType
    ? /ConfigOptionEnum<[^>]+>\s*\(([\s\S]*)\)\s*$/.exec(value)?.[1]
    : /ConfigOptionEnumsGeneric(?:Nullable)?\s*(?:\(\s*\{([\s\S]*)\}\s*\)|\{([\s\S]*)\})\s*$/.exec(value)?.slice(1).find(Boolean);
  if (body === undefined) throw new Error(`invalid enum default expression ${value}`);
  return body
    .split(",")
    .map(raw => resolveEnum(raw, declaredType, lookup))
    .filter(Boolean)
    .join(",");
}

function resolveEnum(raw, declaredType, lookup) {
  const parsed = enumSymbol(raw);
  if (!parsed.symbol) return "";
  const type = parsed.type ?? declaredType;
  if (type) {
    const value = lookup.byType.get(type)?.get(parsed.symbol);
    if (value !== undefined) return value;
    throw new Error(`unknown enum ${type}::${parsed.symbol}`);
  }
  const candidates = lookup.bySymbol.get(parsed.symbol);
  if (candidates?.size === 1) return candidates.values().next().value;
  throw new Error(`ambiguous or unknown enum ${parsed.symbol}`);
}

function enumSymbol(value) {
  const compact = value.replace(/\(int\)|int\(|\)|\s/g, "");
  const segments = compact.split("::");
  return {
    type: segments.length > 1 ? segments.at(-2) : undefined,
    symbol: segments.at(-1),
  };
}
