use std::{ffi::*, os::raw::c_char};

#[repr(C)]
#[derive(Debug)]
pub struct TestParameter {
    pub value: i32,
}

#[repr(C)]
#[derive(Debug)]
pub struct TestResponse {
    pub result: i32,
    pub result_struct: TestParameter,
}

pub trait MSupplyPlugin {
    fn on_load(&self) -> ();
    fn on_unload(&self) -> ();
    fn test(&self, value: i32, str: String, complex_params: &TestParameter) -> TestResponse;
}

pub type CreatePluginFunc = unsafe extern "C" fn() -> *mut dyn MSupplyPlugin;
pub type DestroyPluginFunc = unsafe extern "C" fn(plugin: *mut c_void) -> ();
pub type OnLoadFunc = unsafe extern "C" fn(plugin: *mut c_void) -> ();
pub type OnUnloadFunc = unsafe extern "C" fn(plugin: *mut c_void) -> ();
pub type TestFunc = unsafe extern "C" fn(
    plugin: *mut c_void,
    value: i32,
    str: *const c_char,
    complex_params: *const TestParameter,
) -> TestResponse;

#[macro_export]
macro_rules! export_plugin {
    ($PluginType:ty) => {
        #[no_mangle]
        pub extern "C" fn create_plugin() -> *mut $PluginType {
            Box::into_raw(<$PluginType>::new())
        }

        #[no_mangle]
        pub extern "C" fn destroy_plugin(plugin: *mut $PluginType) -> () {
            unsafe {
                // take ownership and drop it
                Box::from_raw(plugin);
            }
        }

        #[no_mangle]
        pub extern "C" fn on_load(plugin: *mut $PluginType) -> () {
            unsafe { plugin.as_ref().unwrap().on_load() }
        }

        #[no_mangle]
        pub extern "C" fn on_unload(plugin: *mut $PluginType) -> () {
            unsafe { plugin.as_ref().unwrap().on_unload() }
        }

        #[no_mangle]
        pub extern "C" fn test(
            plugin: *mut $PluginType,
            value: i32,
            string: *const std::os::raw::c_char,
            complex_params: *const plugin_api::TestParameter,
        ) -> plugin_api::TestResponse {
            unsafe {
                let string = CStr::from_ptr(string).to_str().unwrap().to_owned();
                plugin
                    .as_ref()
                    .unwrap()
                    .test(value, string, complex_params.as_ref().unwrap())
            }
        }
    };
}
