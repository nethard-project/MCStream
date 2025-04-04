// 这是一个简单的NBT处理工具，仅用于存储和传递NBT数据
// MCStream格式本身并不处理NBT内容，只是将其作为二进制数据保存
// 实际项目中可能需要更完整的NBT解析库

use crate::error::McStreamError;

/// 验证NBT数据是否有效（简单验证）
pub fn validate_nbt(data: &[u8]) -> Result<(), McStreamError> {
    if data.is_empty() {
        return Ok(());
    }

    // 检查第一个字节是否为有效的NBT标签类型
    match data[0] {
        // 有效的标签类型：0-12
        0..=12 => Ok(()),
        _ => Err(McStreamError::NbtError("无效的NBT标签类型".to_string())),
    }
}

/// NBT标签类型枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum NbtTagType {
    End = 0,
    Byte = 1,
    Short = 2,
    Int = 3,
    Long = 4,
    Float = 5,
    Double = 6,
    ByteArray = 7,
    String = 8,
    List = 9,
    Compound = 10,
    IntArray = 11,
    LongArray = 12,
}

impl TryFrom<u8> for NbtTagType {
    type Error = McStreamError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(NbtTagType::End),
            1 => Ok(NbtTagType::Byte),
            2 => Ok(NbtTagType::Short),
            3 => Ok(NbtTagType::Int),
            4 => Ok(NbtTagType::Long),
            5 => Ok(NbtTagType::Float),
            6 => Ok(NbtTagType::Double),
            7 => Ok(NbtTagType::ByteArray),
            8 => Ok(NbtTagType::String),
            9 => Ok(NbtTagType::List),
            10 => Ok(NbtTagType::Compound),
            11 => Ok(NbtTagType::IntArray),
            12 => Ok(NbtTagType::LongArray),
            _ => Err(McStreamError::NbtError(format!(
                "无效的NBT标签类型: {}",
                value
            ))),
        }
    }
}

/// 获取NBT数据的根标签类型
pub fn get_nbt_root_type(data: &[u8]) -> Result<NbtTagType, McStreamError> {
    if data.is_empty() {
        return Err(McStreamError::NbtError("NBT数据为空".to_string()));
    }

    NbtTagType::try_from(data[0])
}
