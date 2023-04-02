use std::ffi::CString;
use std::os::unix::prelude::OsStrExt;
use std::path::Path;

use eyre::{eyre, Result};
use libguestfs_sys::*;

pub struct GuestFS {
    handle: *mut guestfs_h,
    launched: bool,
}

impl GuestFS {
    #[allow(clippy::new_without_default)]
    pub fn new() -> GuestFS {
        let handle = unsafe { guestfs_create() };
        GuestFS {
            handle,
            launched: false,
        }
    }

    pub fn add_drive<P: AsRef<Path>>(&self, drive: P) -> Result<()> {
        let drive = CString::new(drive.as_ref().as_os_str().as_bytes())?;
        unsafe { guestfs_add_drive(self.handle, drive.as_ptr()) };
        Ok(())
    }

    pub fn launch(&mut self) -> Result<()> {
        if self.launched {
            return Err(eyre!("already launched!"));
        }
        unsafe { guestfs_launch(self.handle) };
        self.launched = true;
        Ok(())
    }

    pub fn list_partitions(&self) -> Result<Vec<String>> {
        let mut partitions = Vec::new();
        // let mut err = 0;
        let mut partitions_ptr = unsafe { guestfs_list_partitions(self.handle) };
        // if err != 0 {
        //     return Err(eyre!("error listing partitions"));
        // }
        while !partitions_ptr.is_null() {
            let partition = unsafe { CString::from_raw(*partitions_ptr) };
            partitions.push(partition.to_string_lossy().to_string());
            partitions_ptr = unsafe { partitions_ptr.offset(1) };
        }
        Ok(partitions)
    }

    pub fn set_error_handler(
        &self,
        eh: unsafe extern "C" fn(*mut guestfs_h, *mut ::std::ffi::c_void, *const i8),
    ) {
        unsafe { guestfs_set_error_handler(self.handle, Some(eh), std::ptr::null_mut()) };
    }
}

impl Drop for GuestFS {
    fn drop(&mut self) {
        unsafe { guestfs_close(self.handle) };
    }
}
