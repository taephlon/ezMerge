# Copyright 2026 ezMerge Developers
# Distributed under the terms of the MIT License

EAPI=8

inherit cargo

DESCRIPTION="Making Gentoo overlays and installation effortless, without hiding Portage's power"
HOMEPAGE="https://github.com/ezmerge/ezmerge-cli"
SRC_URI="https://github.com/ezmerge/ezmerge-cli/archive/v${PV}.tar.gz"

LICENSE="MIT"
SLOT="0"
KEYWORDS="~amd64 ~x86"
IUSE=""

# Dependencies needed at build time
BDEPEND="
	>=virtual/rust-1.70.0
"

# Runtime dependencies (like eselect-repository for overlay management)
RDEPEND="
	app-eselect/eselect-repository
	sys-apps/portage
"

src_unpack() {
	if [[ -n ${EGIT_REPO_URI} ]]; then
		git-r3_src_unpack
	else
		default
	fi
}
