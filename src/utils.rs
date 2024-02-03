use std::fs::Metadata;
use cfg_if::cfg_if;
use time::{OffsetDateTime};
use std::path::PathBuf;


cfg_if! {
    if #[cfg(windows)] {
        fn get_file_info(meta: &Metadata) -> (OffsetDateTime, u64) {
            use std::os::windows::prelude::*;
            (OffsetDateTime::from_unix_timestamp(meta.last_write_time() as i64).unwrap(),
            meta.file_size())
        }
    } else {
        // fn get_file_info(meta: &Metadata) -> (time::Tm, u64) {
        //     use std::os::unix::prelude::*;
        //     (time::at(time::TimeSpec::new(meta.mtime(), 0)),
        //     meta.size())
        // }
        fn get_file_info(meta: &Metadata) -> (OffsetDateTime, u64) {
            use std::os::unix::prelude::*;
            (OffsetDateTime::from_unix_timestamp(meta.mtime()).unwrap(), meta.size())
        }
}
}


const MONTHS: [&str; 12] = [
    "January", "February", "March", "April",
    "May", "June", "July", "August",
    "September", "October", "November", "December"
];

pub fn to_uppercase(data: &mut [u8]) {
    for byte in data {
        if *byte > 'a' as u8 && *byte <= 'z' as u8 {
            *byte -= 32;
        }
    }
}

pub fn add_file_info(path: PathBuf, out: &mut String) {
    let extra = if path.is_dir() { "/" } else { "" };
    let is_dir = if path.is_dir() { "d" } else { "-" };

    let meta = match std::fs::metadata(&path) {
        Ok(meta) => meta,
        _ => return,
    };

    let (time, file_size) = get_file_info(&meta);

    let path = match path.to_str() {
        Some(path) => match path.split("/").last() {
            Some(path) => path,
            _ => return,
        }
        _ => return,
    };

    let rights = if meta.permissions().readonly() {
        "r--r--r--"
    } else {
        "rw-rw-rw"
    };

    let file_str = format!("{is_dir} {rights} {links} {owner} {group} {size}\
    {month} {day} {hour}:{min} {path} {extra} \r\n",
                           is_dir = is_dir, rights = rights, links = 1, owner = "anonymous", group = "anonymous", size = file_size,
                           month = MONTHS[time.month() as usize], day = time.day(), hour = time.hour(),
                           min = time.minute(), path = path, extra = extra);
    println!("==> {:?}", &file_str);
}


#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    #[test]
    fn test_this() {
        use time;
        assert_eq!(time::Month::January as u16, 1u16);
    }

    #[test]
    fn test_dir() {
        let b: PathBuf = PathBuf::from("/usr/bin");
        assert!(b.is_dir())
    }

    #[test]
    fn test_canonize() {
        let b: PathBuf = PathBuf::from(r"C:\Users\maxal\");
        let c = b.canonicalize().unwrap();
        println!("{:?}",c);
        assert!(b.is_dir())
    }
}