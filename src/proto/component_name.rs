use crate::{unsafe_guid, Handle, Status};
use crate::proto::Protocol;
use crate::data_types::{Char16, Char8};
use core::ptr::NonNull;

type GetDriverName2Fn =
    extern "efiapi" fn(this: &ComponentName2, language: *const Char8, driver_name: *mut *const Char16) -> Status;

type GetControllerName2Fn =
    extern "efiapi" fn(this: &ComponentName2, controller: Handle, child: Option<NonNull<Handle>>, language: *const Char8, controller_name: *mut *const Char16) -> Status;

/// Wrapper for ComponentName protocol which allowed UEFI 2.3+
/// modules to query driver and controller names.
#[repr(C)]
#[unsafe_guid("6a7a5cff-e8d9-4f70-bada-75ab3025ce14")]
#[derive(Protocol)]
pub struct ComponentName2 {
    get_driver_name: GetDriverName2Fn,
    get_controller_name: GetControllerName2Fn,
    supported_languages: *const Char8
}

impl ComponentName2 {
    pub fn new(get_driver_name: GetDriverName2Fn, get_controller_name: GetControllerName2Fn, supported_languages: *const Char8) -> ComponentName2 {
        ComponentName2 {
            get_driver_name,
            get_controller_name,
            supported_languages
        }
    }
}

type GetDriverNameFn =
    extern "efiapi" fn(this: &ComponentName, language: *const Char8, driver_name: *mut *const Char16) -> Status;

type GetControllerNameFn =
    extern "efiapi" fn(this: &ComponentName, controller: Handle, child: Option<NonNull<Handle>>, language: *const Char8, controller_name: *mut *const Char16) -> Status;

/// Wrapper for ComponentName protocol which allowed UEFI 2.0+
/// modules to query driver and controller names.
#[repr(C)]
#[unsafe_guid("107a772c-d5e1-11d4-9a46-0090273fc14d")]
#[derive(Protocol)]
pub struct ComponentName {
    get_driver_name: GetDriverNameFn,
    get_controller_name: GetControllerNameFn,
    supported_languages: *const Char8
}

impl ComponentName {
    pub fn new(get_driver_name: GetDriverNameFn, get_controller_name: GetControllerNameFn, supported_languages: *const Char8) -> ComponentName {
        ComponentName {
            get_driver_name,
            get_controller_name,
            supported_languages,
        }
    }
}
