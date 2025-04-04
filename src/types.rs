use std::collections::HashMap;

/// 方块位置（全局坐标）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl BlockPos {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }
    
    /// 获取该位置所在的区块坐标
    pub fn chunk_pos(&self) -> ChunkPos {
        ChunkPos {
            x: self.x >> 4,
            z: self.z >> 4,
        }
    }
    
    /// 获取相对于所在区块的局部坐标
    pub fn local_pos(&self) -> LocalBlockPos {
        LocalBlockPos {
            x: (self.x & 0xF) as u8,
            y: (self.y + 64) as u16, // 编码Y坐标
            z: (self.z & 0xF) as u8,
        }
    }
}

/// 区块位置
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ChunkPos {
    pub x: i32,
    pub z: i32,
}

impl ChunkPos {
    pub fn new(x: i32, z: i32) -> Self {
        Self { x, z }
    }
}

/// 区块内的局部方块坐标
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LocalBlockPos {
    pub x: u8,  // 0-15
    pub y: u16, // 0-383 (编码后的Y坐标，实际为-64至319)
    pub z: u8,  // 0-15
}

impl LocalBlockPos {
    pub fn new(x: u8, y: u16, z: u8) -> Self {
        Self { x, y, z }
    }
    
    /// 将编码后的Y坐标转换为实际的Y坐标
    pub fn actual_y(&self) -> i32 {
        self.y as i32 - 64
    }
}

/// 区块数据
#[derive(Debug, Clone)]
pub struct ChunkData {
    pub pos: ChunkPos,
    pub palette: Vec<String>,  // 方块ID列表
    pub blocks: Vec<Block>,    // 非空气方块列表
}

/// 方块数据
#[derive(Debug, Clone)]
pub struct Block {
    pub palette_index: u16,    // 调色板索引
    pub pos: LocalBlockPos,    // 局部坐标
    pub nbt: Option<Vec<u8>>,  // NBT数据（如果有）
}

/// 区块索引条目
#[derive(Debug, Clone, Copy)]
pub struct ChunkIndexEntry {
    pub chunk_x: i32,
    pub chunk_z: i32,
    pub data_offset: u32,
    pub compressed_size: u32,
}

/// MCS格式头部
#[derive(Debug, Clone)]
pub struct McsHeader {
    pub version: u16,
    pub compression: u8,
    pub flags: u8,
    pub index_table_offset: u32,
}

/// 完整的MCS数据
#[derive(Debug, Clone)]
pub struct McsData {
    pub header: McsHeader,
    pub chunks: HashMap<ChunkPos, ChunkData>,
    pub data_hash: [u8; 32],
    pub signature: Option<Vec<u8>>,
} 