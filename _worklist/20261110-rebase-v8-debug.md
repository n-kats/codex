# Rebase and test worklist

## Goal

Resolve the rebase conflicts, preserve the custom requirements, and pass the documented MCP `almost` workflow. Do not stage or commit.

## Status

- The source-side conflict resolutions and prior `almost` failures outside V8 have been handled. The index remains unmerged because `git add` is prohibited.
- After the MCP restart, the selected V8 test still fails at a one-byte TypedArray. This is reproducible and is not caused by MCP startup or test parallelism.
- The test exposes a real mismatch: the Linux V8 runtime uses a 64-byte inline TypedArray threshold, while the generated Rust binding advertises zero. Bindgen was missing the matching compile definition; it is now passed through the binding CcInfo.
- The latest `almost` log fails before tests: `v8_env.py` requests the checksum manifest from the not-yet-published `rusty-v8-v152.2.0-codex1` release and receives HTTP 404.
- Existing local `_tmp/assets` files match the old pair, but their generated binding still has `V8_TYPED_ARRAY_MAX_SIZE_IN_HEAP = 0`; they cannot validate the current Bazel fix.
- `stage-release-pair` already runs the Bazel build with top-level output downloads, then creates the deterministic gzip archive, copies the matching binding, and writes checksums. The shared staging code also packages the separate upstream Cargo build outputs, so packaging is kept in one helper rather than duplicated in Bazel rules.
- The canary and release workflows were invoking Bazel once directly and then invoking `stage-release-pair`, which builds the same target again. Both now use one staging invocation and pass the commit metadata through to that build.
- The `rusty_v8_bazel.py` unit suite passed earlier (12 tests); no build or test was run in the agent environment per `CUSTOM.md`.
- Make/MCP Cargo tests consume the checksum-verified local pair in `_tmp/assets`; its explicit Bazel staging step is separate from the test targets. Package release builds remain on published Codex assets.
- The host-side `bazel` failure exposed that asset staging bypassed the repository's Docker workflow. Staging now uses `scripts/docker_run.sh`; `docker/Dockerfile` supplies the CI-pinned Bazelisk, so no host Bazel installation is required.
- `make clean` and `clean-dry-run` now include the staged `_tmp/assets` V8 pair; the Bazel cache remains reusable.
- The V8 Bazel stage is a full source build and can take hours on a cold cache. It is now an explicit preparation target rather than an automatic prerequisite of `make almost`.
- Do not weaken the positive-length V8 checks or change the upstream V8 version. The old local pair must be regenerated because its binding reports a zero inline TypedArray threshold.

## Next

1. Rebuild the Docker image and run `make build-v8-test-assets`; confirm it
   stages the current sandbox pair under `_tmp/assets` through the repository
   mount. `make almost` runs this target before the Cargo test phase.

2. Use the MCP `almost` workflow to verify that it gets past artifact
   resolution and then inspect any actual Rust test failures.
3. Finish the rebase review and report the remaining unmerged index state; do not stage or commit.
