//! Toy driver for Rustlab 2023

use core::ops::Deref;

use kernel::{
    file::{self, SeekFrom},
    io_buffer::{IoBufferReader, IoBufferWriter},
    miscdev,
    prelude::*,
    sync::{Arc, Mutex, UniqueArc},
    ForeignOwnable,
};

module! {
    type: HelloRust,
    name: "hello_rust",
    author: "Rustlab 2023",
    description: "Toy driver for Rustlab 2023",
    license: "GPL",
}

struct HelloRust {
    _dev: Pin<Box<miscdev::Registration<File>>>,
}

impl kernel::Module for HelloRust {
    fn init(name: &'static CStr, _module: &'static ThisModule) -> Result<Self> {
        pr_info!("Hello Rust!\n");

        let buffer = SharedBuffer::try_new()?;

        Ok(Self {
            _dev: miscdev::Registration::<File>::new_pinned(fmt!("{name}"), buffer)?,
        })
    }
}

impl Drop for HelloRust {
    fn drop(&mut self) {
        pr_info!("Bye Rust!\n");
    }
}

struct SharedBuffer {
    inner: Mutex<Vec<u8>>,
}

impl SharedBuffer {
    fn try_new() -> Result<Arc<Self>> {
        let mut buffer = Pin::from(UniqueArc::try_new(Self {
            // SAFETY: `mutex_init!` is called below
            inner: unsafe { Mutex::new(Vec::new()) },
        })?);

        // SAFETY: `inner` is pinned when `buffer` is.
        let pinned = unsafe { buffer.as_mut().map_unchecked_mut(|b| &mut b.inner) };
        kernel::mutex_init!(pinned, "SharedState::inner");

        Ok(buffer.into())
    }
}

struct File;

#[vtable]
impl file::Operations for File {
    type Data = Arc<SharedBuffer>;
    type OpenData = Arc<SharedBuffer>;

    fn open(buffer: &Self::OpenData, _file: &file::File) -> Result<Self::Data> {
        pr_info!("Open\n");

        Ok(buffer.clone())
    }

    fn release(_data: Self::Data, _file: &file::File) {
        pr_info!("Close\n");
    }

    fn read(
        data: <Self::Data as ForeignOwnable>::Borrowed<'_>,
        _file: &file::File,
        writer: &mut impl IoBufferWriter,
        offset: u64,
    ) -> Result<usize> {
        let buffer = data.deref().inner.lock();

        let n = writer.len();
        let off = offset as usize;

        // Read starts from beyond the buffer length -> EOF
        if off >= buffer.len() {
            return Ok(0);
        }

        // Truncate `n` to the available bytes
        let n = usize::min(n, buffer.len() - off);

        pr_info!("Reading {n} bytes from buffer[{off}..{}]\n", off + n);
        writer.write_slice(&buffer[off..off + n])?;

        Ok(n)
    }

    fn write(
        data: <Self::Data as ForeignOwnable>::Borrowed<'_>,
        _file: &file::File,
        reader: &mut impl IoBufferReader,
        offset: u64,
    ) -> Result<usize> {
        let mut buffer = data.deref().inner.lock();

        let n = reader.len();
        let off = offset as usize;

        // Make sure we have enough room to push new bytes
        if off + n > buffer.len() {
            buffer.try_resize(off + n, 0)?;
        }

        pr_info!("Writing {n} bytes into buffer[{off}..{}]\n", off + n);
        reader.read_slice(&mut buffer[off..off + n])?;

        Ok(n)
    }
}
