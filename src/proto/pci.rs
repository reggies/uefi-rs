use crate::proto::Protocol;
use crate::{unsafe_guid, Status, Result};
use core::ffi::c_void;
use core::mem::MaybeUninit;

#[cfg(feature = "exts")]
use alloc_api::boxed::Box;

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

/// Marker trait for mapped buffer.
pub trait Mappable: Sized {}

/// Indicate appropriate I/O access size during memory-mapped I/O operations.
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

/// Representation of the bus relative memory address created by Map().
/// TBD: capture lifetime of the system memory object.
/// Note deriving Debug is only necessary so that we will
/// return Mapping back to the caller if it happened to fail
/// to unmap the address
#[derive(Debug)]
pub struct Mapping {
    addr: *const c_void,
    device_addr: u64,
    size: usize
}

impl Mapping {
    /// Captured size of the system memory object.
    pub fn size(&self) -> usize {
        self.size
    }

    /// Mapped bus relative address of the system memory object.
    pub fn device_address(&self) -> u64 {
        self.device_addr
    }
}

pub struct MappingEx<'a, B> {
    mapping: Option<Mapping>,
    pci: &'a PciIO,
    buffer: Box<B>
}

impl<'a, B> MappingEx<'a, B>
where B: Mappable + 'a, {
    /// Expose raw mapping object
    pub fn mapping(&self) -> &Mapping {
        self.mapping.as_ref().unwrap()
    }

    /// Mapped bus relative address of the system memory object.
    pub fn device_address(&self) -> u64 {
        self.mapping.as_ref().unwrap().device_address()
    }

    /// TBD:
    pub fn get_mut(&mut self) -> *mut B {
        &mut *self.buffer as *mut B
    }

    /// TBD
    pub fn get(&self) -> *const B {
        &*self.buffer as *const B
    }
}

impl<'a, B> Drop for MappingEx<'a, B> {
    fn drop(&mut self) {
        if let Some(mapping) = self.mapping.take() {
            self.pci
                .unmap(mapping)
                .expect("failed to unmap something");
            // On error, mapping is moved back into this scope
        }
    }
}

impl PciIO {
    /// Read PCI configuration space into a storage provided by a slice
    pub fn read_config<T: ToIoWidth>(&self, offset: u32, buffer: &mut [T]) -> Result {
        (self.config.read)(self, T::IO_WIDTH, offset, buffer.len(), buffer.as_mut_ptr().cast())
            .into()
    }

    /// Read PCI configuration space into a storage provided by an object of size T
    pub fn read_config_single<T: ToIoWidth>(&self, offset: u32) -> Result<T> {
        let mut buffer: MaybeUninit<T> = MaybeUninit::uninit();
        (self.config.read)(self, T::IO_WIDTH, offset, 1, buffer.as_mut_ptr().cast())
            .into_with_val(|| unsafe { buffer.assume_init() })
    }

    /// Write a number of objects into PCI configuration space
    pub fn write_config<T: ToIoWidth>(&self, offset: u32, buffer: &[T]) -> Result {
        (self.config.write)(self, T::IO_WIDTH, offset, buffer.len(), buffer.as_ptr().cast())
            .into()
    }

    /// Read I/O port space region into a storage provided by a slice
    pub fn read_io<T: ToIoWidth>(&self, bar: IoRegister, offset: u64, buffer: &mut [T]) -> Result {
        (self.io.read)(self, T::IO_WIDTH, bar, offset, buffer.len(), buffer.as_mut_ptr().cast())
            .into()
    }

    /// Read I/O port space region into an object storage
    pub fn read_io_single<T: ToIoWidth>(&self, bar: IoRegister, offset: u64) -> Result<T> {
        let mut buffer: MaybeUninit<T> = MaybeUninit::uninit();
        (self.io.read)(self, T::IO_WIDTH, bar, offset, 1, buffer.as_mut_ptr().cast())
            .into_with_val(|| unsafe { buffer.assume_init() })
    }

    /// Write a number of objects into I/O port space region
    pub fn write_io<T: ToIoWidth>(&self, bar: IoRegister, offset: u64, buffer: &[T]) -> Result {
        (self.io.write)(self, T::IO_WIDTH, bar, offset, buffer.len(), buffer.as_ptr().cast())
            .into()
    }

    /// Read memory-mapped I/O region into a storage provided by a slice
    pub fn read_mem<T: ToIoWidth>(&self, bar: IoRegister, offset: u64, buffer: &mut [T]) -> Result {
        (self.mem.read)(self, T::IO_WIDTH, bar, offset, buffer.len(), buffer.as_mut_ptr().cast())
            .into()
    }

    /// Write an object into memory-mapped I/O region
    pub fn read_mem_single<T: ToIoWidth>(&self, bar: IoRegister, offset: u64) -> Result<T> {
        let mut buffer: MaybeUninit<T> = MaybeUninit::uninit();
        (self.mem.read)(self, T::IO_WIDTH, bar, offset, 1, buffer.as_mut_ptr().cast())
            .into_with_val(|| unsafe { buffer.assume_init() })
    }

    /// Write number of objects into memory-mapped I/O region
    pub fn write_mem<T: ToIoWidth>(&self, bar: IoRegister, offset: u64, buffer: &[T]) -> Result {
        (self.mem.write)(self, T::IO_WIDTH, bar, offset, buffer.len(), buffer.as_ptr().cast())
            .into()
    }

    /// Create bus relative memory address for DMA operation.
    ///
    /// This functions allows an external device to access
    /// the buffer supplied for the duration of the DMA
    /// operation or until unmap() function is called. It is
    /// caller responsibility to ensure that the buffer
    /// lives long enough and has proper size to accomodate
    /// any access operation by the device.
    ///
    /// The caller must also make sure that buffer has
    /// appropriate cache coherency properties and
    /// synchronize any store operations on the buffer with
    /// the device to avoid simultaneous mutation.
    ///
    /// The caller must also make sure to wash their hands.
    pub unsafe fn map(&self, op: IoOperation, host_addr: *const c_void, num_bytes: usize) -> Result<Mapping> {
        let mut out_mapping = core::ptr::null();
        let mut out_num_bytes = num_bytes;
        let mut out_device_addr = 0;
        (self.map)(self, op, host_addr, &mut out_num_bytes, &mut out_device_addr, &mut out_mapping)
            .into_with_err(|_| {})
            .map(|completion| {
                // TBD: -- check out_num_bytes that it matches the request
                // TBD: -- maybe check for alignment/null at least?
                completion.map(|_| Mapping {
                    addr: out_mapping,
                    device_addr: out_device_addr,
                    size: out_num_bytes
                })
            })
    }

    #[cfg(feature = "exts")]
    /// Create bus relative memory address from an object.
    /// TBD: PCI_IO::AllocatePages for cache coherency
    pub fn map_ex<'a, T>(&'a self, op: IoOperation) -> Result<MappingEx<'a, T>>
    where T: Mappable + 'a, {
        let num_bytes = core::mem::size_of::<T>();
        let buffer = unsafe { Box::<T>::new_zeroed().assume_init() };
        let host_addr = &*buffer as *const T as *const c_void;
        unsafe {
            self.map(op, host_addr, num_bytes)
                .map(|completion| {
                    MappingEx {
                        mapping: Some(completion.ignore_warning()),
                        pci: self,
                        buffer
                    }.into()
                })
        }
    }

    /// Remove device memory mapping for the previously mapped system address.
    pub fn unmap(&self, mapping: Mapping) -> Result<(), Mapping> {
        (self.unmap)(self, mapping.addr)
            .into_with_err(|_| mapping)
    }

    /// Flushes all PCI controller specific transactions.
    pub fn flush(&self) -> Result {
        (self.flush)(self)
            .into()
    }
}

newtype_enum! {
    /// An index of the PCI Base Address Register.
    pub enum IoRegister: u8 => {
        R0 = 0,
        R1 = 1,
        R2 = 2,
        R3 = 3,
        R4 = 4,
        R5 = 5,
        PASS_THROUGH_BAR = 0xff,
    }
}

/// Indicator of the upcoming bus master operation.
/// The bus master is going to read or write to system memory.
/// Or both.
#[repr(i32)]
pub enum IoOperation {
    /// A read operation from system memory by a bus master.
    BusMasterRead,
    /// A write operation to system memory by a bus master.
    BusMasterWrite,
    /// Provides both read and write access to system memory
    /// by both the processor and a bus master. The buffer
    /// is coherent from both the processor’s and the bus
    /// master’s point of view.
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
