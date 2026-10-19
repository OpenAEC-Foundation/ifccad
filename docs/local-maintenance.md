# Local worktrees and build caches

Large caches are acceptable while work needs them. The maintenance policy is
to finish cleanup when a task is retired, rather than limit storage during
development. Size and commit age never establish that work is finished. There
are no hard budgets, blocked builds, automatic deletion timers or background
cleanup services.

## Audit

Run from a checkout of this repository:

```powershell
pwsh -NoProfile -File scripts/cleanup_local.ps1
pwsh -NoProfile -File scripts/cleanup_local.ps1 -Sizes
```

The first command reports technical removal blockers. `-Sizes` additionally
separates recognized build cache bytes from other `target` content per checkout.
It traverses the cache and can take longer; it is not a pre-build gate. Sizes
are logical file lengths in decimal GB, not physical disk allocation or
deduplicated hard-link sizes. `-Json` returns the same report for tooling.

Discovery covers the main repository, immediate Git checkouts below `.worktrees/`
and `.superpowers/dependencies/`, and worktrees registered with those repositories.
Repositories are deduplicated by their Git common directory. An unreadable
repository is reported for review without suppressing the other results or
changing Git's ownership/trust configuration. Archives and arbitrary directory
trees are not searched.

IFCCAD worktrees are checked against IFCCAD `main`. Separate codec worktrees use
their own `origin/main` when available, otherwise their own local `main`. A stale
remote-tracking ref can conservatively block removal; establish the actual
upstream integration before retrying. The script does not fetch or infer a merge
from an issue/PR title, an idle chat, age or an unrelated IFCCAD commit.

## Finish a task

1. Confirm the task is complete, its commits are integrated into its repository's
   main line and no active or paused work needs the checkout. Inspect all local
   changes and ignored material. A technically removable checkout may still be
   in active use.
2. Select the unique material to retain under the AGENTS.md retention rules.
   Keep applicable reports and matching detailed measurement results, exact
   source/configuration/dependency identities and needed inputs. Keep source
   files, user-authored material and plans still needed for follow-up work.
   Compare existing copies and verify archived copies before deleting originals.
   Do not archive entire caches or all historical runs by default.
3. Move that selected material to an ignored task-specific archive below
   `.superpowers/worktree-archives/` in the primary IFCCAD checkout, preserving
   relative paths. Record an inventory, retention reasons and discarded categories.
4. Stop builds using the selected cache, and do not start new ones during cleanup.
   Run the audit, then explicitly select the completed worktree:

   ```powershell
   pwsh -NoProfile -File scripts/cleanup_local.ps1 -Apply -RemoveWorktree task-name
   ```

   This removes its recognized cache, an ignored Cargo.lock and the clean Git
   worktree, followed by its verified integrated local task branch. Git worktree
   removal is never forced. Local edits, unfamiliar ignored content, nonempty
   `target/tmp`, measurement directories and locked worktrees block removal.
   The candidate is checked again immediately before removal. A changed commit,
   branch, integration state or newly found local material invalidates the audit.
   Local branch deletion is conditional on its still pointing at the checked
   commit, so a concurrent branch update is preserved and reported.
   The script does not decide which evidence is superseded or archive it itself.
5. Audit again. Account for caches created in main or dependency checkouts as
   well as the task worktree. Remove cache that the finished task no longer needs;
   retain warm cache if a concrete ongoing task still uses it. State that reason
   and task in the completion record. For unresolved cleanup, record the exact
   blocker, last commit date and retained-material path, then resolve recoverable
   file blockers rather than quietly retaining a finished worktree.

Use the same checklist for upstream codec repairs after their integration. The
script reports codec primary checkouts but never deletes a primary repository:
it may contain shared Git metadata needed by other worktrees, local branches or
dependency configuration. Retiring such a clone requires a separate review of
all its worktrees/refs, retained dependencies and files; its cache can still be
explicitly selected for cleanup.

## Clean one cache

```powershell
# Default is main only; other active worktrees are untouched.
pwsh -NoProfile -File scripts/cleanup_local.ps1 -Apply -CleanBuildCache

# Select a single task or dependency checkout by its reported name or key.
pwsh -NoProfile -File scripts/cleanup_local.ps1 -Apply -CleanBuildCache -CacheCheckout codec-task
pwsh -NoProfile -File scripts/cleanup_local.ps1 -Apply -CleanBuildCache -CacheCheckout .superpowers/dependencies/codec-copy
```

This changes the older blanket behavior of `-CleanBuildCache`, which cleared
every IFCCAD worktree. An exact reported relative key disambiguates repeated
names. There is no wildcard/all-worktree cleanup option. Selecting a cache does
not declare its source task complete, authorize worktree removal or modify its
source, Cargo.lock, dependency selection or build profiles.

Recognized cache entries under the checkout's conventional `target/` are
`debug`, `release`, `wasm32-unknown-unknown`, `flycheck0`, `.rustc_info.json`,
`CACHEDIR.TAG` and empty `tmp`. Put research output outside those cache entries;
put lasting evidence outside `target/`. Other target entries remain for review,
including measurement runs. Custom Cargo target directories/profiles are not
guessed or removed; review their actual configuration separately.

Mutations are refused while Cargo/Rust build tools are running or a selected
Cargo profile lock cannot be opened exclusively. The process check is
conservative and can also notice another project's build. Do not start builds
concurrently with cleanup. Paths outside the managed roots, linked directories,
nested cache links, tracked cache files and unreadable/external repositories
are protected. Main and dependency source checkouts are not deleted by cache
cleanup. No measurement is rerun to justify deleting an old artifact.

## Verify script changes

```powershell
pwsh -NoProfile -File scripts/tests/cleanup_local.Tests.ps1
```

The regression scenarios create disposable independent repositories and exercise
real audit, size reporting, targeted cache cleanup, integration checks and
worktree removal. The OS process inventory alone is controlled within the test
process, so unrelated real builds do not interfere with fixture cleanup; the
running-build blocker is separately exercised against real fixture files.
The suite does not clean production worktrees or build Rust. On Windows, its
junction scenario needs permission to create a junction in the disposable fixture.
