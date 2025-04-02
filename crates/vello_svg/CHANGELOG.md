# Changelog

<!-- Instructions

This changelog follows the patterns described here: <https://keepachangelog.com/en/1.0.0/>.

Subheadings to categorize changes are `added, changed, deprecated, removed, fixed, security`.

-->

The latest published Vello SVG release is [0.3.1](#031-2024-07-29) which was released on 2024-07-29.
You can find its changes [documented below](#031-2024-07-29).

## [Unreleased][]

This release has an [MSRV][] of 1.75.

### Changed

- Updated to usvg 0.43.0 ([#31] by [@Eoghanmc22])

## [0.3.1][] (2024-07-29)

This release has an [MSRV][] of 1.75.

### Added

- Support for rendering basic text ([#26] by [@nicoburns])

### Fixed

- Transform of nested SVGs ([#26] by [@nicoburns])

### Changed

- Updated to vello 0.2.1 ([#28] by [@waywardmonkeys])

## [0.3.0][] (2024-07-04)

This release has an [MSRV][] of 1.75.

### Added

- Added `vello_svg::Error`, which is returned by new functions that read text into a `usvg::Tree`. ([#18] by [@simbleau])
- Added `vello_svg::render`, which takes an svg string and renders to a new vello scene. ([#18] by [@simbleau])
- Added `vello_svg::append`, which takes an svg string and renders to a provided vello scene. ([#18] by [@simbleau])
- Added `vello_svg::append_with`, which takes an svg string and renders to a provided vello scene with and error handler. ([#18] by [@simbleau])
- Added `vello_svg::render_tree`, which takes a usvg::Tree and renders to a provided vello scene with and error handler. ([#18] by [@simbleau])

### Changed

- Updated to `vello` 0.2.0 and `usvg` 0.42 ([#18] by [@simbleau])
- Renamed `render_tree` to `append_tree` ([#18] by [@simbleau])
- Renamed `render_tree_with` to `append_tree_with` and removed the `Result<(), E>` return type for the error handler. ([#18] by [@simbleau])

### Removed

- All code and related profiling (`wgpu_profiler`) used in examples. ([#18] by [@simbleau])

## [0.2.0][] (2024-05-26)

This release has an [MSRV][] of 1.75.

### Added

- Make `util` module public and some minor doc fixes. ([#12] by [@nixon-voxell])

### Changed

- Updated `usvg` to 0.41 ([#6] by [@DasLixou])
- Disable `vello`'s default `wgpu` feature, and provide a `wgpu` passthrough feature to turn it back on. ([#10] by [@MarijnS95])

### Fixed

- The image viewBox is now properly translated ([#8] by [@simbleau])
- `vello_svg::render_tree_with` no longer takes a transform parameter. This is to make it consistent with the documentation and `vello_svg::render_tree`. ([#9] by [@simbleau])


### Removed

- MPL 2.0 is no longer a license requirement ([#9] by [@simbleau])
- The root image viewBox clipping was removed, to be added back at a later time ([#9] by [@simbleau])

## [0.1.0][] (2024-03-11)

This release has an [MSRV][] of 1.75.

- Initial release. ([#1] by [@simbleau])

[@Eoghanmc22]: https://github.com/Eoghanmc22
[@nicoburns]: https://github.com/nicoburns
[@waywardmonkeys]: https://github.com/waywardmonkeys
[@simbleau]: https://github.com/simbleau
[@nixon-voxell]: https://github.com/nixon-voxell
[@MarijnS95]: https://github.com/MarijnS95
[@DasLixou]: https://github.com/DasLixou

[#31]: https://github.com/linebender/vello_svg/pull/31
[#26]: https://github.com/linebender/vello_svg/pull/26
[#28]: https://github.com/linebender/vello_svg/pull/28
[#18]: https://github.com/linebender/vello_svg/pull/18
[#12]: https://github.com/linebender/vello_svg/pull/12
[#10]: https://github.com/linebender/vello_svg/pull/10
[#9]: https://github.com/linebender/vello_svg/pull/9
[#8]: https://github.com/linebender/vello_svg/pull/8
[#6]: https://github.com/linebender/vello_svg/pull/6
[#1]: https://github.com/linebender/vello_svg/pull/1

[Unreleased]: https://github.com/linebender/vello_svg/compare/v0.3.1...HEAD
[0.3.1]: https://github.com/linebender/vello_svg/compare/v0.3.0...v0.3.1
[0.3.0]: https://github.com/linebender/vello_svg/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/linebender/vello_svg/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/linebender/vello_svg/releases/tag/v0.1.0

[MSRV]: README.md#minimum-supported-rust-version-msrv
