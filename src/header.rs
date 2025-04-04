use crate::{MCS_MAGIC, MCS_VERSION, CompressionType, error::McStreamError, types::McsHeader};
use byteorder::{BigEndian, LittleEndian, ReadBytesExt, WriteBytesExt};
use std::io::{Read, Write, Seek, SeekFrom};

/// 写入MCS文件头部
pub fn write_header<W: Write>(writer: &mut W, compression: CompressionType, has_signature: bool) -> Result<(), McStreamError> {
    writer.write_all(MCS_MAGIC)?;
    writer.write_u16::<BigEndian>(MCS_VERSION)?;
    writer.write_u8(compression as u8)?;
    
    let flags = if has_signature { 0x01 } else { 0x00 };
    writer.write_u8(flags)?;
    
    // 区块索引表偏移，临时写入0
    writer.write_u32::<LittleEndian>(0)?;
    
    // 预留字段
    writer.write_all(&[0; 4])?;
    
    Ok(())
}

/// 读取MCS文件头部
pub fn read_header<R: Read>(reader: &mut R) -> Result<McsHeader, McStreamError> {
    let mut magic = [0u8; 8];
    reader.read_exact(&mut magic)?;
    
    if magic != *MCS_MAGIC {
        return Err(McStreamError::InvalidMagic);
    }
    
    let version = reader.read_u16::<BigEndian>()?;
    if version != MCS_VERSION {
        return Err(McStreamError::UnsupportedVersion(version));
    }
    
    let compression = reader.read_u8()?;
    if compression > 3 {
        return Err(McStreamError::UnsupportedCompression(compression));
    }
    
    let flags = reader.read_u8()?;
    let index_table_offset = reader.read_u32::<LittleEndian>()?;
    
    let mut reserved = [0u8; 4];
    reader.read_exact(&mut reserved)?;
    
    Ok(McsHeader {
        version,
        compression,
        flags,
        index_table_offset,
    })
}

/// 更新区块索引表偏移值
pub fn update_index_table_offset<W: Write + Seek>(writer: &mut W, offset: u32) -> Result<(), McStreamError> {
    writer.seek(SeekFrom::Start(0x0C))?;
    writer.write_u32::<LittleEndian>(offset)?;
    Ok(())
} 