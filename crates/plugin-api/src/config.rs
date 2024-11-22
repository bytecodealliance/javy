use std::ops::{Deref, DerefMut};

#[derive(Default)]
/// A configuration for the Javy plugin API.
pub struct Config {
    /// The runtime config.
    pub(crate) runtime_config: javy::Config,
    /// Whether to enable the experimental event loop.
    pub(crate) experimental_event_loop: bool,
}

impl Config {
    /// Whether to enable the experimental event loop.
    pub fn experimental_event_loop(&mut self, enabled: bool) -> &mut Self {
        self.experimental_event_loop = enabled;
        self
    }
}

impl Deref for Config {
    type Target = javy::Config;

    fn deref(&self) -> &Self::Target {
        &self.runtime_config
    }
}

impl DerefMut for Config {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.runtime_config
    }
}
