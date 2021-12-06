use crate::proto::Protocol;
use crate::{unsafe_guid, Status, Result};
use core::ffi::c_void;

#[repr(C)]
#[unsafe_guid("ffe06bdd-6107-46a6-7bb2-5a9c7ec5275c")]
#[derive(Protocol)]
pub struct AcpiTable {
    install_table: unsafe extern "efiapi" fn(
        this: &AcpiTable,
        buffer: *const c_void,
        buffer_size: usize,
        table_key: *mut usize) -> Status,
    uninstall_table: unsafe extern "efiapi" fn(
        this: &AcpiTable,
        table_key: usize) -> Status
}

impl AcpiTable {
    pub unsafe fn install_acpi_table(&self, buffer: *const c_void, buffer_size: usize) -> Result<usize> {
        let mut out_table_key = 0;
        (self.install_table)(self, buffer, buffer_size, &mut out_table_key)
            .into_with_val(|| out_table_key)
    }

    pub unsafe fn uninstall_acpi_table(&self, table_key: usize) -> Result {
        (self.uninstall_table)(self, table_key)
            .into()
    }
}
