use std::ffi::{CStr, CString};
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
        let partitions_ptr = unsafe { guestfs_list_partitions(self.handle) };
        let string_count = self.count_strings(partitions_ptr as *const *const i8);
        println!("string count: {}", string_count);

        let parts = unsafe { std::slice::from_raw_parts(partitions_ptr, string_count) };

        for ptr in parts {
            let part = unsafe { CStr::from_ptr(*ptr) };
            let part = part.to_str()?;
            partitions.push(part.to_string());
            self.free(*ptr);
        }

        self.free(partitions_ptr as *mut i8);

        Ok(partitions)
    }

    pub fn list_filesystems(&self) -> Result<Vec<String>> {
        let mut filesystems = Vec::new();
        let filesystems_ptr = unsafe { guestfs_list_filesystems(self.handle) };
        let string_count = self.count_strings(filesystems_ptr as *const *const i8);
        let guestfs_filesystems =
            unsafe { std::slice::from_raw_parts(filesystems_ptr, string_count) };

        for ptr in guestfs_filesystems {
            let filesystem = unsafe { CStr::from_ptr(*ptr) };
            let filesystem = filesystem.to_str().unwrap();
            filesystems.push(filesystem.to_string());
            self.free(*ptr);
        }

        self.free(filesystems_ptr as *mut i8);

        Ok(filesystems)
    }

    fn count_strings(&self, pointer: *const *const i8) -> usize {
        let mut count = 0;
        let mut pointer = pointer;
        while !pointer.is_null() {
            if (unsafe { *pointer }).is_null() {
                break;
            }
            count += 1;
            pointer = unsafe { pointer.add(1) };
        }
        count
    }

    fn free(&self, pointer: *mut i8) {
        unsafe {
            std::ptr::drop_in_place(pointer);
            std::alloc::dealloc(
                pointer as *mut u8,
                std::alloc::Layout::from_size_align_unchecked(
                    std::mem::size_of::<i8>(),
                    std::mem::align_of::<i8>(),
                ),
            );
        }
    }
}

impl Drop for GuestFS {
    fn drop(&mut self) {
        println!("dropping!");
        unsafe { guestfs_close(self.handle) };
        println!("dropped!");
    }
}

#[cfg(test)]
mod tests {
    // LIBGUESTFS_DEBUG=1 LIBGUESTFS_TRACE=1 cargo test
    use super::*;

    use eyre::Result;

    #[test]
    fn test_list_partitions() -> Result<()> {
        // TODO: Make this have
        let mut g = GuestFS::new();
        let path = Path::new("../fixtures/ext4.img").canonicalize()?;
        println!("add drive");
        g.add_drive(path)?;
        println!("launch");
        g.launch().unwrap();
        println!("list");
        let partitions = g.list_partitions()?;
        println!("assert");
        assert_eq!(0, partitions.len());
        Ok(())
    }

    #[test]
    fn test_list_filesystems() -> Result<()> {
        let mut g = GuestFS::new();
        let path = Path::new("../fixtures/ext4.img").canonicalize()?;
        g.add_drive(path)?;
        g.launch().unwrap();
        let filesystems = g.list_filesystems()?;
        dbg!(&filesystems);
        assert_eq!(2, filesystems.len());
        assert_eq!("/dev/sda", filesystems[0]);
        assert_eq!("ext4", filesystems[1]);

        Ok(())
    }

    #[test]
    fn test_list_filesystems_hello_world() -> Result<()> {
        let mut g = GuestFS::new();
        let path = Path::new("../fixtures/hello-world.ext4").canonicalize()?;
        g.add_drive(path)?;
        g.launch().unwrap();
        let filesystems = g.list_filesystems()?;
        dbg!(&filesystems);
        assert_eq!(2, filesystems.len());
        assert_eq!("/dev/sda", filesystems[0]);
        assert_eq!("ext4", filesystems[1]);

        Ok(())
    }
}
