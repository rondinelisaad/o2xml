const root = document.querySelector<HTMLDivElement>("#app");

function showStartupFailure(reason: unknown): void {
  if (!root || root.dataset.initialized === "true") return;
  const detail = reason instanceof Error ? reason.message : String(reason);
  root.replaceChildren();
  const panel = document.createElement("main");
  panel.className = "startup-status startup-error";
  panel.setAttribute("role", "alert");
  const title = document.createElement("strong");
  title.textContent = "Não foi possível iniciar o SciXML";
  const message = document.createElement("span");
  message.textContent = detail || "Falha desconhecida durante a inicialização.";
  panel.append(title, message);
  root.append(panel);
}

window.addEventListener("error", (event) => {
  showStartupFailure(event.error ?? event.message);
});

window.addEventListener("unhandledrejection", (event) => {
  showStartupFailure(event.reason);
});

window.setTimeout(() => {
  if (root?.dataset.initialized !== "true") {
    showStartupFailure("A interface não respondeu dentro do tempo esperado.");
  }
}, 8_000);
