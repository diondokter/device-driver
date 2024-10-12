# dd-manifest-tree

[![crates.io](https://img.shields.io/crates/v/dd-manifest-tree.svg)](https://crates.io/crates/dd-manifest-tree) [![Documentation](https://docs.rs/dd-manifest-tree/badge.svg)](https://docs.rs/dd-manifest-tree)

Part of the [`device-driver`](https://github.com/diondokter/device-driver) toolkit.

Provides a common way of dealing with parsed json, yaml and toml.
This is born from the observation that the crates `serde_json`, `yaml-rust2` and `toml` all have a `Value` type that is modeled in roughly the same way. This crate aims to provide a way of deserializing the various formats and providing a common `Value` interface to them.

The main goal is supporting the work of the device-driver toolkit. However, if this crate is open to receive extra features useful for others as long as they don't get in the way its main purpose.
