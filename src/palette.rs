use crate::error::McStreamError;
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::io::{Read, Write};
use std::collections::HashMap;

/// 验证调色板是否合法（不能包含空气方块）
pub fn validate_palette(palette: &[String]) -> Result<(), McStreamError> {
    if palette.iter().any(|id| id.contains("minecraft:air")) {
        return Err(McStreamError::AirInPalette);
    }
    
    Ok(())
}

/// 写入调色板到数据流
pub fn write_palette<W: Write>(writer: &mut W, palette: &[String]) -> Result<(), McStreamError> {
    // 验证调色板
    validate_palette(palette)?;
    
    // 调色板大小必须小于等于u16的最大值
    if palette.len() > u16::MAX as usize {
        return Err(McStreamError::PaletteError("调色板条目数超过上限".to_string()));
    }
    
    // 写入调色板大小（2字节，小端）
    writer.write_u16::<LittleEndian>(palette.len() as u16)?;
    
    // 写入每个调色板条目
    for entry in palette {
        // 块ID长度必须小于等于u16的最大值
        if entry.len() > u16::MAX as usize {
            return Err(McStreamError::PaletteError("调色板条目长度超过上限".to_string()));
        }
        
        // 写入字符串长度（2字节，小端）
        writer.write_u16::<LittleEndian>(entry.len() as u16)?;
        
        // 写入字符串内容
        writer.write_all(entry.as_bytes())?;
    }
    
    Ok(())
}

/// 从数据流读取调色板
pub fn read_palette<R: Read>(reader: &mut R) -> Result<Vec<String>, McStreamError> {
    // 读取调色板大小（2字节，小端）
    let palette_size = reader.read_u16::<LittleEndian>()?;
    
    // 读取每个调色板条目
    let mut palette = Vec::with_capacity(palette_size as usize);
    for _ in 0..palette_size {
        // 读取字符串长度（2字节，小端）
        let str_len = reader.read_u16::<LittleEndian>()?;
        
        // 读取字符串内容
        let mut buffer = vec![0u8; str_len as usize];
        reader.read_exact(&mut buffer)?;
        
        // 转换为UTF-8字符串
        let entry = String::from_utf8(buffer)
            .map_err(|_| McStreamError::PaletteError("非UTF-8编码的调色板条目".to_string()))?;
        
        // 验证不能包含空气方块
        if entry.contains("minecraft:air") {
            return Err(McStreamError::AirInPalette);
        }
        
        palette.push(entry);
    }
    
    Ok(palette)
}

/// 根据方块ID列表生成调色板
pub fn create_palette(block_ids: &[String]) -> Result<(Vec<String>, HashMap<String, u16>), McStreamError> {
    let mut unique_ids = Vec::new();
    let mut id_to_index = HashMap::new();
    
    for id in block_ids {
        // 跳过空气方块
        if id.contains("minecraft:air") {
            continue;
        }
        
        // 如果这个ID不在调色板中，添加它
        if !id_to_index.contains_key(id) {
            let index = unique_ids.len() as u16;
            unique_ids.push(id.clone());
            id_to_index.insert(id.clone(), index);
        }
    }
    
    // 检查调色板大小
    if unique_ids.len() > u16::MAX as usize {
        return Err(McStreamError::PaletteError("调色板条目数超过上限".to_string()));
    }
    
    Ok((unique_ids, id_to_index))
} 