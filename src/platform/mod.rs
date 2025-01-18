use crate::error::Result;

pub mod linux;
pub mod windows;

#[cfg(target_os = "windows")]
pub type Process = windows::Process;
#[cfg(target_os = "linux")]
pub type Process = linux::Process;

#[cfg(target_os = "windows")]
pub const PROCESS_NAME: &str = "cs2.exe";
#[cfg(target_os = "linux")]
pub const PROCESS_NAME: &str = "cs2";

#[cfg(target_os = "windows")]
pub const DEFAULT_MODULES: [&'static str; 21] = [
    "cs2.exe",
    "client.dll",
    "engine2.dll",
    "schemasystem.dll",
    "animationsystem.dll",
    "rendersystemdx11.dll",
    "filesystem_stdio.dll",
    "inputsystem.dll",
    "materialsystem2.dll",
    "meshsystem.dll",
    "networksystem.dll",
    "panorama.dll",
    "panoramauiclient.dll",
    "resourcesystem.dll",
    "scenesystem.dll",
    "soundsystem.dll",
    "tier0.dll",
    "vphysics2.dll",
    "worldrenderer.dll",
    "matchmaking.dll",
    "server.dll"
];

#[cfg(target_os = "linux")]
pub const DEFAULT_MODULES: [&'static str; 21] = [
    "cs2",
    "libclient.so",
    "libengine2.so",
    "libschemasystem.so",
    "libanimationsystem.so",
    "librendersystemvulkan.so",
    "libfilesystem_stdio.so",
    "libinputsystem.so",
    "libmaterialsystem2.so",
    "libmeshsystem.so",
    "libnetworksystem.so",
    "libpanorama.so",
    "libpanoramauiclient.so",
    "libresourcesystem.so",
    "libscenesystem.so",
    "libsoundsystem.so",
    "libtier0.so",
    "libvphysics2.so",
    "libworldrenderer.so",
    "libmatchmaking.so",
    "libserver.so"
];

#[allow(dead_code)]
pub trait ProcessTrait {
    fn attach(&mut self) -> Result<()>;
    fn detach(&mut self) -> Result<()>;
    
    fn mem_read(&self, address: usize, data: &mut [u8]) -> Result<()>;
    fn mem_write(&self, address: usize, data: &[u8]) -> Result<()>;

    fn mod_find(&self, name: &str) -> Result<ProcessModule>;
}

pub struct ProcessModule {
    pub module_base: usize,
    pub module_size: usize
}
