# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.0] - 2024-06-06

### Added

- Support for parsing full payloads from multiple packets.
- Support for filtering packer and payload search by PID.
- Unit tests for a significant portion of operations.

### Changed

- TSReader to support generic inputs that implement the `Read` and `Seek` traits.
- Updated the `docs.rs` documentation comments to be more thorough.


## [0.1.0] - 2024-06-02

### Added

- Support for parsing transport stream packets from bytes.
- Support for reading payload data from transport stream packets.