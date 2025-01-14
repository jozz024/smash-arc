use std::{fs::File, io::BufReader};

use binrw::{BinReaderExt, BinResult};

pub use crate::filesystem::*;
use crate::ArcFile;
use crate::SeekRead;

#[repr(C)]
#[derive(Debug)]
pub struct LoadedArc {
    pub magic: u64,
    pub stream_section_offset: u64,
    pub file_section_offset: u64,
    pub shared_section_offset: u64,
    pub file_system_offset: u64,
    /// Not too sure about that one
    pub file_system_search_offset: u64,
    pub padding: u64,
    pub uncompressed_fs: *const FileSystemHeader,
    pub fs_header: *const FileSystemHeader,
    /// Not too sure about that one
    pub region_entry: u64,
    pub file_info_buckets: *const FileInfoBucket,
    pub file_hash_to_path_index: *const HashToIndex,
    pub file_paths: *const FilePath,
    pub file_info_indices: *const FileInfoIndex,
    pub dir_hash_to_info_index: *const HashToIndex,
    pub dir_infos: *mut DirInfo,
    pub folder_offsets: *mut DirectoryOffset,
    pub folder_child_hashes: *const HashToIndex,
    pub file_infos: *mut FileInfo,
    pub file_info_to_datas: *mut FileInfoToFileData,
    pub file_datas: *mut FileData,
    pub unk_section: u64,
    pub stream_header: *const StreamHeader,
    pub quick_dirs: *const QuickDir,
    pub stream_hash_to_entries: *const HashToIndex,
    pub stream_entries: *const StreamEntry,
    pub stream_file_indices: *const u32,
    pub stream_datas: *const StreamData,
    pub extra_buckets: *const FileInfoBucket,
    pub extra_entries: u64,
    pub extra_folder_offsets: *mut DirectoryOffset,
    // CppVector
    pub extra_entry_vector: [u64; 3],
    pub version: u32,
    pub extra_count: u32,
    pub loaded_file_system_search: *const LoadedSearchSection,
    // ...
}

impl LoadedArc {
    pub fn open() -> BinResult<ArcFile> {
        Self::from_reader(BufReader::new(File::open("rom:/data.arc")?))
    }

    pub fn from_reader<R: SeekRead + Send + 'static>(mut reader: R) -> BinResult<ArcFile> {
        let arc: ArcFile = reader.read_le()?;

        *arc.reader.lock().unwrap() = Box::new(reader);

        Ok(arc)
    }
}

#[repr(C)]
pub struct SearchSectionHeader {
    pub section_size: u32,
    // ..
}

#[repr(C)]
pub struct SearchSectionBody {
    pub folder_path_count: u32,
    pub path_indices_count: u32,
    pub path_count: u32,
}

#[repr(C)]
pub struct LoadedSearchSection {
    pub search_header: *const SearchSectionHeader,
    pub body: *const SearchSectionBody,
    pub folder_path_index: *const HashToIndex,
    pub folder_path_list: *const FolderPathListEntry,
    pub path_index: *const HashToIndex,
    pub path_list_indices: *const u32,
    pub path_list: *const PathListEntry, // ...
}

