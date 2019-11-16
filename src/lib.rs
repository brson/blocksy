#![allow(unused)]

use b_error::BResult;
use std::io::{Read, Write, Seek};
use std::sync::Arc;
use static_assertions::assert_obj_safe;

pub const BLOCK_SIZE: usize = 4096;

struct BlockBuf([u8; BLOCK_SIZE]);
struct BlockRef(u64);

trait BlockDevice: Send + Sync {
    fn read_block(&self, block: BlockRef) -> BResult<BlockBuf>;
    fn write_block(&self, block: BlockRef, buf: BlockBuf) -> BResult<()>;
    fn sync(&self) -> BResult<()>;
}

assert_obj_safe!(BlockDevice);

trait MetaBlock: Send + Sync {
    fn to_buf(&self) -> BResult<BlockBuf>;
}

assert_obj_safe!(MetaBlock);

trait MetaBlockFactory: Send + Sync {
    fn read(&self, block: BlockRef) -> BResult<Arc<dyn MetaBlock>>;
    fn write(&self, block: BlockRef, meta: &dyn MetaBlock) -> BResult<()>;
}

assert_obj_safe!(MetaBlockFactory);

#[derive(Eq, PartialEq, Copy, Clone)]
enum BlockState { Free, Used }

trait BlockMap: Send + Sync {
    fn get_state(&self, block: BlockRef) -> BResult<BlockState>;
    fn set_state(&self, block: BlockRef, st: BlockState) -> BResult<()>;
}

assert_obj_safe!(BlockMap);

struct Inode(u64);

trait BlockIndex: Send + Sync {
    type Iterator;

    fn new_file(&self) -> BResult<Inode>;
    fn extend_file(&self, blocks: u64) -> BResult<()>;
    fn iter_file_blocks(&self, node: Inode) -> BResult<Self::Iterator>;
    fn delete_file(&self, node: Inode) -> BResult<()>;
}

assert_obj_safe!(BlockIndex<Iterator = ()>);

trait FileSystem {
    type FileStream: FileStream;

    fn open(&mut self) -> Self::FileStream;
}

assert_obj_safe!(FileSystem<FileStream = Box<dyn FileStream>>);
    
trait FileStream: Read + Write + Seek { }

assert_obj_safe!(FileStream);
