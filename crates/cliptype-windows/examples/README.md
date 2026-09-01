# P1 disposable Windows experiments

P1-S01 may place bounded, disposable Cargo examples in this directory, starting
with `p1_s01_windows_native.rs`. Experiment code must remain outside `src/`,
must not define or change production contracts, and must be removable without
changing the workspace member graph.

Any Windows-only experiment dependency belongs in a target-scoped development
section such as `[target.'cfg(windows)'.dev-dependencies]`. A Windows-only
example must retain a non-Windows stub `main` so native-neutral CI can still use
`--all-targets` without compiling Win32 imports.

Record the environment, procedure, observations, inference, and limitations in
`docs/research/P1_WINDOWS_NATIVE_SPIKE.md`. Research evidence does not override
accepted ADRs or normative documents.
