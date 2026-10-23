# Third-party notices and distributor data

The root [MIT License](./LICENSE) applies to NXC-authored code, including the
CLI/core, GUI source, Nix expressions, scripts, and documentation, except for
files or portions identified as third-party material below. It does not
relicense external services, datasets, application icons, trademarks, or
dependencies.

This is a source inventory and packaging guide, not a blanket permission to
redistribute third-party material. A distributor must review the actual data
and dependency versions included in their build and keep any required notices.

## AppStream data

NXC's Flake references
[snowfallorg/nixos-appstream-data](https://github.com/snowfallorg/nixos-appstream-data)
at the revision pinned in `flake.lock`. NXC uses its `appstream-data-all`
output for icon and application metadata lookup. The upstream project states
that metadata and icons follow the `metadata_license` in each metainfo record;
the upstream repository's MIT license covers its code, not those individual
records and icons.

The AppStream data is fetched/built from the Flake input rather than committed
as an NXC source file. The contents of `metadata_license` are not a single
project-wide license. Build/distribution maintainers should inspect the actual
generated XML and preserve the applicable attribution and license information.
Use `python3 scripts/audit-appstream-licenses.py /path/to/appstream-data` to
summarize the license identifiers in a locally built output.

## Arch Linux pkgstats recommendation statistics

`frontend/scripts/update-recommendations.mjs` generates
`frontend/public/data/recommendations.json` from the public
[Arch Linux pkgstats API](https://pkgstats.archlinux.de/api/doc), using the
NXC-maintained package mapping in
`frontend/scripts/recommendation-package-map.json`. The generated JSON is a
local build/distribution artifact and is intentionally ignored by Git; it is
not committed or automatically pushed to this repository.

The API documentation inspected for this change does not state a clear
redistribution license for the statistics payload. The server source license
does not by itself establish the license of API response data. A distributor
who chooses to generate and bundle this file must independently confirm the
applicable terms and comply with any conditions. NXC's MIT license does not
cover or grant additional rights to the statistics.

If the file is not generated, the GUI uses the curated NXC package mapping
without displaying pkgstats popularity percentages. This keeps the core
catalog/recommendation UI usable without placing the third-party statistics
dataset in the source tree.

## Runtime Iconify icons

The GUI can request icon artwork at runtime from Iconify, restricted in code to
the `logos` and `simple-icons` collections. Iconify's collection list
identifies `logos` as CC0 and `simple-icons` as CC0 1.0. Those licenses do
not grant trademark rights in the represented brands. NXC does not package the
Iconify service; a distributor should retain relevant source/attribution
information and review brand-specific terms when redistributing icon artwork.

- [Iconify collection license list](https://github.com/iconify/icon-sets/blob/master/collections.md)
- [Simple Icons disclaimer](https://github.com/simple-icons/simple-icons/blob/develop/DISCLAIMER.md)

## Geist Variable font

The GUI imports `@fontsource-variable/geist`. The Geist font software is
licensed under the SIL Open Font License 1.1. The corresponding license text
is copied to `frontend/public/licenses/OFL-1.1.txt` so it is included in the
Vite output and can accompany a GUI distribution.

- [Geist font source and license](https://github.com/google/fonts/tree/main/ofl/geist)
- [SIL Open Font License 1.1](https://openfontlicense.org)

## Other direct assets and package dependencies

- `frontend/src-tauri/icons/`, `frontend/public/favicon.svg`,
  `frontend/public/icons.svg`, and `frontend/src/assets/hero.png` contain
  binary or SVG assets whose original author/license could not be established
  from the repository metadata during this review. Do not treat those assets as
  confirmed MIT-licensed material until their provenance is recorded.
- Cargo and pnpm dependencies retain their own licenses. Their licenses are
  not changed by NXC's MIT license. Before producing a release, generate a
  dependency-license report from the exact lockfiles and include the license
  texts/notices required for the dependencies actually shipped in the bundle.

The unknown-provenance assets and the pkgstats data terms are release-review
items; this document intentionally does not mark them as cleared.
