import { copyFileSync, existsSync, mkdirSync, statSync } from "node:fs";
import { execFileSync } from "node:child_process";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const scriptDirectory = dirname(fileURLToPath(import.meta.url));
const appDirectory = resolve(scriptDirectory, "..");
const workspaceDirectory = resolve(appDirectory, "..");
const requestedTarget = process.argv[2];
const target = requestedTarget ?? execFileSync(
  "rustc",
  ["--print", "host-tuple"],
  { encoding: "utf8" },
).trim();
const profile = process.argv[3] ?? "release";

if (!/^[A-Za-z0-9_.-]+$/u.test(target)) {
  throw new Error(`invalid Rust target triple: ${target}`);
}
if (profile !== "debug" && profile !== "release") {
  throw new Error(`invalid Cargo profile: ${profile}`);
}

const extension = target.includes("-windows-") ? ".exe" : "";
const executable = `xml-libxml2-helper${extension}`;
const candidates = [
  resolve(workspaceDirectory, "target", target, profile, executable),
  resolve(workspaceDirectory, "target", profile, executable),
];
const source = candidates.find((candidate) => existsSync(candidate) && statSync(candidate).isFile());
if (!source) {
  throw new Error(
    `xml-libxml2-helper was not found; build it for ${target} in ${profile} mode first`,
  );
}

const destinationDirectory = resolve(appDirectory, "src-tauri", "binaries");
const destination = resolve(
  destinationDirectory,
  `xml-libxml2-helper-${target}${extension}`,
);
mkdirSync(destinationDirectory, { recursive: true });
copyFileSync(source, destination);
console.log(`staged ${destination}`);
