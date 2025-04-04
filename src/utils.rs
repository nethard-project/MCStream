use crate::error::McStreamError;
use sha2::{Sha256, Digest};
use std::io::{Read, Write, Seek};
use byteorder::{ReadBytesExt, WriteBytesExt};

/// 计算SHA-256哈希
pub fn calculate_sha256(data: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize().into()
}

/// 计算从文件起始到当前位置的SHA-256哈希
pub fn calculate_file_hash<R: Read + Seek>(reader: &mut R) -> Result<[u8; 32], McStreamError> {
    let current_pos = reader.stream_position()?;
    reader.seek(std::io::SeekFrom::Start(0))?;
    
    let mut buffer = Vec::new();
    reader.read_to_end(&mut buffer)?;
    
    reader.seek(std::io::SeekFrom::Start(current_pos))?;
    
    Ok(calculate_sha256(&buffer))
}

/// 验证文件哈希是否匹配
pub fn verify_file_hash<R: Read + Seek>(
    reader: &mut R, 
    expected_hash: &[u8; 32]
) -> Result<bool, McStreamError> {
    let actual_hash = calculate_file_hash(reader)?;
    Ok(&actual_hash == expected_hash)
}

/// 验证文件大小是否在4GB限制内
pub fn validate_file_size<S: Seek>(seeker: &mut S) -> Result<(), McStreamError> {
    let size = seeker.seek(std::io::SeekFrom::End(0))?;
    
    if size > 0xFFFFFFFF {
        return Err(McStreamError::FileTooLarge);
    }
    
    seeker.seek(std::io::SeekFrom::Start(0))?;
    
    Ok(())
}

/// 读取文件签名
pub fn read_signature<R: Read>(reader: &mut R) -> Result<Vec<u8>, McStreamError> {
    let signature_len = reader.read_u16::<byteorder::LittleEndian>()?;
    let mut signature = vec![0u8; signature_len as usize];
    reader.read_exact(&mut signature)?;
    
    Ok(signature)
}

/// 写入文件签名
pub fn write_signature<W: Write>(
    writer: &mut W, 
    signature: &[u8]
) -> Result<(), McStreamError> {
    if signature.len() > u16::MAX as usize {
        return Err(McStreamError::ValidationError("签名数据过大".to_string()));
    }
    
    writer.write_u16::<byteorder::LittleEndian>(signature.len() as u16)?;
    writer.write_all(signature)?;
    
    Ok(())
} 