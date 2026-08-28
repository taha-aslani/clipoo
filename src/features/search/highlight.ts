export interface HighlightRange {
  start: number;
  end: number;
}

function normalizeChar(char: string): string {
  const code = char.codePointAt(0);
  if (code === undefined) {
    return char;
  }

  switch (code) {
    case 0x064a:
      return "\u06cc";
    case 0x0643:
      return "\u06a9";
    case 0x0624:
      return "\u0648";
    case 0x0623:
    case 0x0625:
    case 0x0622:
      return "\u0627";
    case 0x0621:
    case 0x0626:
    case 0x0640:
    case 0x200b:
    case 0x200c:
    case 0x200d:
    case 0xfeff:
      return "";
    default:
      break;
  }

  if ((code >= 0x064b && code <= 0x065f) || code === 0x0670) {
    return "";
  }
  if ((code >= 0x0610 && code <= 0x061a) || (code >= 0x06d6 && code <= 0x06ed)) {
    return "";
  }
  if (/\s/u.test(char)) {
    return "";
  }

  return char;
}

function needlesFromQuery(query: string): string[] {
  const trimmed = query.trim();
  if (!trimmed) {
    return [];
  }

  const needles = trimmed.split(/\s+/u).filter((part) => part.length > 0);
  if (needles.length > 1) {
    needles.push(trimmed);
  }
  return needles;
}

function rangesForNeedle(text: string, needle: string): HighlightRange[] {
  const origin: number[] = [];
  let normalized = "";
  let originalIndex = 0;

  for (const char of text) {
    const mapped = normalizeChar(char);
    for (const mappedChar of mapped) {
      origin.push(originalIndex);
      normalized += mappedChar;
    }
    originalIndex += char.length;
  }

  const normalizedQuery = [...needle].map(normalizeChar).join("");
  if (!normalizedQuery) {
    return [];
  }

  const ranges: HighlightRange[] = [];
  let from = 0;

  while (from + normalizedQuery.length <= normalized.length) {
    const matchAt = normalized.indexOf(normalizedQuery, from);
    if (matchAt === -1) {
      break;
    }

    const start = origin[matchAt];
    const last = origin[matchAt + normalizedQuery.length - 1];
    if (start === undefined || last === undefined) {
      break;
    }

    const matchedChar = [...text.slice(last)][0] ?? "";
    ranges.push({ start, end: last + matchedChar.length });
    from = matchAt + Math.max(normalizedQuery.length, 1);
  }

  return ranges;
}

function mergeRanges(ranges: HighlightRange[]): HighlightRange[] {
  if (ranges.length === 0) {
    return [];
  }

  const sorted = [...ranges].sort((left, right) => left.start - right.start);
  const merged: HighlightRange[] = [];
  let current = sorted[0];
  if (!current) {
    return [];
  }

  for (let index = 1; index < sorted.length; index += 1) {
    const next = sorted[index];
    if (!next) {
      continue;
    }
    if (next.start <= current.end) {
      current = { start: current.start, end: Math.max(current.end, next.end) };
    } else {
      merged.push(current);
      current = next;
    }
  }

  merged.push(current);
  return merged;
}

export function findHighlightRanges(text: string, query: string): HighlightRange[] {
  const ranges = needlesFromQuery(query).flatMap((needle) => rangesForNeedle(text, needle));
  return mergeRanges(ranges);
}
