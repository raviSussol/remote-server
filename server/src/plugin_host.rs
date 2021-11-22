extern crate libloading;

#[cfg(unix)]
use libloading::os::unix::Symbol as RawSymbol;
#[cfg(windows)]
use libloading::os::windows::Symbol as RawSymbol;
use libloading::Library;
use log::{error, info};
use plugin_api::DestroyPluginFunc;
use plugin_api::{
    CreatePluginFunc, MSupplyPlugin, OnLoadFunc, OnUnloadFunc, TestFunc, TestParameter,
    TestResponse,
};
use std::ffi::*;
use std::fs;

pub fn load_plugins() -> anyhow::Result<Vec<PluginHostProxy>> {
    let mut plugins = Vec::<PluginHostProxy>::new();
    for item in fs::read_dir("./plugins")? {
        let entry = match item {
            Err(_) => continue,
            Ok(entry) => entry,
        };

        let plugin = match entry.file_name().to_str() {
            None => continue,
            Some(name) => {
                if !name.ends_with(".so") && !name.ends_with(".dll") {
                    continue;
                }
                info!("Plugin found: {}", name);
                match load_plugin(name.to_string()) {
                    Err(e) => {
                        error!("Failed to load plugin: {}", e);
                        continue;
                    }
                    Ok(plugin) => plugin,
                }
            }
        };
        plugin.on_load();
        plugins.push(plugin);
    }
    Ok(plugins)
}

pub struct PluginMethods {
    pub create: RawSymbol<CreatePluginFunc>,
    pub destroy: RawSymbol<DestroyPluginFunc>,
    pub on_load: RawSymbol<OnLoadFunc>,
    pub on_unload: RawSymbol<OnUnloadFunc>,
    pub test: RawSymbol<TestFunc>,
}

impl PluginMethods {
    unsafe fn load(lib: &Library) -> Result<PluginMethods, libloading::Error> {
        Ok(PluginMethods {
            create: lib.get::<CreatePluginFunc>(b"create_plugin")?.into_raw(),
            destroy: lib.get::<DestroyPluginFunc>(b"destroy_plugin")?.into_raw(),
            on_load: lib.get::<OnLoadFunc>(b"on_load")?.into_raw(),
            on_unload: lib.get::<OnUnloadFunc>(b"on_unload")?.into_raw(),
            test: lib.get::<TestFunc>(b"test")?.into_raw(),
        })
    }
}

/// Delegates calls from the host to the actual implementation
pub struct PluginHostProxy {
    _filename: String,
    _lib: Library,
    methods: PluginMethods,
    plugin: *mut dyn MSupplyPlugin,
}

impl MSupplyPlugin for PluginHostProxy {
    fn on_load(&self) -> () {
        unsafe {
            let func = &self.methods.on_load;
            func(self.plugin as *mut c_void)
        }
    }

    fn on_unload(&self) -> () {
        unsafe {
            let func = &self.methods.on_unload;
            func(self.plugin as *mut c_void);
        }
    }

    fn test(&self, value: i32, str: String, complex_params: &TestParameter) -> TestResponse {
        let str = CString::new(str).unwrap();
        unsafe {
            let func = &self.methods.test;
            func(
                self.plugin as *mut c_void,
                value,
                str.as_ptr(),
                complex_params,
            )
        }
    }
}

impl Drop for PluginHostProxy {
    fn drop(&mut self) {
        unsafe {
            self.on_unload();

            let func = &self.methods.destroy;
            func(self.plugin as *mut c_void);
        }
    }
}

fn load_plugin(filename: String) -> anyhow::Result<PluginHostProxy> {
    unsafe {
        let lib = Library::new(&filename)?;
        let methods = PluginMethods::load(&lib)?;
        let create_func = &methods.create;
        let plugin = create_func();

        Ok(PluginHostProxy {
            _filename: filename,
            _lib: lib,
            methods,
            plugin: plugin,
        })
    }
}
