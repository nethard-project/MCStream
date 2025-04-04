use clap::{Parser, Subcommand};
use mcstream::{
    CompressionType,
    McsEncoder,
    McsDecoder,
    McStreamError,
};
use std::path::PathBuf;
use std::fs::File;
use std::io::{BufReader, BufWriter};

/// MCStream格式命令行工具 - Minecraft建筑高效二进制流式存储格式
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// 将Minecraft建筑数据打包为MCS格式
    Pack {
        /// 输入文件路径（JSON格式）
        #[arg(short, long)]
        input: PathBuf,
        
        /// 输出MCS文件路径
        #[arg(short, long)]
        output: PathBuf,
        
        /// 压缩算法: none, zstd, lz4, brotli
        #[arg(short, long, default_value = "zstd")]
        compression: String,
    },
    
    /// 将MCS格式文件解包为Minecraft建筑数据
    Unpack {
        /// 输入MCS文件路径
        #[arg(short, long)]
        input: PathBuf,
        
        /// 输出文件路径（JSON格式）
        #[arg(short, long)]
        output: PathBuf,
    },
    
    /// 查看MCS文件信息
    Info {
        /// MCS文件路径
        #[arg(short, long)]
        file: PathBuf,
        
        /// 是否详细输出
        #[arg(short, long)]
        verbose: bool,
    },
}

fn main() -> Result<(), McStreamError> {
    let cli = Cli::parse();
    
    match &cli.command {
        Commands::Pack { input, output, compression } => {
            println!("输入文件: {}", input.display());
            println!("输出文件: {}", output.display());
            
            // 检查输入文件是否存在
            if !input.exists() {
                return Err(McStreamError::Io(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    format!("输入文件不存在: {}", input.display())
                )));
            }
            
            // 确保输出目录存在
            if let Some(parent) = output.parent() {
                println!("检查输出目录: {}", parent.display());
                std::fs::create_dir_all(parent)?;
                println!("检查目录写入权限...");
            }
            
            let compression_type = match compression.to_lowercase().as_str() {
                "none" => CompressionType::None,
                "zstd" => CompressionType::Zstandard,
                "lz4" => CompressionType::LZ4, 
                "brotli" => CompressionType::Brotli,
                _ => {
                    println!("不支持的压缩算法: {}，使用默认的zstd", compression);
                    CompressionType::Zstandard
                }
            };
            
            println!("打包中...");
            match pack_json_to_mcs(input, output, compression_type) {
                Ok(_) => {
                    println!("打包完成: {}", output.display());
                    Ok(())
                },
                Err(e) => {
                    eprintln!("打包失败: {}", e);
                    if let McStreamError::Io(ref io_error) = e {
                        if io_error.kind() == std::io::ErrorKind::PermissionDenied {
                            eprintln!("权限不足，请尝试以下解决方案：");
                            eprintln!("1. 以管理员身份运行程序");
                            eprintln!("2. 选择其他输出目录（如桌面或文档文件夹）");
                            eprintln!("3. 修改目标目录的权限");
                        }
                    }
                    Err(e)
                }
            }
        },
        
        Commands::Unpack { input, output } => {
            // 检查输入文件是否存在
            if !input.exists() {
                return Err(McStreamError::Io(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    format!("输入文件不存在: {}", input.display())
                )));
            }
            
            // 确保输出目录存在
            if let Some(parent) = output.parent() {
                std::fs::create_dir_all(parent)?;
            }
            
            println!("解包中...");
            match unpack_mcs_to_json(input, output) {
                Ok(_) => {
                    println!("解包完成: {}", output.display());
                    Ok(())
                },
                Err(e) => {
                    eprintln!("解包失败: {}", e);
                    Err(e)
                }
            }
        },
        
        Commands::Info { file, verbose } => {
            // 检查文件是否存在
            if !file.exists() {
                return Err(McStreamError::Io(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    format!("文件不存在: {}", file.display())
                )));
            }
            
            match print_mcs_info(file, *verbose) {
                Ok(_) => Ok(()),
                Err(e) => {
                    eprintln!("获取文件信息失败: {}", e);
                    Err(e)
                }
            }
        },
    }
}

/// 打包JSON建筑数据为MCS格式
fn pack_json_to_mcs(
    input: &PathBuf, 
    output: &PathBuf, 
    compression: CompressionType
) -> Result<(), McStreamError> {
    // 读取JSON文件
    let file = File::open(input)?;
    let reader = BufReader::new(file);
    
    // 解析JSON
    let data: serde_json::Value = serde_json::from_reader(reader)
        .map_err(|e| McStreamError::ValidationError(format!("JSON解析错误: {}", e)))?;
    
    // 创建MCS编码器
    let mut encoder = McsEncoder::new(compression);
    
    // 处理方块数据
    if let Some(blocks) = data.get("blocks").and_then(|b| b.as_array()) {
        for block in blocks {
            let block_id = block.get("id")
                .and_then(|id| id.as_str())
                .ok_or_else(|| McStreamError::ValidationError("方块缺少id字段".to_string()))?
                .to_string();
            
            let pos = block.get("pos")
                .and_then(|p| p.as_array())
                .ok_or_else(|| McStreamError::ValidationError("方块缺少pos字段".to_string()))?;
            
            if pos.len() != 3 {
                return Err(McStreamError::ValidationError("方块坐标格式错误".to_string()));
            }
            
            let x = pos[0].as_i64().unwrap_or(0) as i32;
            let y = pos[1].as_i64().unwrap_or(0) as i32;
            let z = pos[2].as_i64().unwrap_or(0) as i32;
            
            // 处理NBT数据
            let nbt = block.get("nbt").map(|n| {
                serde_json::to_vec(n)
                    .map_err(|e| McStreamError::ValidationError(format!("无法序列化NBT: {}", e)))
            }).transpose()?;
            
            encoder.add_block(block_id, x, y, z, nbt)?;
        }
    }
    
    // 写入文件
    encoder.write_to_file(output)?;
    
    Ok(())
}

/// 解包MCS文件为JSON格式
fn unpack_mcs_to_json(
    input: &PathBuf, 
    output: &PathBuf
) -> Result<(), McStreamError> {
    // 读取MCS文件
    let decoder = McsDecoder::from_file(input)?;
    
    // 解析并获取数据
    let chunks = decoder.get_chunks();
    
    // 构建JSON对象
    let mut blocks = Vec::new();
    
    for chunk in chunks.values() {
        for block in &chunk.blocks {
            // 获取方块ID
            let block_id = chunk.palette.get(block.palette_index as usize)
                .ok_or_else(|| McStreamError::ValidationError("无效的调色板索引".to_string()))?;
            
            // 计算全局坐标
            let x = (chunk.pos.x * 16) + block.pos.x as i32;
            let y = block.pos.actual_y();
            let z = (chunk.pos.z * 16) + block.pos.z as i32;
            
            // 转换NBT数据
            let nbt = if let Some(nbt_data) = &block.nbt {
                serde_json::from_slice::<serde_json::Value>(nbt_data)
                    .map_err(|e| McStreamError::ValidationError(format!("NBT解析错误: {}", e)))?
            } else {
                serde_json::Value::Null
            };
            
            // 创建方块对象
            let block_obj = serde_json::json!({
                "id": block_id,
                "pos": [x, y, z],
                "nbt": nbt
            });
            
            blocks.push(block_obj);
        }
    }
    
    // 创建最终的JSON对象
    let json = serde_json::json!({
        "format": "mcs",
        "version": "1.0",
        "blocks": blocks
    });
    
    // 写入文件
    let file = File::create(output)?;
    let writer = BufWriter::new(file);
    serde_json::to_writer_pretty(writer, &json)
        .map_err(|e| McStreamError::ValidationError(format!("JSON写入错误: {}", e)))?;
    
    Ok(())
}

/// 打印MCS文件信息
fn print_mcs_info(file: &PathBuf, verbose: bool) -> Result<(), McStreamError> {
    let decoder = McsDecoder::from_file(file)?;
    let header = decoder.header();
    let chunks = decoder.get_chunks();
    
    println!("=== MCS文件信息 ===");
    println!("文件: {}", file.display());
    println!("版本: {}.{}", header.version >> 8, header.version & 0xFF);
    
    let compression = match header.compression {
        0 => "无压缩",
        1 => "Zstandard",
        2 => "LZ4",
        3 => "Brotli",
        _ => "未知"
    };
    println!("压缩算法: {} ({})", compression, header.compression);
    
    let has_signature = (header.flags & 0x01) != 0;
    println!("是否有签名: {}", if has_signature { "是" } else { "否" });
    
    println!("区块数量: {}", chunks.len());
    
    let mut total_blocks = 0;
    for chunk in chunks.values() {
        total_blocks += chunk.blocks.len();
    }
    println!("方块总数: {}", total_blocks);
    
    if verbose {
        println!("\n=== 详细信息 ===");
        
        for (i, (pos, chunk)) in chunks.iter().enumerate() {
            println!("区块 #{} ({}, {})", i + 1, pos.x, pos.z);
            println!("  方块数量: {}", chunk.blocks.len());
            println!("  调色板大小: {}", chunk.palette.len());
            
            if chunk.blocks.len() > 0 && i < 5 {
                println!("  方块示例:");
                for (j, block) in chunk.blocks.iter().take(3).enumerate() {
                    let block_id_str = "unknown".to_string();
                    let block_id = chunk.palette.get(block.palette_index as usize)
                        .unwrap_or(&block_id_str);
                    println!("    #{}: {} @ ({}, {}, {})", 
                        j + 1, 
                        block_id, 
                        (pos.x * 16) + block.pos.x as i32,
                        block.pos.actual_y(),
                        (pos.z * 16) + block.pos.z as i32
                    );
                }
                
                if chunk.blocks.len() > 3 {
                    println!("    ... 还有 {} 个方块", chunk.blocks.len() - 3);
                }
            }
        }
    }
    
    Ok(())
} 