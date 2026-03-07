# gm2tiled

[![license](https://img.shields.io/badge/license-GPLv3-blue)](LICENSE) <img src="https://img.shields.io/github/repo-size/Bli_AIk/gm2tiled.svg"/> <img src="https://img.shields.io/github/last-commit/Bli_AIk/gm2tiled.svg"/> <br>
<img src="https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white" />

> Current Status: 🚧 Early Development (Initial version in progress)

**gm2tiled** — Convert GameMaker 1.x/2.x `data.win` tilemaps to [Tiled](https://www.mapeditor.org/) `.tmx` projects.

| English | Simplified Chinese |
|---------|-------------------|
| English | [简体中文](./readme_zh-hans.md) |

## Introduction

`gm2tiled` is a command-line tool that extracts room/tilemap data from GameMaker `data.win` archives
and converts it to [Tiled](https://www.mapeditor.org/) `.tmx` project files.

It solves the problem of GameMaker map data being locked in an opaque binary format, allowing users
to view, edit, and remix game levels using the open-source Tiled map editor.

This project was started with the goal of **creating a Tiled asset library for
[Undertale](https://undertale.com/) and [Deltarune](https://www.deltarune.com/)** — making it easy
for fan-game developers to work with the original maps in a standard format.

### Acknowledgements

This tool is built on top of [UndertaleModTool (UTMT)](https://github.com/UnderminersTeam/UndertaleModTool),
an open-source GameMaker reverse-engineering toolkit. Huge thanks to all UTMT contributors for their
incredible work on documenting and implementing the GameMaker data formats that make this project
possible.

## Features

* Supports **GMS1** (`data.win` v14, e.g. Undertale) and **GMS2** (`data.win` v17, e.g. Deltarune)
* Exports tileset images (cropped from GameMaker texture atlas pages) as `.png`
* Generates Tiled `.tsx` tileset files for each unique background/tileset
* Generates Tiled `.tmx` map files with layers grouped by render depth
* Grid-aligned uniform tiles → efficient `TileLayer` (CSV encoding)
* Non-uniform / free-placed tiles → `ObjectGroup` fallback with tile objects
* GameObjects → `ObjectGroup` with `type` property set to the object name
* `--skip-extract` flag to reuse previously extracted data (fast iteration)
* (Planned) Batch conversion: convert all rooms in a `data.win` at once
* (Planned) GMS2 native `Tiles` layer support (grid-based `uint[][]` tile data)
* (Planned) Deltarune background layer export (runtime-set backgrounds)

## How to Use

### Prerequisites

* **Rust** 1.85 or later
* **[UndertaleModTool CLI](https://github.com/UnderminersTeam/UndertaleModTool)** (`utmt`) installed and on your `PATH`

Install Rust if not already installed:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### Steps

1. **Clone the repository**:

   ```bash
   git clone https://github.com/Bli_AIk/gm2tiled.git
   cd gm2tiled
   ```

2. **Build and install**:

   ```bash
   cargo install --path .
   ```

3. **Convert rooms**:

   ```bash
   # Convert specific rooms from Undertale
   gm2tiled --input /path/to/UNDERTALE/data.win \
             --output ./output \
             --rooms room_ruins1,room_ruins2

   # Convert all rooms
   gm2tiled --input /path/to/data.win --output ./output --rooms all

   # Skip re-extraction if you already ran once (faster iteration)
   gm2tiled --input /path/to/data.win --output ./output --rooms room_ruins1 --skip-extract
   ```

4. **Open in Tiled**:

   Open any `.tmx` file in `./output/rooms/` with [Tiled](https://www.mapeditor.org/).

## How to Build

### Prerequisites

* Rust 1.85 or later
* `utmt` CLI on your `PATH`

### Build Steps

1. **Clone the repository**:

   ```bash
   git clone https://github.com/Bli_AIk/gm2tiled.git
   cd gm2tiled
   ```

2. **Build the project**:

   ```bash
   cargo build --release
   ```

3. **Run tests**:

   ```bash
   cargo test
   ```

4. **Install globally** (optional):

   ```bash
   cargo install --path .
   ```

## Output Structure

```
output/
├── rooms/
│   ├── room_ruins1.tmx
│   └── room_ruins2.tmx
├── tilesets/
│   ├── bg_ruinsplaceholder.tsx
│   └── bg_ruinsplaceholder.png
└── extracted/          # raw JSON + texture pages from utmt
    ├── backgrounds.json
    ├── rooms/
    └── textures/
```

## Dependencies

| Crate | Version | Description |
|-------|---------|-------------|
| [clap](https://crates.io/crates/clap) | 4 | Command-line argument parsing |
| [anyhow](https://crates.io/crates/anyhow) | 1 | Ergonomic error handling |
| [serde](https://crates.io/crates/serde) | 1 | Serialization framework |
| [serde_json](https://crates.io/crates/serde_json) | 1 | JSON deserialization for utmt output |
| [image](https://crates.io/crates/image) | 0.25 | PNG texture atlas cropping |
| [quick-xml](https://crates.io/crates/quick-xml) | 0.36 | TMX/TSX XML generation |

## Contributing

Contributions are welcome!
Whether you want to fix a bug, add a feature, or improve documentation:

* Submit an **Issue** or **Pull Request**.
* Share ideas and discuss design or architecture.

## License

This project is licensed under the [GNU General Public License v3.0](LICENSE).

It uses [UndertaleModTool](https://github.com/UnderminersTeam/UndertaleModTool) as an external tool,
which is also licensed under GPLv3.
