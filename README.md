# MCStream (MCS) 格式实现

MCStream是一种为Minecraft建筑设计的高效二进制流式存储格式，支持按需加载、低内存占用和完整的NBT兼容性。

## 核心特性

- **流式分块**：按区块独立压缩，支持随机读取
- **紧凑编码**：调色板复用、局部坐标压缩、NBT存储
- **多种压缩算法**：支持Zstandard、LZ4、Brotli和无压缩模式
- **数据校验**：内置SHA-256哈希校验和可选数字签名
- **高性能**：并行压缩和解压，低内存开销

## 编译与安装

确保已安装Rust工具链（1.56.0或更高版本），然后执行：

```bash
# 克隆仓库
git clone https://github.com/nethard/mcstream.git
cd mcstream

# 编译
cargo build --release

# 安装
cargo install --path .
```

## 命令行使用

### 打包建筑数据（JSON格式）为MCS文件

```bash
mcs pack -i building.json -o building.mcs -c zstd
```

压缩算法选项：
- `none`：无压缩
- `zstd`：Zstandard压缩（默认，兼顾速度与压缩率）
- `lz4`：LZ4压缩（高速但压缩率较低）
- `brotli`：Brotli压缩（高压缩率但较慢）

### 解包MCS文件为JSON格式

```bash
mcs unpack -i building.mcs -o building.json
```

### 查看MCS文件信息

```bash
# 基本信息
mcs info -f building.mcs

# 详细信息
mcs info -f building.mcs -v
```

## 程序API使用

### 打包示例

```rust
use mcstream::{McsEncoder, CompressionType, McStreamError};

fn main() -> Result<(), McStreamError> {
    // 创建编码器
    let mut encoder = McsEncoder::new(CompressionType::Zstandard);
    
    // 添加方块
    encoder.add_block("minecraft:stone".to_string(), 0, 0, 0, None)?;
    
    // 添加带NBT数据的方块
    let nbt_data = serde_json::to_vec(&serde_json::json!({
        "Items": [{"id": "minecraft:diamond", "Count": 1}]
    }))?;
    encoder.add_block("minecraft:chest".to_string(), 1, 0, 0, Some(nbt_data))?;
    
    // 批量添加相同类型的方块
    let positions = vec![(2, 0, 0), (2, 0, 1), (2, 0, 2)];
    encoder.add_blocks("minecraft:oak_planks".to_string(), &positions, None)?;
    
    // 写入文件
    encoder.write_to_file("output.mcs")?;
    
    Ok(())
}
```

### 解包示例

```rust
use mcstream::{McsDecoder, McStreamError};

fn main() -> Result<(), McStreamError> {
    // 从文件加载
    let decoder = McsDecoder::from_file("input.mcs")?;
    
    // 获取文件头信息
    let header = decoder.header();
    println!("版本: {}.{}", header.version >> 8, header.version & 0xFF);
    
    // 获取区块数据
    for (pos, chunk) in decoder.get_chunks() {
        println!("区块 [{}, {}] 包含 {} 个方块", pos.x, pos.z, chunk.blocks.len());
        
        // 处理方块
        for block in &chunk.blocks {
            let block_id = &chunk.palette[block.palette_index as usize];
            let global_x = (pos.x * 16) + block.pos.x as i32;
            let global_y = block.pos.actual_y(); // 转换为实际Y坐标
            let global_z = (pos.z * 16) + block.pos.z as i32;
            
            println!("方块 {} 位于 [{}, {}, {}]", block_id, global_x, global_y, global_z);
        }
    }
    
    Ok(())
}
```

## JSON格式规范

输入和输出的JSON格式遵循以下结构：

```json
{
  "format": "mcs",
  "version": "1.0",
  "blocks": [
    {
      "id": "minecraft:stone",
      "pos": [0, 0, 0]
    },
    {
      "id": "minecraft:chest",
      "pos": [1, 0, 0],
      "nbt": { /* 可选的NBT数据 */ }
    }
  ]
}
```

注意：
- 方块坐标使用 `pos` 字段作为数组，按顺序表示 [x, y, z]
- 空气方块 (minecraft:air) 会自动被忽略
- NBT数据为可选字段，格式为标准JSON对象

## 格式说明

MCStream格式基于二进制结构，由以下组件组成：

- **头部**：8字节魔数 + 2字节版本号 + 压缩和标志字段
- **区块索引表**：每个区块的位置、大小和坐标信息
- **区块数据**：调色板编码的方块数据，按区块独立压缩
- **SHA-256哈希**：文件数据的完整性校验
- **签名数据**：可选的数字签名（如果有）

更详细的格式规范请参见[格式规范文档](FORMAT.md)。

## 性能优化

MCStream格式针对Minecraft建筑数据进行了多项性能优化：

- **区块级并行处理**：使用Rayon实现压缩和解压的并行计算
- **内存优化**：使用流式读写减少内存占用
- **调色板复用**：相同方块ID在一个区块内只存储一次
- **坐标压缩**：使用区块局部坐标减少每个方块的存储开销
- **选择性NBT存储**：只有需要NBT数据的方块才会包含额外数据

## 许可证

此项目采用GNU Affero General Public License v3.0许可证。详情见[LICENSE](LICENSE)文件。

## 鸣谢

- [Nethard Studio](https://github.com/nethard-project) - 项目维护者
- 所有贡献者和测试者 
