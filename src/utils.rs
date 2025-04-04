use crate::error::McStreamError;
use std::io::Seek;

/// 验证文件大小是否在4GB限制内
pub fn validate_file_size<S: Seek>(seeker: &mut S) -> Result<(), McStreamError> {
    let size = seeker.seek(std::io::SeekFrom::End(0))?;

    if size > 0xFFFFFFFF {
        return Err(McStreamError::FileTooLarge);
    }

    seeker.seek(std::io::SeekFrom::Start(0))?;

    Ok(())
}
