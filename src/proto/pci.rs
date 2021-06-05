use crate::proto::Protocol;
use crate::{unsafe_guid, Status, Result};
use core::ffi::c_void;

#[repr(C)]
struct IoSpace {
    read: extern "efiapi" fn(this: &PciIO, width: IoWidth, bar: IoRegister, offset: u64, count: usize, buffer: *mut u8) -> Status,
    write: extern "efiapi" fn(this: &PciIO, width: IoWidth, bar: IoRegister, offset: u64, count: usize, buffer: *const u8) -> Status
}

#[repr(C)]
struct ConfigSpace {
    read: extern "efiapi" fn(this: &PciIO, width: IoWidth, offset: u32, count: usize, buffer: *mut u8) -> Status,
    write: extern "efiapi" fn(this: &PciIO, width: IoWidth, offset: u32, count: usize, buffer: *const u8) -> Status
}

#[repr(C)]
#[unsafe_guid("4cf5b200-68b8-4ca5-9eec-b23e3f50029a")]
#[derive(Protocol)]
pub struct PciIO {
    poll_mem: usize,
    poll_io: usize,
    mem: IoSpace,
    io: IoSpace,
    config: ConfigSpace,
    copy_mem: usize,
    map: extern "efiapi" fn(this: &PciIO, op: IoOperation, host_addr: *const c_void, num_bytes: &mut usize, device_addr: &mut u64, mapping: &mut *const c_void) -> Status,
    unmap: extern "efiapi" fn(this: &PciIO, mapping: *const c_void) -> Status,
    allocate_buffer: usize,
    free_buffer: usize,
    flush: extern "efiapi" fn(this: &PciIO) -> Status,
    get_location: usize,
    attributes: usize,
    get_bar_attributes: usize,
    set_bar_attributes: usize,
    rom_size_bytes: u64,
    rom_image: *const c_void,
}

pub trait ToIoWidth {
    const IO_WIDTH: IoWidth;
}

impl ToIoWidth for u8 {
    const IO_WIDTH: IoWidth = IoWidth::U8;
}

impl ToIoWidth for u16 {
    const IO_WIDTH: IoWidth = IoWidth::U16;
}

impl ToIoWidth for u32 {
    const IO_WIDTH: IoWidth = IoWidth::U32;
}

// Note deriving Debug is only necessary so that we will
// return Mapping back to the caller if it happened to fail
// to unmap the address
#[derive(Debug)]
pub struct Mapping {
    addr: *const c_void,
    device_addr: u64,
    size: usize
}

impl Mapping {
    pub fn size(&self) -> usize {
        self.size
    }

    pub fn device_address(&self) -> u64 {
        self.device_addr
    }

    pub fn as_ptr(&self) -> *const c_void {
        self.addr
    }
}

impl PciIO {
    pub fn read_config<T: ToIoWidth>(&self, offset: u32, buffer: &mut [T]) -> Result {
        (self.config.read)(self, T::IO_WIDTH, offset, buffer.len(), buffer.as_mut_ptr().cast())
            .into()
    }

    pub fn write_config<T: ToIoWidth>(&self, offset: u32, buffer: &[T]) -> Result {
        (self.config.write)(self, T::IO_WIDTH, offset, buffer.len(), buffer.as_ptr().cast())
            .into()
    }

    pub fn read_io<T: ToIoWidth>(&self, bar: IoRegister, offset: u64, buffer: &mut [T]) -> Result {
        (self.io.read)(self, T::IO_WIDTH, bar, offset, buffer.len(), buffer.as_mut_ptr().cast())
            .into()
    }

    pub fn write_io<T: ToIoWidth>(&self, bar: IoRegister, offset: u64, buffer: &[T]) -> Result {
        (self.io.write)(self, T::IO_WIDTH, bar, offset, buffer.len(), buffer.as_ptr().cast())
            .into()
    }

    pub fn map(&self, op: IoOperation, host_addr: *const c_void, num_bytes: usize) -> Result<Mapping> {
        let mut out_mapping = core::ptr::null();
        let mut out_num_bytes = num_bytes;
        let mut out_device_addr = 0;
        (self.map)(self, op, host_addr, &mut out_num_bytes, &mut out_device_addr, &mut out_mapping)
            .into_with_err(|_| {})
            .map(|completion| {
                // TBD: -- maybe check for alignment/null at least?
                completion.map(|_| Mapping {
                    addr: out_mapping,
                    device_addr: out_device_addr,
                    size: out_num_bytes
                })
            })
    }

    pub fn unmap(&self, mapping: Mapping) -> Result<(), Mapping> {
        (self.unmap)(self, mapping.addr)
            .into_with_err(|_| mapping)
    }

    pub fn flush(&self) -> Result {
        (self.flush)(self)
            .into()
    }
}

newtype_enum! {
    pub enum IoRegister: u8 => {
        R0 = 0,
        R1 = 1,
        R2 = 2,
        R3 = 3,
        R4 = 4,
        R5 = 5,
    }
}

#[repr(i32)]
pub enum IoOperation {
    BusMasterRead,
    BusMasterWrite,
    BusMasterCommonBuffer
}

newtype_enum! {
    pub enum IoIncrement: i32 => {
        LOOP   = 0,
        FIFO   = 4,
        FILL   = 8,
    }
}

newtype_enum! {
    // U8        = 0,
    // U16       = 1,
    // U32       = 2,
    // U64       = 3,
    // FIFO_U8   = 4,
    // FIFO_U16  = 5,
    // FIFO_U32  = 6,
    // FIFO_U64  = 7,
    // FILL_U8   = 8,
    // FILL_U16  = 9,
    // FILL_U32  = 10,
    // FILL_U64  = 11,
    pub enum IoWidth: i32 => {
        U8        = 0,
        U16       = 1,
        U32       = 2,
        U64       = 3,
    }
}
