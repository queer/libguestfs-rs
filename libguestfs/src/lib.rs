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

    pub fn add_drive<P: AsRef<Path>>(&self, drive: P) -> Result<i32> {
        let drive = Self::path_to_cstring_host(drive)?;
        let out = unsafe { guestfs_add_drive(self.handle, drive.as_ptr()) };
        Ok(out)
    }

    pub fn add_drive_ro<P: AsRef<Path>>(&self, drive: P) -> Result<i32> {
        let drive = Self::path_to_cstring_host(drive.as_ref())?;
        let out = unsafe { guestfs_add_drive_ro(self.handle, drive.as_ptr()) };
        Ok(out)
    }

    pub fn available_all_groups(&self) -> Result<Vec<String>> {
        let mut groups = Vec::new();
        let groups_ptr = unsafe { guestfs_available_all_groups(self.handle) };
        let string_count = self.count_strings(groups_ptr as *const *const i8);
        let guestfs_groups = unsafe { std::slice::from_raw_parts(groups_ptr, string_count) };

        for ptr in guestfs_groups {
            let group = unsafe { CStr::from_ptr(*ptr) };
            let group = group.to_str().unwrap();
            groups.push(group.to_string());
            self.free(*ptr);
        }

        self.free(groups_ptr as *mut i8);

        Ok(groups)
    }

    pub fn base64_in<P: AsRef<Path>, Q: AsRef<Path>>(
        &self,
        base64file: P,
        filename: Q,
    ) -> Result<i32> {
        let base64file = Self::path_to_cstring_host(base64file)?;
        let filename = Self::path_to_cstring_guest(filename)?;
        let out = unsafe { guestfs_base64_in(self.handle, base64file.as_ptr(), filename.as_ptr()) };
        Ok(out)
    }

    pub fn base64_out<P: AsRef<Path>, Q: AsRef<Path>>(
        &self,
        filename: P,
        base64file: Q,
    ) -> Result<i32> {
        let filename = Self::path_to_cstring_guest(filename)?;
        let base64file = Self::path_to_cstring_host(base64file)?;
        let out =
            unsafe { guestfs_base64_out(self.handle, filename.as_ptr(), base64file.as_ptr()) };
        Ok(out)
    }

    pub fn launch(&mut self) -> Result<i32> {
        if self.launched {
            return Err(eyre!("already launched!"));
        }
        let out = unsafe { guestfs_launch(self.handle) };
        self.launched = true;
        Ok(out)
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

    pub fn mount<P: AsRef<Path>, Q: AsRef<Path>>(
        &self,
        mountable: P,
        mountpoint: Q,
    ) -> Result<i32> {
        let mountable = Self::path_to_cstring_guest(mountable)?;
        let mountpoint = Self::path_to_cstring_guest(mountpoint)?;
        let out = unsafe { guestfs_mount(self.handle, mountable.as_ptr(), mountpoint.as_ptr()) };
        Ok(out)
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

    fn path_to_cstring_host<P: AsRef<Path>>(path: P) -> Result<CString> {
        let path = path.as_ref().canonicalize()?;
        let path = path.as_os_str().as_bytes();
        let path = CString::new(path)?;
        Ok(path)
    }

    fn path_to_cstring_guest<P: AsRef<Path>>(path: P) -> Result<CString> {
        let path = path.as_ref();
        let path = path.as_os_str().as_bytes();
        let path = CString::new(path)?;
        Ok(path)
    }

    // I am going to commit acts of violence
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

    #[test]
    fn test_available_all_groups() -> Result<()> {
        let mut g = GuestFS::new();
        let path = Path::new("../fixtures/hello-world.ext4").canonicalize()?;
        g.add_drive(path)?;
        g.launch().unwrap();
        let groups = g.available_all_groups()?;
        dbg!(&groups);
        assert!(!groups.is_empty());
        assert!(groups.contains(&"extlinux".to_string()));

        Ok(())
    }

    #[test]
    fn test_base64_in() -> Result<()> {
        let mut g = GuestFS::new();
        let path = Path::new("../fixtures/ext4.img").canonicalize()?;
        g.add_drive(path)?;
        g.launch().unwrap();
        g.mount("/dev/sda", "/")?;

        let original_contents = "SGVsbG8gV29ybGQhCg==\n"; // Hello World!\n
        let filename = "test_base64_in-input.txt";
        std::fs::write(filename, original_contents)?;
        g.base64_in(Path::new(filename), Path::join(Path::new("/"), filename))?;

        std::fs::File::create("./test_base64_in.txt")?;
        g.base64_out(Path::join(Path::new("/"), filename), "./test_base64_in.txt")?;
        let contents = std::fs::read_to_string("./test_base64_in.txt")?;
        assert_eq!(original_contents, contents);

        Ok(())
    }
}
