use std::cmp::Ordering;

use crate::state::FileInfo;

pub enum Sorting {
    Name,
    Ctime,
    Mtime,
    Size,
    Natural,
}

pub fn name(file1: &FileInfo, file2: &FileInfo) -> Ordering {
    file1.name.cmp(&file2.name)
}

pub fn ctime(file1: &FileInfo, file2: &FileInfo) -> Ordering {
    file1.ctime.cmp(&file2.ctime)
}

pub fn mtime(file1: &FileInfo, file2: &FileInfo) -> Ordering {
    file1.mtime.cmp(&file2.mtime)
}

pub fn size(file1: &FileInfo, file2: &FileInfo) -> Ordering {
    file1.size.cmp(&file2.size)
}

pub fn natural(file1: &FileInfo, file2: &FileInfo) -> Ordering {
    natord::compare(&file1.name.to_string_lossy(), &file2.name.to_string_lossy())
}

pub fn sort_files(files: &mut Vec<FileInfo>, sorting: &Sorting) {
    match sorting {
        Sorting::Name => files.sort_by(name),
        Sorting::Ctime => files.sort_by(ctime),
        Sorting::Mtime => files.sort_by(mtime),
        Sorting::Size => files.sort_by(size),
        Sorting::Natural => files.sort_by(natural),
    }
}
