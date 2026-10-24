# Builds and automatic cache maintenance

[English](build-cache.md) · [Français](build-cache.fr.md) · [Documentation](README.md)

`make build`, `make release` and `make test` use `scripts/build.py`.
The wrapper maintains this repository's recognized build caches before and after
Cargo, including failed builds. It does not restart a daemon.

- Shared budget: **10 GiB**, not 10 GiB per worktree.
- Age limit: **7 days**, based on the cache contents' latest modification.
- Oldest profiles are removed first when the budget is exceeded.
- Only `deps`, `build`, `incremental` and `.fingerprint` inside
  `target/debug` and `target/release` are removed. Rebuilding takes longer afterward.
- Top-level executables, sources, backups and Bridget state remain.
  Other projects' caches are outside the scope.
- A busy Cargo lock or an open file preserves the profile. If `lsof` cannot
  establish a reliable result, cleanup is refused. The budget is a target,
  not a reason to bypass these guards.
- Cargo arguments and its exit code are preserved. A maintenance failure warns
  without hiding a failed build or failing a successful one. Failed builds do
  not trigger worktree removal. Keep diagnostic logs outside intermediate cache
  directories; those intermediates may be removed even after a failed build.

Requirements: Unix, Python **3.11+**, Git, Cargo and `lsof`. No extra Python
package is required. Sizes are logical; APFS clones can make physical space
recovery differ.

## Commands

From the repository or one of its worktrees:

```sh
make build
make release
make test
make clean-builds DRY_RUN=1  # Preview; no deletion
make clean-builds          # Apply age and size policy
make test-build-cleanup    # Isolated cleanup tests

python3 scripts/build.py cargo test --workspace --release
python3 scripts/build.py cargo check --locked

BRIDGET_BUILD_CACHE_GIB=15 BRIDGET_BUILD_CACHE_DAYS=14 make release
```

`make clean` applies the same policy. It no longer runs `cargo clean`, which
would also remove binaries potentially in use. Zero purges all eligible caches,
with all guards still enforced. Negative, infinite or invalid values are refused.

## Temporary build directories

Before and after a wrapped build, existing directories named `bridget-build-*` directly
under the user's `.cache` directory can be registered when selected by
`CARGO_TARGET_DIR` or `--target-dir`. A marker ties the directory to this
repository. A directory created by a failed build is registered afterward too.
It then shares the worktree cache budget.

Older unmarked directories and directories owned by other repositories are
ignored.

After a separately verified installation, intermediate files in a registered
temporary directory can be removed immediately:

```sh
python3 scripts/build.py installed --target "$bridget_build_dir" --binary "$bridget_installed_binary"
```

Both variables must contain the actual absolute paths. This requires a registered
temporary directory and a matching SHA-256 binary installed outside it. The
temporary executable and Cargo lock remain. `make install` performs this step
after installation; this guide does not authorize replacing an active installation.

## Scope and limits

Direct `cargo` calls bypass maintenance. Use Make or the wrapper for new builds,
including agent-driven builds. This is required in the repository's agent
instructions. Reuse the checkout's existing target directory rather than copying
it for each test. Use a registered temporary directory when isolation is needed. The K1 installer uses the wrapper when present;
its older standalone mode keeps its Cargo fallback. The historical SSH deployment
path is unchanged and adds no remote cleanup prerequisite.

Custom profiles, cross-compilation outputs and arbitrary external directories
are not scanned. The global Cargo cache is not covered. Preserve sources,
worktrees, state and rollback artifacts independently of build-cache cleanup.

Maintenance runs with builds, not on a timer. No background model or scheduler
is required. Installed executables and rollback copies are outside this cache
budget and must be accounted for separately at delivery.
