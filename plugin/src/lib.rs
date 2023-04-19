#[cfg(feature = "config")]
pub mod geyser_config {
    pub use crate::config::PluginConfig;
}

#[cfg(not(feature = "config"))]
use solana_geyser_plugin_interface::geyser_plugin_interface::GeyserPlugin;

#[cfg(not(feature = "config"))]
mod builders;

mod config;

#[cfg(not(feature = "config"))]
mod events;

#[cfg(not(feature = "config"))]
mod executors;

#[cfg(not(feature = "config"))]
mod observers;

#[cfg(not(feature = "config"))]
mod plugin;

#[cfg(not(feature = "config"))]
mod pool_position;

#[cfg(not(feature = "config"))]
mod utils;

#[cfg(not(feature = "config"))]
pub use plugin::ClockworkPlugin;

#[cfg(not(feature = "config"))]
#[no_mangle]
#[allow(improper_ctypes_definitions)]
/// # Safety
///
/// The Solana validator and this plugin must be compiled with the same Rust compiler version and Solana core version.
/// Loading this plugin with mismatching versions is undefined behavior and will likely cause memory corruption.
pub unsafe extern "C" fn _create_plugin() -> *mut dyn GeyserPlugin {
    let plugin: Box<dyn GeyserPlugin> = Box::new(ClockworkPlugin::default());
    Box::into_raw(plugin)
}
