export type ApplicationCloseDecision = "save" | "discard" | "cancel";

interface ApplicationCloseOptions<T> {
  documents: readonly T[];
  isDirty: (document: T) => boolean;
  decide: (dirtyCount: number) => Promise<ApplicationCloseDecision>;
  save: (document: T) => Promise<boolean>;
}

export async function approveApplicationClose<T>({
  documents,
  isDirty,
  decide,
  save,
}: ApplicationCloseOptions<T>): Promise<boolean> {
  const dirtyDocuments = documents.filter(isDirty);
  if (dirtyDocuments.length === 0) return true;

  const decision = await decide(dirtyDocuments.length);
  if (decision === "cancel") return false;
  if (decision === "discard") return true;

  for (const document of dirtyDocuments) {
    if (isDirty(document) && !(await save(document))) return false;
  }
  return true;
}
