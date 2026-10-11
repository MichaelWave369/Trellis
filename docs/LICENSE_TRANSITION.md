# Trellis licensing transition

From the commit that introduces this document, the Trellis project's copyright-controlled source code and documentation are offered under the **MIT License**; see the repository's `LICENSE` file and `Cargo.toml` metadata. This change was requested by the repository owner.

## Historical revisions

Earlier released and pinned Trellis revisions were published under **AGPL-3.0-only**. Those existing grants are not revoked by this change, and referencing an earlier immutable revision does **not** make that revision an MIT-licensed checkout. Consult the `LICENSE` file at the exact commit being built. The preceding Trellis main revision was `9522aad85e04d75907689f97f4892d8e34ef9546`.

## Third-party content

The project-level MIT grant does not automatically relicense third-party dependencies, externally sourced material, or independently licensed packages delivered *through* Trellis. The package catalog's example manifests declare their own package licenses and those declarations have not been changed or independently audited here. Preserve upstream notices and obligations when bundling dependencies or fixtures.

## PhiOS pin

PhiOS v0.60 initially used an immutable, pre-transition Trellis source commit `c4036f38e8b635709d069e505506191dfccf8c5c` for ephemeral CI staging. That earlier SHA remains AGPL-licensed as published. Updating PhiOS to use the MIT-licensed Trellis revision requires an explicit source-pin update and a new verification run; never substitute the moving `main` branch for a fixed source revision.

## Provenance

Repository history attributes the code commits to the project owner's account. This does not, by itself, prove that every embedded component is free of third-party rights. Before redistributing binaries, independently review all embedded code, assets and dependency notices. Do not represent that audit as complete simply because the project's top-level license is MIT.
