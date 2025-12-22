use std::{
    fs,
    fs::Metadata,
    path::{MAIN_SEPARATOR_STR, Path},
};

#[cfg(unix)]
use std::os::unix::fs::{MetadataExt, PermissionsExt};

#[cfg(windows)]
use std::os::windows::fs::MetadataExt;

use uzers::{get_group_by_gid, get_user_by_uid};
use xattr;

pub fn get_platform_specific_info(metadata: &Metadata) -> (String, u64, String, String) {
    #[cfg(unix)]
    {
        let permissions = mode_to_string(metadata.mode());
        let hard_links = metadata.nlink();
        let (owner, group) = (metadata.uid(), metadata.gid());
        let (user_name, group_name) = get_user_and_group(owner, group);
        (permissions, hard_links, user_name, group_name)
    }

    #[cfg(windows)]
    {
        (
            String::from("rw-r--r--"),
            1,
            String::from("Owner"),
            String::from("Group"),
        )
    }
}

pub fn get_extended_attributes(path: &Path) -> String {
    #[cfg(unix)]
    {
        if has_extended_attributes(path) {
            String::from("@")
        } else {
            String::new()
        }
    }

    #[cfg(windows)]
    {
        String::new()
    }
}

pub fn get_symlink_info(metadata: &Metadata) -> String {
    #[cfg(unix)]
    {
        if metadata.file_type().is_symlink() {
            String::from("-> symlink")
        } else {
            String::new()
        }
    }

    #[cfg(windows)]
    {
        String::new()
    }
}

pub fn is_hidden(path: &Path) -> bool {
    #[cfg(unix)]
    {
        path.file_name()
            .map(|name| name.to_string_lossy().starts_with('.'))
            .unwrap_or(false)
    }

    #[cfg(target_os = "windows")]
    {
        use winapi::um::winnt::FILE_ATTRIBUTE_HIDDEN;

        fs::metadata(path)
            .map(|metadata| metadata.file_attributes() & FILE_ATTRIBUTE_HIDDEN != 0)
            .unwrap_or(false)
    }
}

pub fn classify(path: &Path) -> String {
    if path.is_dir() {
        return String::from(MAIN_SEPARATOR_STR);
    } else if path.is_symlink() {
        return String::from("@");
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::FileTypeExt;

        if let Ok(metadata) = path.metadata() {
            let file_type = metadata.file_type();

            if metadata.permissions().mode() & 0o111 != 0 {
                return String::from("*");
            } else if file_type.is_fifo() {
                return String::from("|");
            } else if file_type.is_socket() {
                return String::from("=");
            }
        }
    }

    #[cfg(windows)]
    {
        if let Some(ext) = path.extension() {
            let ext = ext.to_string_lossy().to_lowercase();
            if ["exe", "bat", "cmd", "com"].contains(&ext.as_str()) {
                return String::from("*");
            }
        }
    }

    String::from("")
}

pub fn get_total_blocks_in_directory(path: &Path) -> Option<u64> {
    let mut total_blocks = 0;
    let mut failed = false;

    if path.is_dir() {
        match fs::read_dir(path) {
            Ok(entries) => {
                for entry in entries.filter_map(Result::ok) {
                    match entry.metadata() {
                        Ok(metadata) => {
                            let blocks = metadata.blocks(); // Returns the number of 512-byte blocks
                            total_blocks += blocks;
                        }
                        Err(_) => failed = true,
                    }
                }
            }
            Err(_) => failed = true,
        }
    }

    if failed { None } else { Some(total_blocks) }
}

fn mode_to_string(mode: u32) -> String {
    let user = (mode >> 6) & 0b111;
    let group = (mode >> 3) & 0b111;
    let other = mode & 0b111;

    let mut perms_str = String::new();

    for &bits in &[user, group, other] {
        perms_str.push(if bits & 0b100 != 0 { 'r' } else { '-' });
        perms_str.push(if bits & 0b010 != 0 { 'w' } else { '-' });
        perms_str.push(if bits & 0b001 != 0 { 'x' } else { '-' });
    }

    perms_str
}

fn has_extended_attributes(path: &Path) -> bool {
    match xattr::list(path) {
        Ok(attrs) => attrs.count() > 0,
        Err(_) => false,
    }
}

fn get_user_and_group(uid: u32, gid: u32) -> (String, String) {
    let user = get_user_by_uid(uid)
        .map(|u| u.name().to_string_lossy().into_owned())
        .unwrap_or_else(|| format!("{}", uid));
    let group = get_group_by_gid(gid)
        .map(|g| g.name().to_string_lossy().into_owned())
        .unwrap_or_else(|| format!("{}", gid));

    (user, group)
}
