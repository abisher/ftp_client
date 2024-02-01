use std::path::PathBuf;

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
    is_dir=is_dir,rights=rights,links=1,owner="anonymous", group="anonymous",size=file_size,
    months=time.t)
}