# Third-party notices

The root [MIT License](./LICENSE) applies to NXC-authored code, including the
CLI/core, GUI source, Nix expressions, scripts, and documentation, except for
files or portions identified as third-party material below. It does not
relicense external services, datasets, application icons, trademarks, or
dependencies.

This document records known sources and outstanding checks. It is not a
blanket permission to redistribute third-party material. A release must be
reviewed against the exact assets and dependency versions it includes.

## AppStream data

NXC's Flake references
[snowfallorg/nixos-appstream-data](https://github.com/snowfallorg/nixos-appstream-data)
at the revision pinned in `flake.lock`. NXC uses its `appstream-data-all`
output for icon and application metadata lookup. The upstream project states
that metadata and icons follow the `metadata_license` in each metainfo record;
the upstream repository's MIT license covers its code, not those individual
records and icons.

The AppStream data is fetched/built from the Flake input rather than committed
as an NXC source file. Individual records can have different metadata licenses.
The presence of AppStream does not itself prevent NXC distribution; distributors
must review the actual generated data and retain the notices/attribution that
apply to the included records and icons.

## Arch Linux pkgstats recommendation statistics

The current source tree contains
`frontend/public/data/recommendations.json`, which is generated from the public
[Arch Linux pkgstats API](https://pkgstats.archlinux.de/api/doc). The workflow
`.github/workflows/update-recommendations.yml` periodically refreshes and
commits this snapshot. This PR intentionally does not change the dataset, the
workflow, or the GUI's current recommendation behavior.

The API documentation inspected for this change does not establish a clear
redistribution license for the statistics payload. The server source license
does not by itself establish the license of API response data. NXC's MIT
license does not cover or grant additional rights to these statistics. Before
including the snapshot in a release, confirm the applicable terms and comply
with any conditions. If those terms cannot be established, handle the
recommendation-data distribution design in a separate change rather than
silently treating the data as cleared.

## Runtime Iconify icons

The GUI can request icon artwork at runtime from Iconify, restricted in code to
the `logos` and `simple-icons` collections. Iconify's collection list
identifies `logos` as CC0 and `simple-icons` as CC0 1.0. Those licenses do
not grant trademark rights in the represented brands. NXC does not package the
Iconify service; distributors should retain relevant source/attribution
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
  `frontend/public/icons.svg`, `frontend/src/assets/hero.png`,
  `frontend/src/assets/react.svg`, and `frontend/src/assets/vite.svg` contain
  binary or SVG assets whose original author/license could not be established
  from the repository metadata during this review. Their provenance must be
  recorded before describing them as confirmed MIT-licensed material.
- Cargo and pnpm dependencies retain their own licenses. Their licenses are
  not changed by NXC's MIT license. Before producing a release, generate a
  dependency-license report from the exact lockfiles and include the license
  texts/notices required for the dependencies actually shipped in the bundle.

The pkgstats data terms, several direct assets, and the exact notices required
for release dependencies remain release-review items. This document does not
mark them as cleared.
