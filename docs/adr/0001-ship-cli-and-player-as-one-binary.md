# Ship CLI and desktop player as one binary

Vinyl ships the `vn` desktop executable from the internal `vn_cli` crate, containing both writer tooling and the Bevy player. This keeps installation and versioning simple for writers; the accepted cost is a larger binary and renderer/audio dependencies in every release.
