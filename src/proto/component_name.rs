use crate::prelude::*;
use crate::{unsafe_guid, Guid, Handle, Result, Status};
use crate::proto::Protocol;
use core::ffi::c_void;
use core::mem::transmute;
use crate::data_types::{CStr16, CStr8, Char16, Char8};
use core::str;

type GetDriverName2Fn =
    unsafe extern "efiapi" fn(this: &ComponentName2, language: *const Char8, driver_name: *mut *const Char16) -> Status;

type GetControllerName2Fn =
    unsafe extern "efiapi" fn(this: &ComponentName2, controller: Handle, child: Option<Handle>, language: *const Char8, controller_name: *mut *const Char16) -> Status;

#[repr(C)]
#[unsafe_guid("6a7a5cff-e8d9-4f70-bada-75ab3025ce14")]
#[derive(Protocol)]
pub struct ComponentName2 {
    get_driver_name: GetDriverName2Fn,
    get_controller_name: GetControllerName2Fn,
    supported_languages: *const Char8
}

type GetDriverNameFn =
    unsafe extern "efiapi" fn(this: &ComponentName, language: *const Char8, driver_name: *mut *const Char16) -> Status;

type GetControllerNameFn =
    unsafe extern "efiapi" fn(this: &ComponentName, controller: Handle, child: Option<Handle>, language: *const Char8, controller_name: *mut *const Char16) -> Status;

impl ComponentName2 {
    pub fn new(get_driver_name: GetDriverName2Fn, get_controller_name: GetControllerName2Fn, supported_languages: *const Char8) -> ComponentName2 {
        ComponentName2 {
            get_driver_name,
            get_controller_name,
            supported_languages
        }
    }
}

#[repr(C)]
#[unsafe_guid("107a772c-d5e1-11d4-9a46-0090273fc14d")]
#[derive(Protocol)]
pub struct ComponentName {
    get_driver_name: GetDriverNameFn,
    get_controller_name: GetControllerNameFn,
    supported_languages: *const Char8
}
