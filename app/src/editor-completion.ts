export interface AttributeApplication {
  insert: string;
  valueFromOffset: number;
  valueToOffset: number;
}

export interface AttributeTextContext {
  elementName: string;
  presentAttributes: string[];
  editingName: { from: number; to: number } | null;
  value: { attributeName: string; from: number; to: number } | null;
}

const XML_NAME = "[:\\-.\\w\\u00b7-\\uffff]";

export function attributeTextContext(
  source: string,
  position: number,
): AttributeTextContext | null {
  const cursor = Math.min(Math.max(position, 0), source.length);
  const windowFrom = Math.max(0, cursor - 4096);
  const beforeCursor = source.slice(windowFrom, cursor);
  const relativeOpening = beforeCursor.lastIndexOf("<");
  if (relativeOpening < 0 || relativeOpening <= beforeCursor.lastIndexOf(">")) return null;

  const opening = windowFrom + relativeOpening;
  const prefix = source.slice(opening, cursor);
  if (/^<\s*[/!?]/u.test(prefix)) return null;
  const element = prefix.match(new RegExp(`^<\\s*(${XML_NAME}+)`, "u"));
  if (!element) return null;

  const elementName = element[1];
  if (!elementName) return null;
  const afterElement = opening + element[0].length;
  if (cursor <= afterElement) return null;

  const tagEnd = findTagEnd(source, opening);
  const tagSource = source.slice(opening, tagEnd);
  const assigned = new RegExp(`\\s+(${XML_NAME}+)\\s*=`, "gu");
  const presentAttributes = Array.from(tagSource.matchAll(assigned)).flatMap((match) =>
    match[1] ? [match[1]] : []
  );
  const valueMatch = prefix.match(
    new RegExp(`\\s+(${XML_NAME}+)\\s*=\\s*(["'])([^"']*)$`, "u"),
  );
  if (valueMatch?.index !== undefined) {
    const attributeName = valueMatch[1];
    const quote = valueMatch[2];
    if (!attributeName || !quote) return null;
    const valueFrom = opening + valueMatch.index + valueMatch[0].lastIndexOf(quote) + 1;
    let valueTo = cursor;
    while (valueTo < tagEnd && source[valueTo] !== quote) valueTo += 1;
    return {
      elementName,
      presentAttributes: presentAttributes.filter((name) => name !== attributeName),
      editingName: null,
      value: { attributeName, from: valueFrom, to: valueTo },
    };
  }

  const nameMatch = prefix.match(new RegExp(`\\s+(${XML_NAME}*)$`, "u"));
  if (!nameMatch?.index && nameMatch?.index !== 0) return null;
  const typedName = nameMatch[1] ?? "";
  const nameFrom = cursor - typedName.length;
  let nameTo = cursor;
  while (nameTo < tagEnd && new RegExp(XML_NAME, "u").test(source[nameTo] ?? "")) nameTo += 1;
  return {
    elementName,
    presentAttributes: presentAttributes.filter((name) => name !== typedName),
    editingName: { from: nameFrom, to: nameTo },
    value: null,
  };
}

function findTagEnd(source: string, opening: number): number {
  let quote = "";
  for (let index = opening + 1; index < source.length; index += 1) {
    const character = source[index];
    if (quote) {
      if (character === quote) quote = "";
    } else if (character === '"' || character === "'") {
      quote = character;
    } else if (character === ">") {
      return index;
    }
  }
  return Math.min(source.length, opening + 4096);
}

export function attributeApplication(
  name: string,
  defaultValue: string | null,
): AttributeApplication {
  const value = defaultValue ?? "";
  const valueFromOffset = name.length + 2;
  return {
    insert: `${name}="${value}"`,
    valueFromOffset,
    valueToOffset: valueFromOffset + value.length,
  };
}

export function otherAttributeNames(
  attributes: ReadonlyArray<{ name: string; from: number; to: number }>,
  editing: { from: number; to: number } | null,
): string[] {
  return attributes
    .filter((attribute) =>
      !editing || attribute.from !== editing.from || attribute.to !== editing.to
    )
    .map((attribute) => attribute.name);
}

export function isAttributeCompletionPosition(source: string, position: number): boolean {
  const boundedPosition = Math.min(Math.max(position, 0), source.length);
  const fragment = source.slice(Math.max(0, boundedPosition - 4096), boundedPosition);
  const opening = fragment.lastIndexOf("<");
  if (opening <= fragment.lastIndexOf(">")) return false;
  const tag = fragment.slice(opening);
  if (/^<[/!?]/u.test(tag)) return false;
  return /\s[:\-.\w\u00b7-\uffff]*$/u.test(tag)
    || /\s[:\-.\w\u00b7-\uffff]+\s*=\s*["'][^"']*$/u.test(tag);
}
