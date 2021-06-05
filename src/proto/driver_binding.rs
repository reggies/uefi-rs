use crate::{unsafe_guid, Handle, Status};
use crate::proto::device_path::DevicePath;
use crate::proto::Protocol;

/// Start routine
type StartFn = extern "efiapi" fn(this: &DriverBinding, controller: Handle, remaining_path: *mut DevicePath) -> Status;

/// Supported routine
type SupportedFn = extern "efiapi" fn(this: &DriverBinding, controller: Handle, remaining_path: *mut DevicePath) -> Status;

/// Stop routine
type StopFn = extern "efiapi" fn(this: &DriverBinding, controller: Handle, num_child_controller: usize, child_controller: *mut Handle) -> Status;

#[repr(C)]
#[unsafe_guid("18a031ab-b443-4d1a-a5c0-0c09261e9f71")]
#[derive(Protocol)]
pub struct DriverBinding {
    supported: SupportedFn,
    start: StartFn,
    stop: StopFn,
    version: u32,
    image_handle: Handle,
    driver_binding_handle: Handle
}

impl DriverBinding {
    pub fn new(start: StartFn, supported: SupportedFn, stop: StopFn, version: u32, image_handle: Handle, driver_binding_handle: Handle) -> DriverBinding {
        DriverBinding {
            start,
            supported,
            stop,
            version,
            image_handle,
            driver_binding_handle
        }
    }

    pub fn driver_handle(&self) -> Handle {
        self.driver_binding_handle
    }
}
