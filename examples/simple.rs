use mcstream::{CompressionType, McStreamError, McsDecoder, McsEncoder};
use std::path::Path;

fn main() -> Result<(), McStreamError> {
    // 生成示例数据
    println!("创建示例建筑...");
    let output_path = Path::new("simple_example.mcs");

    // 创建编码器
    let mut encoder = McsEncoder::new(CompressionType::Zstandard);

    // 添加一些方块：一个简单的3x3x3石头立方体，中心是箱子
    for x in -1..=1 {
        for y in -1..=1 {
            for z in -1..=1 {
                if x == 0 && y == 0 && z == 0 {
                    // 中心放一个箱子，带有NBT数据
                    let nbt_data = simple_chest_nbt();
                    encoder.add_block("minecraft:chest".to_string(), x, y, z, Some(nbt_data))?;
                } else {
                    // 其他位置放石头
                    encoder.add_block("minecraft:stone".to_string(), x, y, z, None)?;
                }
            }
        }
    }

    // 保存MCS文件
    println!("保存到文件: {}", output_path.display());
    encoder.write_to_file(output_path)?;

    // 读取MCS文件
    println!("\n读取文件: {}", output_path.display());
    let decoder = McsDecoder::from_file(output_path)?;

    // 输出文件信息
    let header = decoder.header();
    let compression_name = match header.compression {
        0 => "无压缩",
        1 => "Zstandard",
        2 => "LZ4",
        3 => "Brotli",
        _ => "未知",
    };

    println!("文件信息:");
    println!(
        "  格式版本: {}.{}",
        (header.version >> 8) & 0xFF,
        header.version & 0xFF
    );
    println!("  压缩算法: {}", compression_name);
    println!("  区块数量: {}", decoder.get_chunks().len());

    // 遍历所有方块
    println!("\n方块列表:");
    let mut total_blocks = 0;

    for (pos, chunk) in decoder.get_chunks() {
        println!(
            "区块 [{}, {}] 包含 {} 个方块",
            pos.x,
            pos.z,
            chunk.blocks.len()
        );
        total_blocks += chunk.blocks.len();

        // 输出每个方块的信息
        for block in &chunk.blocks {
            let block_id = &chunk.palette[block.palette_index as usize];
            let global_x = (pos.x * 16) + block.pos.x as i32;
            let global_y = block.pos.actual_y();
            let global_z = (pos.z * 16) + block.pos.z as i32;

            print!(
                "  方块 {} 位于 [{}, {}, {}]",
                block_id, global_x, global_y, global_z
            );
            if block.nbt.is_some() {
                println!(" (带有NBT数据)");
            } else {
                println!();
            }
        }
    }

    println!("\n总计: {} 个方块", total_blocks);

    Ok(())
}

// 创建一个简单的箱子NBT数据
fn simple_chest_nbt() -> Vec<u8> {
    // 这是一个极度简化的NBT数据，实际项目中可能需要一个完整的NBT库
    // 此示例只是为了演示功能，并不是有效的Minecraft箱子NBT
    vec![
        10, // Compound标签类型
        0, 5, // 名称长度（5字节）
        73, 116, 101, 109, 115, // "Items" 的UTF-8编码
        9,   // List标签类型
        0, 0, // 空列表
        0, // Compound结束标记
    ]
}
