use std::{
    fs::File,
    path::Path,
    sync::Mutex,
    io::BufReader,
    collections::{HashMap, HashSet},
};

use binread::{
    BinRead,
    FilePtr64,
    BinResult,
    BinReaderExt,
    io::Cursor,
};

use crate::{Hash40, FileNode, FileSystem, CompressedFileSystem};
use crate::hash_labels::HashLabels;

pub trait SeekRead: std::io::Read + std::io::Seek {}
impl<R: std::io::Read + std::io::Seek> SeekRead for R {}

#[derive(BinRead)]
#[br(magic = 0xABCDEF9876543210_u64)]
pub struct ArcFile {
    pub stream_section_offset: u64,
    pub file_section_offset: u64,
    pub shared_section_offset: u64,

    #[br(parse_with = FilePtr64::parse)]
    #[br(map = |x: CompressedFileSystem| x.0)]
    pub file_system: FileSystem,
    pub patch_section: u64,

    #[br(calc = Mutex::new(Box::new(Cursor::new([])) as _))]
    pub reader: Mutex<Box<dyn SeekRead>>,

    #[cfg(feature = "dir-listing")]
    #[br(calc = generate_dir_listing(&file_system))]
    pub dirs: HashMap<Hash40, Vec<FileNode>>,
}

#[cfg(feature = "dir-listing")]
fn parents_of_dir(dir: Hash40, labels: &HashLabels) -> Option<Vec<(Hash40, FileNode)>> {
    let mut label = dir.label(&labels)?;
    let mut hashes = Vec::new();
    let mut last_hash = dir;

    while let Some(len) = label.trim_end_matches('/').rfind('/') {
        label = &label[..len];

        let hash = crate::hash40::hash40(label);
        hashes.push((hash, FileNode::Dir(last_hash)));
        last_hash = hash;
    }

    hashes.push((crate::hash40::hash40("/"), FileNode::Dir(last_hash)));

    Some(hashes)
}

#[cfg(feature = "dir-listing")]
fn dir_listing_flat<'a>(fs: &'a FileSystem, labels: &'a HashLabels) -> impl Iterator<Item = (Hash40, FileNode)> + 'a {
    let dirs: HashSet<_> = fs.file_paths.iter().map(|path| path.parent.hash40()).collect();

    // Generate parents for directories
    let dirs = dirs.into_iter()
        .filter_map(move |dir| parents_of_dir(dir, labels).map(|x| x.into_iter()))
        .flatten();

    // Generate parents for files
    fs.file_paths.iter()
        .map(|path| (path.parent.hash40(), FileNode::File(path.path.hash40())))
        .chain(dirs)
}

#[cfg(feature = "dir-listing")]
fn generate_dir_listing(fs: &FileSystem) -> HashMap<Hash40, Vec<FileNode>> {
    let mut dirs = HashMap::new();

    let labels = crate::hash_labels::GLOBAL_LABELS.read();
    for (parent, child) in dir_listing_flat(fs, &labels) {
        let listing = dirs.entry(parent).or_insert_with(Vec::new);
        match listing.binary_search(&child) {
            Ok(_) => (),
            Err(insert_point) => listing.insert(insert_point, child)
        }
    }

    dirs
}

impl ArcFile {
    pub fn open<P: AsRef<Path>>(path: P) -> BinResult<Self> {
        Self::from_reader(BufReader::new(File::open(path)?))
    }

    pub fn from_reader<R: SeekRead + 'static>(mut reader: R) -> BinResult<Self> {
        let arc: Self = reader.read_le()?;

        *arc.reader.lock().unwrap() = Box::new(reader);

        Ok(arc)
    }

    #[cfg(feature = "dir-listing")]
    pub fn get_dir_listing<Hash: Into<Hash40>>(&self, hash: Hash) -> Option<&[FileNode]> {
        self.dirs.get(&hash.into()).map(AsRef::as_ref)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn print_tree_hash(arc: &ArcFile, hash: Hash40, depth: usize) {
        for file in arc.get_dir_listing(hash).unwrap() {
            (0..depth).for_each(|_| print!("    "));
            match file {
                FileNode::File(file) => {
                    println!("L {}", file.global_label().unwrap_or_else(|| format!("{:#x}", file.as_u64())));
                }
                FileNode::Dir(dir) => {
                    println!("L {}", dir.global_label().unwrap_or_else(|| format!("{:#x}", dir.as_u64())));
                    //print_tree_hash(arc, *dir, depth + 1);
                }
            }
        }
    }

    fn print_tree(arc: &ArcFile, dir: &str) {
        println!("{}:", dir);
        print_tree_hash(arc, dir.into(), 1);
    }

    #[test]
    fn test_listing() {
        Hash40::set_global_labels_file("/home/jam/Downloads/hashes.txt");
        let arc = ArcFile::open("/home/jam/re/ult/900/data.arc").unwrap();

        print_tree(&arc, "/");
        //dbg!(arc.get_dir_listing("fighter/mario/model/body/c00/"));
    }
}