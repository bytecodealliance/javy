//! Shared Configuration for Javy.
//!
//! This crate serves as a central place to facilitate configuration sharing
//! between the Javy CLI and the Javy crate. It addresses the challenge of
//! passing configuration settings in environments where the Javy CLI commands
//! predominantly execute WebAssembly.
//!
//! The purpose of this crate is to consolidate configuration parameters,
//! ensuring consistent and accessible settings across both the CLI and the
//! crate. This approach simplifies the management of configuration settings and
//! enhances the integration between different components of the Javy ecosystem.
//!
//! Currently, this crate includes only a subset of the available configuration
//! options. The objective is to eventually encompass all configurable
//! parameters found in [javy::Config].
//!
//! The selection of the current configuration options was influenced by the
//! need to override non-standard defaults typically set during CLI invocations.
//! These defaults often do not align with the preferences of the CLI users.
//!
//! In gneneral this crate should be treated as an internal detail and
//! a contract between the CLI and the Javy crate.

use bitflags::bitflags;

bitflags! {
    pub struct Config: u32 {
        const OVERRIDE_JSON_PARSE_AND_STRINGIFY = 1;
        const JAVY_JSON = 1 << 1;
        const JAVY_STREAM_IO = 1 << 1;
        const REDIRECT_STDOUT_TO_STDERR = 1 << 2;
        const TEXT_ENCODING = 1 << 3;
    }
}
