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

if (!/^[A-Za-z0-9_.-]+$/u.test(target)) {
  throw new Error(`invalid Rust target triple: ${target}`);
}

const extension = target.includes("-windows-") ? ".exe" : "";
const executable = `scixml-packtools-validator${extension}`;
const source = resolve(
  workspaceDirectory,
  "target",
  "packtools-sidecar",
  executable,
);
if (!existsSync(source) || !statSync(source).isFile()) {
  throw new Error(`Packtools validator sidecar was not found: ${source}`);
}

const destinationDirectory = resolve(appDirectory, "src-tauri", "binaries");
const destination = resolve(
  destinationDirectory,
  `scixml-packtools-validator-${target}${extension}`,
);
mkdirSync(destinationDirectory, { recursive: true });
copyFileSync(source, destination);
const licenseSource = resolve(
  workspaceDirectory,
  "target",
  "packtools-sidecar",
  "THIRD-PARTY-LICENSES.txt",
);
if (!existsSync(licenseSource) || !statSync(licenseSource).isFile()) {
  throw new Error(`Packtools third-party notices were not found: ${licenseSource}`);
}
const licenseDestinationDirectory = resolve(
  appDirectory,
  "src-tauri",
  "resources",
  "packtools-sidecar",
);
mkdirSync(licenseDestinationDirectory, { recursive: true });
copyFileSync(
  licenseSource,
  resolve(licenseDestinationDirectory, "THIRD-PARTY-LICENSES.txt"),
);
console.log(`staged ${destination}`);
