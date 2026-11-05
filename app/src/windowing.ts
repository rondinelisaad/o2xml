export interface DetachedDocumentRequest {
  documentId: number;
  sourceLabel: string;
  readyEvent: string;
}

interface ScreenPoint {
  screenX: number;
  screenY: number;
}

interface WindowBounds extends ScreenPoint {
  outerWidth: number;
  outerHeight: number;
}

const WINDOW_LABEL = /^[a-zA-Z0-9_/:.-]+$/u;
const READY_EVENT = /^scixml-document-[a-zA-Z0-9_-]+-ready$/u;

export function detachedDocumentRequest(search: string): DetachedDocumentRequest | null {
  const parameters = new URLSearchParams(search);
  const documentId = Number(parameters.get("detachedDocument"));
  const sourceLabel = parameters.get("sourceWindow") ?? "";
  const readyEvent = parameters.get("readyEvent") ?? "";
  if (!Number.isSafeInteger(documentId) || documentId < 1) return null;
  if (!WINDOW_LABEL.test(sourceLabel) || !READY_EVENT.test(readyEvent)) return null;
  return { documentId, sourceLabel, readyEvent };
}

export function isOutsideWindowDrop(point: ScreenPoint, bounds: WindowBounds, margin = 8): boolean {
  if (!Number.isFinite(point.screenX) || !Number.isFinite(point.screenY)) return false;
  if (point.screenX === 0 && point.screenY === 0) return false;
  return point.screenX < bounds.screenX - margin
    || point.screenY < bounds.screenY - margin
    || point.screenX > bounds.screenX + bounds.outerWidth + margin
    || point.screenY > bounds.screenY + bounds.outerHeight + margin;
}
