#![allow(unused)]

use std::num::Wrapping;
use std::mem;
use b_error::BResult;
use std::io::{Read, Write, Seek};
use std::sync::Arc;
use static_assertions::{assert_obj_safe, assert_eq_size};

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
pub struct FsPath(Vec<u8>);

/// Using the locking policy described by SQLite:
///
///    https://sqlite.org/lockingv3.html
///
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum LockState {
    Unlocked,
    Shared,
    Reserved,
    Pending,
    Exclusive,
}

pub trait FileSystem {
    fn open(&self, path: &FsPath) -> BResult<Arc<dyn FileStream>>;
    fn change_lock_state(&self, path: &FsPath, type_: LockState) -> BResult<()>;
}

assert_obj_safe!(FileSystem);
    
pub trait FileStream: Read + Write + Seek { }

assert_obj_safe!(FileStream);

#[repr(packed)]
struct HeaderBlock {
    magic_number: [u8; 8],
    generation: Wrapping<u64>,
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

static MAGIC_NUMBER: &'static [u8; 8] = b"BLOCKSY!";

assert_eq_size!(HeaderBlock, [u8; BLOCK_SIZE]);

#[repr(packed)]
struct LockBytes {
    reserved_byte: u8,
    pending_byte: u8,
}

struct Checksum([u8; 32]);
