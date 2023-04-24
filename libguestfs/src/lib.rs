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

    pub fn blockdev_getro<P: AsRef<Path>>(&self, device: P) -> Result<()> {
        let device = Self::path_to_cstring_guest(device)?;
        let out = unsafe { guestfs_blockdev_getro(self.handle, device.as_ptr()) };
        if out == 1 {
            Ok(())
        } else {
            self.check_error()
        }
    }

    pub fn blockdev_getsize64<P: AsRef<Path>>(&self, device: P) -> Result<i64> {
        let device = Self::path_to_cstring_guest(device)?;
        let out = unsafe { guestfs_blockdev_getsize64(self.handle, device.as_ptr()) };
        Ok(out)
    }

    pub fn blockdev_rereadpt<P: AsRef<Path>>(&self, device: P) -> Result<()> {
        let device = Self::path_to_cstring_guest(device)?;
        let out = unsafe { guestfs_blockdev_rereadpt(self.handle, device.as_ptr()) };
        if out == 0 {
            Ok(())
        } else {
            self.check_error()
        }
    }

    pub fn blockdev_setro<P: AsRef<Path>>(&self, device: P) -> Result<()> {
        let device = Self::path_to_cstring_guest(device)?;
        let out = unsafe { guestfs_blockdev_setro(self.handle, device.as_ptr()) };
        if out == 0 {
            Ok(())
        } else {
            self.check_error()
        }
    }

    pub fn blockdev_setrw<P: AsRef<Path>>(&self, device: P) -> Result<()> {
        let device = Self::path_to_cstring_guest(device)?;
        let out = unsafe { guestfs_blockdev_setrw(self.handle, device.as_ptr()) };
        if out == 0 {
            Ok(())
        } else {
            self.check_error()
        }
    }

    pub fn chmod<P: AsRef<Path>>(&self, mode: i32, path: P) -> Result<()> {
        let path = Self::path_to_cstring_guest(path)?;
        let out = unsafe { guestfs_chmod(self.handle, mode, path.as_ptr()) };
        if out == 0 {
            Ok(())
        } else {
            self.check_error()
        }
    }

    pub fn chown<P: AsRef<Path>>(&self, owner: i32, group: i32, path: P) -> Result<()> {
        let path = Self::path_to_cstring_guest(path)?;
        let out = unsafe { guestfs_chown(self.handle, owner, group, path.as_ptr()) };
        if out == 0 {
            Ok(())
        } else {
            self.check_error()
        }
    }

    pub fn command(&self, arguments: &[&str]) -> Result<String> {
        let mut args = Vec::new();
        for arg in arguments {
            let arg = Self::path_to_cstring_host(arg)?.as_ptr() as *mut i8;
            args.push(arg);
        }
        let args_ptr = args.as_ptr();
        let guestfs_out = unsafe { guestfs_command(self.handle, args_ptr) };
        let guestfs_out_cstr = unsafe { CStr::from_ptr(guestfs_out) };
        let out = guestfs_out_cstr.to_str()?.to_string();
        self.free(guestfs_out as *mut i8);
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

    // int guestfs_mke2fs_argv (guestfs_h *g, const char *device, const struct guestfs_mke2fs_argv *optargs);
    pub fn mke2fs_argv<P: AsRef<Path>>(
        &self,
        device: P,
        optargs: &libguestfs_sys::guestfs_mke2fs_argv,
    ) -> Result<()> {
        let device = Self::path_to_cstring_guest(device)?;
        let out = unsafe { guestfs_mke2fs_argv(self.handle, device.as_ptr(), optargs) };
        if out == 0 {
            Ok(())
        } else {
            self.check_error()
        }
    }

    pub fn mount<P: AsRef<Path>, Q: AsRef<Path>>(&self, mountable: P, mountpoint: Q) -> Result<()> {
        let mountable = Self::path_to_cstring_guest(mountable)?;
        let mountpoint = Self::path_to_cstring_guest(mountpoint)?;
        let out = unsafe { guestfs_mount(self.handle, mountable.as_ptr(), mountpoint.as_ptr()) };
        if out == 0 {
            Ok(())
        } else {
            self.check_error()
        }
    }

    pub fn statns<P: AsRef<Path>>(&self, path: P) -> Result<StatNS> {
        let path = Self::path_to_cstring_guest(path)?;
        let guest_stat = unsafe { guestfs_statns(self.handle, path.as_ptr()) };
        let stat = unsafe { StatNS::new(guest_stat) };
        self.free(guest_stat as *mut i8);
        Ok(stat)
    }

    pub fn touch<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let path = Self::path_to_cstring_guest(path)?;
        let out = unsafe { guestfs_touch(self.handle, path.as_ptr()) };
        if out == 0 {
            Ok(())
        } else {
            self.check_error()
        }
    }

    fn last_error(&self) -> Result<String> {
        let guestfs_out = unsafe { guestfs_last_error(self.handle) };
        let guestfs_out_cstr = unsafe { CStr::from_ptr(guestfs_out) };
        let out = guestfs_out_cstr.to_str()?.to_string();
        self.free(guestfs_out as *mut i8);
        Ok(out)
    }

    fn last_errno(&self) -> Result<i32> {
        let out = unsafe { guestfs_last_errno(self.handle) };
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

    fn check_error<T>(&self) -> Result<T> {
        let errno = self.last_errno()?;
        let message = self.last_error()?;
        dbg!(LibGuestFsError::GuestFsError {
            errno,
            message: message.clone()
        });
        Err(LibGuestFsError::GuestFsError { errno, message }.into())
    }
}

impl Drop for GuestFS {
    fn drop(&mut self) {
        unsafe { guestfs_close(self.handle) };
    }
}

pub struct StatNS {
    pub st_dev: i64,
    pub st_ino: i64,
    pub st_mode: i64,
    pub st_nlink: i64,
    pub st_uid: i64,
    pub st_gid: i64,
    pub st_rdev: i64,
    pub st_size: i64,
    pub st_blksize: i64,
    pub st_blocks: i64,
    pub st_atime_sec: i64,
    pub st_atime_nsec: i64,
    pub st_mtime_sec: i64,
    pub st_mtime_nsec: i64,
    pub st_ctime_sec: i64,
    pub st_ctime_nsec: i64,
    pub st_spare1: i64,
    pub st_spare2: i64,
    pub st_spare3: i64,
    pub st_spare4: i64,
    pub st_spare5: i64,
    pub st_spare6: i64,
}

impl StatNS {
    unsafe fn new(guestfs_stat: *const guestfs_statns) -> Self {
        let guestfs_stat = unsafe { *guestfs_stat };
        Self {
            st_dev: guestfs_stat.st_dev,
            st_ino: guestfs_stat.st_ino,
            st_mode: guestfs_stat.st_mode,
            st_nlink: guestfs_stat.st_nlink,
            st_uid: guestfs_stat.st_uid,
            st_gid: guestfs_stat.st_gid,
            st_rdev: guestfs_stat.st_rdev,
            st_size: guestfs_stat.st_size,
            st_blksize: guestfs_stat.st_blksize,
            st_blocks: guestfs_stat.st_blocks,
            st_atime_sec: guestfs_stat.st_atime_sec,
            st_atime_nsec: guestfs_stat.st_atime_nsec,
            st_mtime_sec: guestfs_stat.st_mtime_sec,
            st_mtime_nsec: guestfs_stat.st_mtime_nsec,
            st_ctime_sec: guestfs_stat.st_ctime_sec,
            st_ctime_nsec: guestfs_stat.st_ctime_nsec,
            st_spare1: guestfs_stat.st_spare1,
            st_spare2: guestfs_stat.st_spare2,
            st_spare3: guestfs_stat.st_spare3,
            st_spare4: guestfs_stat.st_spare4,
            st_spare5: guestfs_stat.st_spare5,
            st_spare6: guestfs_stat.st_spare6,
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum LibGuestFsError {
    #[error("libguestfs error: {message} (errno={errno})")]
    GuestFsError { errno: i32, message: String },
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;

    // LIBGUESTFS_DEBUG=1 LIBGUESTFS_TRACE=1 cargo test
    use super::*;

    use eyre::Result;

    fn empty_image() -> Result<TempImage> {
        TempImage::new("../fixtures/ext4.img")
    }

    #[test]
    fn test_list_partitions() -> Result<()> {
        let mut g = GuestFS::new();
        let img = empty_image()?;
        let path = &img.0;
        println!("add drive");
        g.add_drive(path)?;
        println!("launch");
        g.launch()?;
        println!("list");
        let partitions = g.list_partitions()?;
        println!("assert");
        assert_eq!(0, partitions.len());
        Ok(())
    }

    #[test]
    fn test_list_filesystems() -> Result<()> {
        let mut g = GuestFS::new();
        let img = empty_image()?;
        let path = &img.0;
        g.add_drive(path)?;
        g.launch()?;
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
        g.launch()?;
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
        g.launch()?;
        let groups = g.available_all_groups()?;
        dbg!(&groups);
        assert!(!groups.is_empty());
        assert!(groups.contains(&"extlinux".to_string()));

        Ok(())
    }

    #[test]
    fn test_base64_in() -> Result<()> {
        let mut g = GuestFS::new();
        let img = empty_image()?;
        let path = &img.0;
        g.add_drive(path)?;
        g.launch()?;
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

    #[test]
    fn test_chmod() -> Result<()> {
        let mut g = GuestFS::new();
        let img = empty_image()?;
        let path = &img.0;
        g.add_drive(path)?;
        g.launch()?;
        g.mount("/dev/sda", "/")?;

        let filename = "test_chmod.txt";
        let path = Path::join(Path::new("/"), filename);
        g.touch(&path)?;

        g.chmod(0o777, &path)?;

        let stat = g.statns(path)?;
        assert_eq!(0o100777, stat.st_mode);

        Ok(())
    }

    #[test]
    fn test_chown() -> Result<()> {
        let mut g = GuestFS::new();
        let img = empty_image()?;
        let path = &img.0;
        g.add_drive(path)?;
        g.launch()?;
        g.mount("/dev/sda", "/")?;

        let filename = "test_chown.txt";
        let path = Path::join(Path::new("/"), filename);
        g.touch(&path)?;
        g.chown(1000, 1000, &path)?;

        let stat = g.statns(path)?;
        assert_eq!(1000, stat.st_uid);
        assert_eq!(1000, stat.st_gid);

        Ok(())
    }

    #[test]
    fn test_command() -> Result<()> {
        let mut g = GuestFS::new();
        let img = empty_image()?;
        let path = &img.0;
        g.add_drive(path)?;
        g.launch()?;
        g.mount("/dev/sda", "/")?;

        let filename = "test_command.txt";
        let path = Path::join(Path::new("/"), filename);
        std::fs::File::create(&path)?;
        g.command(&["touch", path.to_str().unwrap()])?;

        let stat = g.statns(Path::join(Path::new("/"), filename))?;
        assert_eq!(0o100644, stat.st_mode);

        Ok(())
    }

    struct TempImage(PathBuf, TempDir);

    impl TempImage {
        fn new<P: Into<PathBuf>>(path: P) -> Result<Self> {
            let path = path.into();
            let tmp = TempDir::new()?;
            let mut tmp_path = tmp.path_view();
            tmp_path.push(path.file_name().unwrap());
            fs::copy(&path, &tmp_path)?;

            Ok(Self(tmp_path, tmp))
        }

        #[allow(unused)]
        fn path_view(&self) -> &Path {
            &self.0
        }
    }

    impl Drop for TempImage {
        fn drop(&mut self) {
            std::fs::remove_file(&self.0).unwrap();
        }
    }

    pub struct TempDir {
        path: PathBuf,
    }

    impl TempDir {
        pub fn new() -> Result<TempDir> {
            let mut path = std::env::temp_dir();
            path.push(format!("libguestfsrs-workdir-{}", rand::random::<u64>()));
            fs::create_dir_all(&path)?;

            Ok(TempDir { path })
        }

        pub fn path_view(&self) -> PathBuf {
            self.path.clone()
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            if self.path.exists() {
                std::fs::remove_dir_all(&self.path).unwrap();
            }
        }
    }
}
