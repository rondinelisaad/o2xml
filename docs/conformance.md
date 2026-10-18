# External-engine conformance

External engines and upstream corpora are never downloaded by a normal build or
test run. These gates are explicit so contributors can verify a reviewed engine
without committing third-party binaries or silently changing versions.

## Jing V20241231

Reviewed source: `relaxng/jing-trang` tag `V20241231` at signed commit `a6bc004`.
The upstream project and Jing are under the 3-clause BSD license. The gate uses:

- `trang/test/compacttest.rnc`;
- `mod/rng-schema/test/compacttest.xml` (90 upstream cases);
- its recursive `doc/compact/relaxng.rnc` dependency;
- a real `build/jing.jar` produced from that checkout.

Reproduction:

```sh
git clone --recurse-submodules --branch V20241231 \
  https://github.com/relaxng/jing-trang.git /tmp/jing-trang-V20241231
cd /tmp/jing-trang-V20241231
export JAVA_HOME=/path/to/a/jdk
./ant ant-jar

cd /path/to/o2xml
O2XML_JING_UPSTREAM_ROOT=/tmp/jing-trang-V20241231 \
O2XML_JING_JAR=/tmp/jing-trang-V20241231/build/jing.jar \
O2XML_JAVA="$JAVA_HOME/bin/java" \
cargo test -p xml-validation --test jing_upstream -- --ignored
```

The O2XML test verifies the exact upstream version, resolves the recursive RNC
dependency through the offline catalog broker, rewrites the approved graph into
private staging, and validates the official corpus container with the real JAR.
The upstream 90-case compact conversion target was also run on 2026-10-18 and
passed after a temporary test-harness-only modernization from the obsolete
`saxon:node-set`/`saxon:output` extensions to their XSLT 2.0 equivalents. No
upstream patch or binary is part of the O2XML tree.

The run exposed a broker bug where `element include` and `element external`
were mistaken for dependency directives. The lexer now treats those words as
resource directives only when the next significant token is a quoted URI; the
regression is covered by a normal unit test.
