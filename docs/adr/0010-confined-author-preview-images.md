# ADR 0010: Confined local images in the SciELO Author preview

## Status

Accepted.

## Context

The first Packtools preview intentionally removed every image because generated
HTML is untrusted and a browser-visible local path could expose arbitrary files
or trigger network access. Scientific articles nevertheless depend on figures,
and Packtools emits relative `img src` values from JATS `graphic` and
`inline-graphic` references.

## Decision

The native desktop service, not the webview, discovers those two JATS reference
types. It accepts at most 64 references of at most 1 KiB each. References must
be relative paths without parent components or URI schemes. Each candidate is
canonicalized below the saved XML document directory; a final symbolic link is
denied. Files are limited to 8 MiB each and 32 MiB in aggregate.

PNG, JPEG, GIF, WebP, and TIFF are identified by their contents; extensions and
XML MIME hints are not trusted. Browser-safe formats pass through unchanged.
TIFF files are decoded with strict 16,384-pixel width/height limits, a 128 MiB
decoder-allocation ceiling, and the existing 8 MiB output ceiling, then encoded
as PNG. SVG, audio, video, remote URLs, missing files, and unknown content are
skipped without disabling the text preview. Packtools' historical extension
normalization from missing/TIFF extensions to JPEG remains the exact preview
source key; an adjacent JPEG is preferred, with the original TIFF used as the
bounded conversion fallback.

Approved bytes are base64-encoded by the native service and returned as data
URLs keyed by the generated relative `src`. The HTML sanitizer retains `img`
only when the exact key exists, replaces its source with that data URL, removes
all other URL-bearing attributes, and applies numeric bounds to dimensions. The
permissionless origin-opaque iframe keeps scripts, navigation, network, forms,
objects, SVG, audio, and video disabled; its CSP permits `data:` only in
`img-src`.

## Consequences

- Saved JATS packages can display adjacent local raster figures offline.
- `.tif` and `.tiff` figures display without requiring browser TIFF support or
  an external conversion program.
- The webview receives copied image bytes, not filesystem paths or filesystem
  permissions.
- Unsaved documents and missing/unsupported assets retain a safe textual
  preview and report only loaded/skipped counts.
- Very large packages require a future explicit media workflow rather than
  silently expanding the preview's memory budget.
