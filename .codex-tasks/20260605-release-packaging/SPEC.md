# Release Packaging

## Goal

Add a GitHub Release flow for Hook that publishes practical release assets similar in shape to Aether: install/update shell scripts plus platform-specific packages.

## Requirements

- Trigger release packaging from stable tags such as `vX.Y.Z`.
- Build using Hook's actual backend crate and embedded frontend workflow.
- Produce platform packages that users can download directly from GitHub Releases.
- Include checksums and scripts that make installation/update straightforward.
- Keep Docker Compose source-build deployment documented and intact.
- Do not add fake or untested release paths.
