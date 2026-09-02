# Release asset policy

A public ClipType Windows prerelease must satisfy all of the following:

1. the executable contains application icon resource id `1`;
2. the tray shell loads notification-area icon resource id `2` with a stock-icon fallback limited to development failures;
3. the release archive, portable executable, dependency inventory, build information, and checksum manifest come from one exact source commit;
4. every published file has a SHA-256 entry, a Sigstore keyless bundle, and a GitHub artifact attestation;
5. the GitHub release is marked as a prerelease;
6. release notes state clearly that no Authenticode publisher certificate is configured;
7. publishing does not convert hosted-runner coverage into a universal per-application compatibility guarantee.
