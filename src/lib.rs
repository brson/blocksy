#![allow(unused)]

use std::num::Wrapping;
use std::mem;
use b_error::BResult;
use std::io::{Read, Write, Seek};
use std::sync::Arc;
use static_assertions::{assert_obj_safe, assert_eq_size};
use endian_type::LittleEndian;
use std::ops::Range;

pub const BLOCK_SIZE: usize = 4096;

pub struct BlockBuf(Vec<u8>);
pub struct BlockRef(u64);

pub trait BlockDevice: Send + Sync {
    fn read_block(&self, block: BlockRef) -> BResult<BlockBuf>;
    fn write_block(&self, block: BlockRef, buf: BlockBuf) -> BResult<()>;
    fn sync(&self) -> BResult<()>;
}

assert_obj_safe!(BlockDevice);

pub trait MetaBlock: Send + Sync {
    fn to_buf(&self) -> BResult<BlockBuf>;
}

assert_obj_safe!(MetaBlock);

pub trait MetaBlockFactory: Send + Sync {
    fn read(&self, block: BlockRef) -> BResult<Arc<dyn MetaBlock>>;
    fn write(&self, block: BlockRef, meta: &dyn MetaBlock) -> BResult<()>;
}

assert_obj_safe!(MetaBlockFactory);

#[derive(Eq, PartialEq, Copy, Clone)]
pub enum BlockState { Free, Used }

pub trait BlockMap: Send + Sync {
    fn get_state(&self, block: BlockRef) -> BResult<BlockState>;
    fn set_state(&self, block: BlockRef, st: BlockState) -> BResult<()>;
}

assert_obj_safe!(BlockMap);

pub struct Inode(u64);

pub trait BlockIndex: Send + Sync {
    fn new_file(&self) -> BResult<Inode>;
    fn extend_file(&self, blocks: u64) -> BResult<()>;
    fn iter_file_blocks(&self, node: Inode) -> BResult<Box<dyn BlockIterator>>;
    fn delete_file(&self, node: Inode) -> BResult<()>;
}

assert_obj_safe!(BlockIndex);

pub trait BlockIterator {
}

assert_obj_safe!(BlockIterator);

#[derive(Clone)]
pub struct FilePath(Vec<u8>);

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum LockType {
    Read, Write
}

pub trait FileSystem {
    fn open(&self, path: &FilePath) -> BResult<Arc<dyn FileHandle>>;
    fn lock(&self, path: &dyn FileHandle, range: Range<u64>) -> BResult<()>;
    fn unlock(&self, path: &dyn FileHandle, range: Range<u64>) -> BResult<()>;
}

assert_obj_safe!(FileSystem);

pub trait FileHandle: FileStream + Send + Sync { }
    
pub trait FileStream: Read + Write + Seek { }

assert_obj_safe!(FileStream);

#[repr(packed)]
struct HeaderBlock {
    magic_number: MagicNumber,
    generation: Wrapping<LittleEndian<u64>>,
    lock_bytes: LockBytes,
    _padding: [u8; HEADER_PADDING_BYTES],
    header_checksum: Checksum,
}

const HEADER_PADDING_BYTES: usize =
    BLOCK_SIZE -
    mem::size_of::<[u8; 8]>() -
    mem::size_of::<Wrapping<u64>>() -
    mem::size_of::<LockBytes>() -
    mem::size_of::<Checksum>();

static MAGIC_NUMBER: MagicNumber = MagicNumber(*b"BLOCKSY!");

struct MagicNumber([u8; 8]);

assert_eq_size!(HeaderBlock, [u8; BLOCK_SIZE]);

#[repr(packed)]
struct LockBytes {
    reserved_byte: u8,
    pending_byte: u8,
}

struct Checksum([u8; 32]);
