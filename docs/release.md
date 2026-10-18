# Release and installer workflow

O2XML builds installers through the manually triggered **Build installers**
GitHub Actions workflow. Every run first executes the Rust and frontend quality
gates. Successful runs create or update a **draft** GitHub release; they never
publish it automatically.

## Produced bundles

| Platform | Architecture | Bundles |
|---|---|---|
| Linux | x86-64 | Debian package and AppImage |
| Windows | x86-64 | MSI and NSIS installer |
| macOS | Apple Silicon | application bundle and DMG |
| macOS | Intel | application bundle and DMG |

The isolated `xml-libxml2-helper` is compiled for the same target as the desktop
application and embedded as a Tauri sidecar. Linux and macOS CI builds request
static libxml2/libxslt linkage so that Homebrew or runner-specific library paths
do not leak into the distributed helper. Windows uses the `static-md` vcpkg
triplet. The official vcpkg libxml2 port does not enable its legacy Schematron
module; Windows therefore requires the supported SchXslt/Saxon adapter for
Schematron, while DTD, XSD, RELAX NG, XPath, and XSLT 1.0 remain available in
the native helper.

## Start a draft release

1. Synchronize the version in the workspace `Cargo.toml`, `app/package.json`,
   and `app/src-tauri/tauri.conf.json`.
2. Update both lockfiles and run the quality gates documented in the README.
3. Push the release commit.
4. Open **Actions → Build installers → Run workflow**, select the intended
   branch, and start the run.
5. Download and test every artifact attached to the draft release before
   publication.

The equivalent authenticated GitHub CLI command is:

```sh
gh workflow run release.yml --ref main
```

The release tag is derived by Tauri as `o2xml-v<version>`. Do not reuse an
existing published version.

## Local package verification

Build and stage the native helper before invoking Tauri:

```sh
cargo build --locked --release --package xml-libxml2-helper
node app/scripts/stage-sidecar.mjs "$(rustc --print host-tuple)"
cd app
npm ci
npm run tauri -- build --bundles app
```

The staged executable below `app/src-tauri/binaries/` is generated output and
must not be committed.

## Signing and secrets

The initial workflow produces unsigned Linux/Windows artifacts and ad-hoc-signed
macOS artifacts suitable for internal testing. Public distribution requires:

- an Apple Developer ID certificate, hardened runtime signing, and notarization;
- a trusted Windows Authenticode certificate and timestamping configuration;
- protection rules for the GitHub release environment; and
- repository/environment secrets configured only in GitHub, never committed to
  source control or printed in workflow logs.

Follow Tauri's signing documentation when enabling these credentials. Limit
secret access to protected release branches and require review for the release
environment.

## Smoke-test checklist

Test on a clean supported machine or VM for each produced architecture:

- install, launch, and uninstall without elevated privileges beyond those the
  native installer normally requests;
- open, edit, save, format, and validate a representative XML document;
- confirm the packaged libxml2 helper executes without an external package
  manager or developer toolchain;
- verify the JATS Inspector, Problems panel, XPath panel, and project reload;
- verify Portuguese (Brazil) and English UI selection;
- check that local/remote resource policy remains deny-by-default; and
- inspect the artifact and application logs for credentials or document content.

Keep the GitHub release as a draft if any platform fails. Correct the source or
workflow, increment the version when artifact identity could be ambiguous, and
run the workflow again. Publishing the draft is a deliberate maintainer action,
not an automated CI step.
