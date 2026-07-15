# Ebuild Packaging Guidelines for ezmerge-overlay

To maintain a high Trust Score and ensure an effortless installation experience for beginners, all packages in `ezmerge-overlay` must meet standard QA guidelines.

## 📋 Quality Standards Checklist

1. **EAPI Version**: Always use `EAPI=8` (or the latest standard EAPI approved by Gentoo Council).
2. **Metadata**: Each package directory must contain `metadata.xml` defining the upstream homepage, maintainer emails, and a description of local USE flags.
3. **Thin Manifests**: The overlay uses thin manifests (`thin-manifests = true` in `layout.conf`). Run `ebuild <file> manifest` to regenerate manifests before committing.
4. **License Declarations**: Ensure `LICENSE` matches Gentoo standards. Avoid unreviewed proprietary licenses without user consent.
5. **No Slot Conflicts**: Slot allocations must be explicit.

---

## 🛠️ Sample Cargo Ebuild Structure

Since many modern packages (including `ezmerge-cli`) are written in Rust, here is the boilerplate template for a Cargo-based ebuild:

```gentoo
# Copyright 2026 ezMerge Developers
# Distributed under the terms of the MIT License

EAPI=8

inherit cargo

DESCRIPTION="A short description of your application"
HOMEPAGE="https://github.com/developer/app"
SRC_URI="https://github.com/developer/app/archive/refs/tags/v${PV}.tar.gz"

LICENSE="MIT"
SLOT="0"
KEYWORDS="~amd64 ~x86"

BDEPEND=">=virtual/rust-1.70.0"

src_unpack() {
	default
	# cargo_src_unpack will unpack cargo dependencies if pre-packaged crate tarball exists
}
```
