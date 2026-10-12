use std::cmp::Ordering;

use crate::state::DiaryEntry;

pub enum Sorting {
    Date,
    Size,
}

pub fn date(file1: &DiaryEntry, file2: &DiaryEntry) -> Ordering {
    file1.date.cmp(&file2.date)
}

pub fn size(file1: &DiaryEntry, file2: &DiaryEntry) -> Ordering {
    file1.size.cmp(&file2.size)
}

pub fn sort_files(files: &mut Vec<DiaryEntry>, sorting: &Sorting) {
    match sorting {
        Sorting::Date => files.sort_by(date),
        Sorting::Size => files.sort_by(size),
    }
}
