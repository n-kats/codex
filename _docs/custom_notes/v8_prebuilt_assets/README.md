# Local Bazel-built V8 pair for Make/MCP tests

## Purpose

Make/MCP Cargo tests need the V8 sandbox archive and its matching generated Rust
binding. The public Codex release tag can lag current source changes or may not
exist yet, so workspace tests must build the pair from the checked-out Bazel
graph instead of depending on that release.

## Behavior

- `make build-v8-test-assets` runs through the normal Docker runner. The
  `docker/Dockerfile` image includes Bazelisk 1.30.0 as `bazel`; `.bazelversion`
  selects Bazel 9.2.0 inside the container. No host Bazel installation is needed.
- `make build-v8-test-assets` builds the Linux x86_64 sandbox release-pair
  target from `third_party/v8/BUILD.bazel` and stages its archive, binding, and
  checksums under `_tmp/assets`.
- The pair is generated explicitly with `make build-v8-test-assets`, which
  runs the full V8 Bazel build in Docker and stages the result under `_tmp/assets`.
  This can be a long first build, so it is not an automatic prerequisite of
  `make almost` or the other Cargo test targets. Those targets consume the
  staged pair; if it is missing, `v8_env.py` reports how to stage it.
- `scripts/codex_package/v8_env.py` verifies both files against the staged
  checksum manifest and passes their paths to Cargo. It does not fetch a
  GitHub Release asset or accept a manual override.
- The package builder remains on the published, version-pinned Codex release
  path. This local test path does not change package distribution.

## Why this path is required

The MCP failure was an HTTP 404 while requesting the checksum manifest for
`rusty-v8-v152.2.0-codex1`, before any Rust tests started. The existing local
`_tmp/assets` pair at that point was also stale: its binding advertised
`V8_TYPED_ARRAY_MAX_SIZE_IN_HEAP = 0`, while the current Bazel bindgen inputs
select 64 for Linux. Rebuilding from the current Bazel graph fixes both the
missing-asset dependency and the stale binding risk.

This is the primary V8 input path for Make/MCP tests, not a fallback to an
alternate artifact source. If the Bazel build fails, Cargo tests must stop;
they must not silently use an older public or cached pair.

## Verification

- Build or refresh the pair explicitly with `make build-v8-test-assets`; Bazel
  runs in the Docker image and stages the pair through the repository mount.
- `make clean` removes `_tmp/assets`, including the staged V8 archive, binding,
  and checksums. `make clean-dry-run` lists that directory before removal.
- The Docker-mounted Bazel output/cache remains reusable, like the Cargo
  dependency cache; `make clean` removes staged test inputs and Cargo outputs.
- Then run the full documented workflow through MCP:
  `run_make_almost_equivalent`.
- A successful run must pass artifact resolution and the V8 TypedArray checks;
  helper tests alone do not establish that the generated V8 pair works.

## Related files

- `Makefile`
- `.github/scripts/rusty_v8_bazel.py`
- `scripts/codex_package/v8.py`
- `scripts/codex_package/v8_env.py`
- `third_party/v8/BUILD.bazel`
