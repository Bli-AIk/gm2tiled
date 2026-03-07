# gm2tiled

[![license](https://img.shields.io/badge/license-GPLv3-blue)](LICENSE) <img src="https://img.shields.io/github/repo-size/Bli-AIk/gm2tiled.svg"/> <img src="https://img.shields.io/github/last-commit/Bli-AIk/gm2tiled.svg"/> <br>
<img src="https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white" />

> 当前状态：🚧 早期开发中（初始版本进行中）

**gm2tiled** — 将 GameMaker 1.x/2.x `data.win` 地图数据转换为 [Tiled](https://www.mapeditor.org/) `.tmx` 工程文件。

| English | 简体中文 |
|---------|---------|
| [English](./readme.md) | 简体中文 |

## 简介

`gm2tiled` 是一个命令行工具，用于从 GameMaker `data.win` 文件中提取房间/瓦片地图数据，
并将其转换为 [Tiled](https://www.mapeditor.org/) `.tmx` 工程文件。

它解决了 GameMaker 地图数据被锁在不透明二进制格式中的问题，让用户能够用开源的 Tiled 地图编辑器
查看、编辑和二次创作游戏关卡。

本项目以**为 [Undertale](https://undertale.com/) 和 [Deltarune](https://www.deltarune.com/)
构建 Tiled 素材库**为出发点——让同人游戏开发者能够以标准格式使用原版地图资源。

### 致谢

本工具构建于 [UndertaleModTool (UTMT)](https://github.com/UnderminersTeam/UndertaleModTool) 之上，
UTMT 是一个开源的 GameMaker 逆向工程工具集。衷心感谢所有 UTMT 贡献者的卓越工作——
正是他们对 GameMaker 数据格式的记录和实现，使本项目得以成为可能。

输出格式面向 [Tiled](https://www.mapeditor.org/)——一款强大的免费开源地图编辑器。
感谢 Tiled 团队和社区构建并维护了如此优秀的工具。

房间和瓦片集的结构也参考了官方的
[GameMaker Tiled 导入/导出插件](https://github.com/YoYoGames/GMTK-2024)
以及社区中关于 GameMaker 房间格式的相关资料。

## 功能

* 支持 **GMS1**（`data.win` v14，如 Undertale）和 **GMS2**（`data.win` v17，如 Deltarune）
* 从 GameMaker 纹理图集页中裁剪并导出瓦片集图片（`.png`）
* 为每个独立的背景/瓦片集生成 Tiled `.tsx` 文件
* 生成按渲染深度分组的 Tiled `.tmx` 地图文件
* 网格对齐的均匀瓦片 → 高效的 `TileLayer`（CSV 编码）
* 非均匀/自由放置的瓦片 → 使用瓦片对象的 `ObjectGroup` 兜底方案
* 游戏对象 → `ObjectGroup`，`type` 属性设为对象名称
* `--skip-extract` 参数可复用已提取的数据（快速迭代）
* （计划中）批量转换：一次性转换 `data.win` 中的所有房间
* （计划中）GMS2 原生 `Tiles` 层支持（基于网格的 `uint[][]` 瓦片数据）
* （计划中）Deltarune 背景层导出（运行时设置的背景）

## 使用方法

### 前置条件

* **Rust** 1.85 或更高版本
* **[UndertaleModTool CLI](https://github.com/UnderminersTeam/UndertaleModTool)**（`utmt`）已安装并在 `PATH` 中

安装 Rust（如尚未安装）：

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### 步骤

1. **克隆仓库**：

   ```bash
   git clone https://github.com/Bli-AIk/gm2tiled.git
   cd gm2tiled
   ```

2. **构建并安装**：

   ```bash
   cargo install --path .
   ```

3. **转换房间**：

   ```bash
   # 转换 Undertale 中的指定房间
   gm2tiled --input /path/to/UNDERTALE/data.win \
             --output ./output \
             --rooms room_ruins1,room_ruins2

   # 转换所有房间
   gm2tiled --input /path/to/data.win --output ./output --rooms all

   # 跳过重新提取（已运行过一次时，速度更快）
   gm2tiled --input /path/to/data.win --output ./output --rooms room_ruins1 --skip-extract
   ```

4. **在 Tiled 中打开**：

   用 [Tiled](https://www.mapeditor.org/) 打开 `./output/rooms/` 中的任意 `.tmx` 文件。

## 构建方法

### 前置条件

* Rust 1.85 或更高版本
* `utmt` CLI 在 `PATH` 中

### 构建步骤

1. **克隆仓库**：

   ```bash
   git clone https://github.com/Bli-AIk/gm2tiled.git
   cd gm2tiled
   ```

2. **构建项目**：

   ```bash
   cargo build --release
   ```

3. **运行测试**：

   ```bash
   cargo test
   ```

4. **全局安装**（可选）：

   ```bash
   cargo install --path .
   ```

## 输出结构

```
output/
├── rooms/
│   ├── room_ruins1.tmx
│   └── room_ruins2.tmx
├── tilesets/
│   ├── bg_ruinsplaceholder.tsx
│   └── bg_ruinsplaceholder.png
└── extracted/          # utmt 导出的原始 JSON 和纹理页
    ├── backgrounds.json
    ├── rooms/
    └── textures/
```

## 依赖

| Crate | 版本 | 说明 |
|-------|------|------|
| [clap](https://crates.io/crates/clap) | 4 | 命令行参数解析 |
| [anyhow](https://crates.io/crates/anyhow) | 1 | 人体工程学错误处理 |
| [serde](https://crates.io/crates/serde) | 1 | 序列化框架 |
| [serde_json](https://crates.io/crates/serde_json) | 1 | 反序列化 utmt 输出的 JSON |
| [image](https://crates.io/crates/image) | 0.25 | PNG 纹理图集裁剪 |
| [quick-xml](https://crates.io/crates/quick-xml) | 0.36 | 生成 TMX/TSX XML |

## 贡献

欢迎贡献！无论是修复 bug、添加功能还是改进文档：

* 提交 **Issue** 或 **Pull Request**。
* 分享想法，讨论设计或架构。

## 许可证

本项目以 [GNU 通用公共许可证 v3.0](LICENSE) 授权。

本工具以外部工具的形式使用了 [UndertaleModTool](https://github.com/UnderminersTeam/UndertaleModTool)，
后者同样以 GPLv3 授权。
