# Security Policy

cargo-target-gc deletes build artifacts, so unsafe cleanup behavior is treated as a
security-sensitive bug.

## Supported Versions

Until the first stable release, only the latest commit on the default branch is
supported. After `1.0`, supported release lines will be listed here.

## Reporting a Vulnerability

Do not file a public issue for a suspected unsafe deletion bug. Report it
privately to the maintainer or through the repository's private vulnerability
reporting channel once the project is hosted publicly.

Please include:

- cargo-target-gc version or commit SHA.
- Operating system and filesystem type if relevant.
- The command that was run.
- The target project layout.
- Whether symlinks, nested workspaces, or custom target directories were
  involved.
- The expected and actual deletion scope.

The maintainer should acknowledge confirmed reports within 7 days, provide a
fix plan, and publish a security note when a release is available.
