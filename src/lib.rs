use libloading::{Library, Symbol};
use std::ffi::c_void;

#[repr(C)]
pub enum OodleFuzzSafe {
    No = 0,
    Yes = 1,
}

#[repr(C)]
pub enum OodleCheckCrc {
    No = 0,
    Yes = 1,
}

#[repr(C)]
pub enum OodleVerbosity {
    None = 0,
    Minimal = 1,
    Some = 2,
    Lots = 3,
}

#[repr(C)]
pub enum OodleDecodeThreadPhase {
    Phase1 = 1,
    Phase2 = 2,
    All = 3,
}

#[repr(C)]
pub enum OodleCompressor {
    Kraken = 8,
    Leviathan = 13,
    Mermaid = 9,
    Selkie = 11,
    Hydra = 12,
}

#[repr(C)]
pub enum OodleCompressionLevel {
    None = 0,
    SuperFast = 1,
    VeryFast = 2,
    Fast = 3,
    Normal = 4,
    Optimal1 = 5,
    Optimal2 = 6,
    Optimal3 = 7,
    Optimal4 = 8,
    Optimal5 = 9,
    HyperFast1 = -1,
    HyperFast2 = -2,
    HyperFast3 = -3,
    HyperFast4 = -4,
}

type CompressFn = unsafe extern "C" fn(
    compressor: OodleCompressor,
    input_ptr: *const c_void,
    input_size: usize,
    output_ptr: *mut c_void,
    level: OodleCompressionLevel,
    options: *mut c_void,
    scratch: *mut c_void,
    callback1: *mut c_void,
    callback2: *mut c_void,
    user_data1: usize,
    user_data2: usize,
) -> usize;

type DecompressFn = unsafe extern "C" fn(
    input_ptr: *const c_void,
    input_size: usize,
    output_ptr: *mut c_void,
    output_size: usize,
    fuzz_safe: OodleFuzzSafe,
    check_crc: OodleCheckCrc,
    verbosity: OodleVerbosity,
    decoder_mem: *mut c_void,
    decoder_mem_size: usize,
    callback1: *mut c_void,
    callback2: *mut c_void,
    callback3: *mut c_void,
    user_data1: usize,
    thread_phase: OodleDecodeThreadPhase,
    user_data2: usize,
) -> usize;

type GetCompressedBufferSizeNeededFn =
    unsafe extern "C" fn(compressor: OodleCompressor, input_size: usize, options: usize) -> usize;

struct OodleFunc {
    pub compress: Symbol<'static, CompressFn>,
    pub decompress: Symbol<'static, DecompressFn>,
    pub get_compressed_buffer_size_needed: Symbol<'static, GetCompressedBufferSizeNeededFn>,
}

impl OodleFunc {
    pub fn load(lib: &'static Library) -> Result<Self, libloading::Error> {
        unsafe {
            Ok(Self {
                compress: lib.get(b"OodleLZ_Compress")?,
                decompress: lib.get(b"OodleLZ_Decompress")?,
                get_compressed_buffer_size_needed: lib
                    .get(b"OodleLZ_GetCompressedBufferSizeNeeded")?,
            })
        }
    }
}

pub struct Oodle {
    _lib: &'static Library,
    funcs: OodleFunc,
}

pub enum Error {
    LibLoadError(libloading::Error),
    FunctionLoadError(libloading::Error),
}

impl Oodle {
    pub fn load(path: &str) -> Result<Self, Error> {
        let lib = Box::leak(Box::new(unsafe {
            Library::new(path).map_err(Error::LibLoadError)?
        }));
        let funcs = OodleFunc::load(lib).map_err(Error::FunctionLoadError)?;
        Ok(Self { _lib: lib, funcs })
    }

    pub fn decompress(&self, source: &[u8], dest: &mut [u8]) -> usize {
        unsafe {
            (self.funcs.decompress)(
                source.as_ptr() as *const c_void,
                source.len(),
                dest.as_mut_ptr() as *mut c_void,
                dest.len(),
                OodleFuzzSafe::Yes,
                OodleCheckCrc::No,
                OodleVerbosity::None,
                std::ptr::null_mut(),
                0,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                0,
                OodleDecodeThreadPhase::All,
                0,
            )
        }
    }
    pub fn compress(
        &self,
        compressor: OodleCompressor,
        level: OodleCompressionLevel,
        input: &[u8],
        output: &mut [u8],
    ) -> usize {
        unsafe {
            (self.funcs.compress)(
                compressor,
                input.as_ptr() as *const _,
                input.len(),
                output.as_mut_ptr() as *mut _,
                level,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                0,
                0,
            )
        }
    }

    pub fn get_compressed_buffer_size_needed<T: Into<usize>>(
        &self,
        compressor: OodleCompressor,
        input_len: T,
    ) -> usize {
        unsafe { (self.funcs.get_compressed_buffer_size_needed)(compressor, input_len.into(), 0) }
    }
}
