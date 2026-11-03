import { syntaxTree } from "@codemirror/language";
import type { ChangeSet, ChangeSpec, EditorState } from "@codemirror/state";

type SyntaxNode = ReturnType<typeof syntaxTree>["topNode"];

function nearest(node: SyntaxNode, name: string): SyntaxNode | null {
  for (let current: SyntaxNode | null = node; current; current = current.parent) {
    if (current.name === name) return current;
  }
  return null;
}

function editedOpeningTagName(
  state: EditorState,
  from: number,
  to: number,
): SyntaxNode | null {
  const tree = syntaxTree(state);
  const positions = new Set([
    Math.max(0, from - 1),
    Math.min(state.doc.length, from),
    Math.min(state.doc.length, to),
  ]);
  for (const position of positions) {
    for (const bias of [-1, 1] as const) {
      const resolved = tree.resolveInner(position, bias);
      const openTag = nearest(resolved, "OpenTag");
      const nameNode = resolved.name === "TagName"
        ? resolved
        : openTag?.getChild("TagName") ?? null;
      if (
        nameNode
        && nameNode.parent?.name === "OpenTag"
        && from <= nameNode.to
        && to >= nameNode.from
      ) {
        return nameNode;
      }
    }
  }
  return null;
}

export function linkedClosingTagChange(
  startState: EditorState,
  nextState: EditorState,
  changes: ChangeSet,
): ChangeSpec | null {
  let linkedChange: ChangeSpec | null = null;
  changes.iterChanges((fromA, toA) => {
    if (linkedChange) return;
    const nameNode = editedOpeningTagName(startState, fromA, toA);
    if (!nameNode) return;
    const element = nearest(nameNode, "Element");
    const closeName = element?.getChild("CloseTag")?.getChild("TagName");
    if (!closeName) return;
    const oldName = startState.doc.sliceString(nameNode.from, nameNode.to);
    if (startState.doc.sliceString(closeName.from, closeName.to) !== oldName) return;
    const openFrom = changes.mapPos(nameNode.from, -1);
    const openTo = changes.mapPos(nameNode.to, 1);
    const newName = nextState.doc.sliceString(openFrom, openTo);
    if (!/^[:A-Z_a-z\u00c0-\uffff][:\-.0-9A-Z_a-z\u00b7-\uffff]*$/u.test(newName)) return;
    linkedChange = {
      from: changes.mapPos(closeName.from, -1),
      to: changes.mapPos(closeName.to, 1),
      insert: newName,
    };
  });
  return linkedChange;
}
