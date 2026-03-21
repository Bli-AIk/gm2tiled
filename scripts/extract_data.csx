// extract_data.csx — utmt (UndertaleModTool) CLI script
// Extracts textures, backgrounds, and rooms from a GameMaker data.win file.
// Usage: utmt load <data.win> --scripts extract_data.csx
//
// The output directory is read from /tmp/gm2tiled_outdir.

using System;
using System.IO;
using System.Text.Json;
using System.Collections.Generic;

// Read output directory
string outDir = File.ReadAllText("/tmp/gm2tiled_outdir").Trim();
string texturesDir = Path.Combine(outDir, "textures");
string roomsDir = Path.Combine(outDir, "rooms");

Directory.CreateDirectory(texturesDir);
Directory.CreateDirectory(roomsDir);

var jsonOptions = new JsonSerializerOptions
{
    PropertyNamingPolicy = JsonNamingPolicy.CamelCase,
    WriteIndented = false
};

// ─── Export texture pages ────────────────────────────────────────────────────

ScriptMessage($"Exporting {Data.EmbeddedTextures.Count} texture pages...");
for (int i = 0; i < Data.EmbeddedTextures.Count; i++)
{
    var tex = Data.EmbeddedTextures[i];
    string pngPath = Path.Combine(texturesDir, $"{i}.png");
    using var fs = File.OpenWrite(pngPath);
    tex.TextureData.Image.SavePng(fs);
}

// ─── Export backgrounds ───────────────────────────────────────────────────────

ScriptMessage($"Exporting {Data.Backgrounds.Count} backgrounds...");
var bgList = new List<object>();
for (int i = 0; i < Data.Backgrounds.Count; i++)
{
    var bg = Data.Backgrounds[i];
    if (bg.Texture == null) continue;
    int pageIndex = Data.EmbeddedTextures.IndexOf(bg.Texture.TexturePage);
    bgList.Add(new
    {
        name = bg.Name.Content,
        texturePageIndex = pageIndex,
        sourceX = (int)bg.Texture.SourceX,
        sourceY = (int)bg.Texture.SourceY,
        sourceWidth = (int)bg.Texture.SourceWidth,
        sourceHeight = (int)bg.Texture.SourceHeight,
        gms2TileWidth = (int)bg.GMS2TileWidth,
        gms2TileHeight = (int)bg.GMS2TileHeight
    });
}

// GMS2 compat: tile tilesets are stored as sprites named "bg_*" (not in Data.Backgrounds)
var existingBgNames = new HashSet<string>();
foreach (var bg in Data.Backgrounds)
{
    if (bg.Texture != null)
    {
        existingBgNames.Add(bg.Name.Content);
    }
}
foreach (var spr in Data.Sprites)
{
    string sprName = spr.Name?.Content ?? "";
    if (!sprName.StartsWith("bg_")) continue;
    if (existingBgNames.Contains(sprName)) continue;
    if (spr.Textures == null || spr.Textures.Count == 0) continue;
    var tex = spr.Textures[0]?.Texture;
    if (tex == null) continue;
    int pageIndex = Data.EmbeddedTextures.IndexOf(tex.TexturePage);
    if (pageIndex < 0) continue;
    bgList.Add(new
    {
        name = sprName,
        texturePageIndex = pageIndex,
        sourceX = (int)tex.SourceX,
        sourceY = (int)tex.SourceY,
        sourceWidth = (int)tex.SourceWidth,
        sourceHeight = (int)tex.SourceHeight
    });
}

File.WriteAllText(
    Path.Combine(outDir, "backgrounds.json"),
    JsonSerializer.Serialize(bgList, jsonOptions)
);

// ─── Export sprites ───────────────────────────────────────────────────────────

ScriptMessage($"Exporting {Data.Sprites.Count} sprites...");
var spriteList = new List<object>();
foreach (var spr in Data.Sprites)
{
    string sprName = spr.Name?.Content ?? "";
    if (string.IsNullOrWhiteSpace(sprName)) continue;
    if (spr.Textures == null || spr.Textures.Count == 0) continue;

    var frames = new List<object>();
    foreach (var sprFrame in spr.Textures)
    {
        var tex = sprFrame?.Texture;
        if (tex == null) continue;

        int pageIndex = Data.EmbeddedTextures.IndexOf(tex.TexturePage);
        if (pageIndex < 0) continue;

        frames.Add(new
        {
            texturePageIndex = pageIndex,
            sourceX = (int)tex.SourceX,
            sourceY = (int)tex.SourceY,
            sourceWidth = (int)tex.SourceWidth,
            sourceHeight = (int)tex.SourceHeight
        });
    }

    if (frames.Count == 0) continue;

    spriteList.Add(new
    {
        name = sprName,
        originX = (int)spr.OriginX,
        originY = (int)spr.OriginY,
        frames = frames
    });
}

File.WriteAllText(
    Path.Combine(outDir, "sprites.json"),
    JsonSerializer.Serialize(spriteList, jsonOptions)
);

// ─── Export rooms ─────────────────────────────────────────────────────────────

ScriptMessage($"Exporting {Data.Rooms.Count} rooms...");
foreach (var room in Data.Rooms)
{
    string roomName = room.Name.Content;
    ScriptMessage($"  Room: {roomName}");

    var tiles = new List<object>();
    var gameObjects = new List<object>();
    var views = new List<object>();
    var gms2TileLayers = new List<object>();

    bool isGms1 = room.Layers == null || room.Layers.Count == 0;

    if (isGms1)
    {
        // GMS1: tiles from room.Tiles, objects from room.GameObjects
        foreach (var tile in room.Tiles)
        {
            tiles.Add(new
            {
                x = tile.X,
                y = tile.Y,
                sourceX = tile.SourceX,
                sourceY = tile.SourceY,
                width = tile.Width,
                height = tile.Height,
                depth = tile.TileDepth,
                background = tile.BackgroundDefinition?.Name?.Content ?? "",
                scaleX = tile.ScaleX,
                scaleY = tile.ScaleY,
                color = tile.Color,
                instanceId = tile.InstanceID
            });
        }
        foreach (var obj in room.GameObjects)
        {
            var spr = obj.ObjectDefinition?.Sprite;
            int spritePage = -1, sprSrcX = 0, sprSrcY = 0, sprSrcW = 0, sprSrcH = 0, sprOriX = 0, sprOriY = 0;
            if (spr?.Textures != null && spr.Textures.Count > 0)
            {
                var sprTex = spr.Textures[0].Texture;
                if (sprTex != null)
                {
                    spritePage = Data.EmbeddedTextures.IndexOf(sprTex.TexturePage);
                    sprSrcX = (int)sprTex.SourceX;
                    sprSrcY = (int)sprTex.SourceY;
                    sprSrcW = (int)sprTex.SourceWidth;
                    sprSrcH = (int)sprTex.SourceHeight;
                    sprOriX = (int)spr.OriginX;
                    sprOriY = (int)spr.OriginY;
                }
            }
            gameObjects.Add(new
            {
                x = obj.X,
                y = obj.Y,
                objectName = obj.ObjectDefinition?.Name?.Content ?? "",
                scaleX = obj.ScaleX,
                scaleY = obj.ScaleY,
                rotation = obj.Rotation,
                color = obj.Color,
                instanceId = obj.InstanceID,
                spritePage = spritePage,
                spriteSourceX = sprSrcX,
                spriteSourceY = sprSrcY,
                spriteSourceWidth = sprSrcW,
                spriteSourceHeight = sprSrcH,
                spriteOriginX = sprOriX,
                spriteOriginY = sprOriY
            });
        }
    }
    else
    {
        // GMS2: layers
        foreach (var layer in room.Layers)
        {
            string layerType = layer.LayerType.ToString();

            if (layerType == "Assets" && layer.AssetsData?.LegacyTiles != null)
            {
                foreach (var tile in layer.AssetsData.LegacyTiles)
                {
                    tiles.Add(new
                    {
                        x = tile.X,
                        y = tile.Y,
                        sourceX = tile.SourceX,
                        sourceY = tile.SourceY,
                        width = tile.Width,
                        height = tile.Height,
                        depth = tile.TileDepth,
                        background = tile.BackgroundDefinition?.Name?.Content
                                     ?? tile.SpriteDefinition?.Name?.Content
                                     ?? "",
                        scaleX = tile.ScaleX,
                        scaleY = tile.ScaleY,
                        color = tile.Color,
                        instanceId = tile.InstanceID
                    });
                }
            }
            else if (layerType == "Instances" && layer.InstancesData?.Instances != null)
            {
                foreach (var inst in layer.InstancesData.Instances)
                {
                    var spr = inst.ObjectDefinition?.Sprite;
                    int spritePage = -1, sprSrcX = 0, sprSrcY = 0, sprSrcW = 0, sprSrcH = 0, sprOriX = 0, sprOriY = 0;
                    if (spr?.Textures != null && spr.Textures.Count > 0)
                    {
                        var sprTex = spr.Textures[0].Texture;
                        if (sprTex != null)
                        {
                            spritePage = Data.EmbeddedTextures.IndexOf(sprTex.TexturePage);
                            sprSrcX = (int)sprTex.SourceX;
                            sprSrcY = (int)sprTex.SourceY;
                            sprSrcW = (int)sprTex.SourceWidth;
                            sprSrcH = (int)sprTex.SourceHeight;
                            sprOriX = (int)spr.OriginX;
                            sprOriY = (int)spr.OriginY;
                        }
                    }
                    gameObjects.Add(new
                    {
                        x = inst.X,
                        y = inst.Y,
                        objectName = inst.ObjectDefinition?.Name?.Content ?? "",
                        scaleX = inst.ScaleX,
                        scaleY = inst.ScaleY,
                        rotation = inst.Rotation,
                        color = (uint)inst.Color,
                        instanceId = inst.InstanceID,
                        spritePage = spritePage,
                        spriteSourceX = sprSrcX,
                        spriteSourceY = sprSrcY,
                        spriteSourceWidth = sprSrcW,
                        spriteSourceHeight = sprSrcH,
                        spriteOriginX = sprOriX,
                        spriteOriginY = sprOriY
                    });
                }
            }
            else if (layerType == "Tiles" && layer.TilesData != null)
            {
                var rawData = layer.TilesData.TileData;
                var tileData = new List<List<uint>>();
                foreach (var row in rawData)
                {
                    var rowList = new List<uint>(row);
                    tileData.Add(rowList);
                }
                gms2TileLayers.Add(new
                {
                    name = layer.LayerName?.Content ?? "",
                    depth = layer.LayerDepth,
                    background = layer.TilesData.Background?.Name?.Content ?? "",
                    tilesX = layer.TilesData.TilesX,
                    tilesY = layer.TilesData.TilesY,
                    tileData = tileData
                });
            }
        }
    }

    // Views (common to both GMS1 and GMS2)
    if (room.Views != null)
    {
        foreach (var view in room.Views)
        {
            views.Add(new
            {
                enabled = view.Enabled,
                viewX = view.ViewX,
                viewY = view.ViewY,
                viewWidth = view.ViewWidth,
                viewHeight = view.ViewHeight,
                portX = view.PortX,
                portY = view.PortY,
                portWidth = view.PortWidth,
                portHeight = view.PortHeight,
                borderX = view.BorderX,
                borderY = view.BorderY,
                speedX = view.SpeedX,
                speedY = view.SpeedY
            });
        }
    }

    var roomData = new
    {
        name = roomName,
        width = room.Width,
        height = room.Height,
        speed = room.Speed,
        persistent = room.Persistent,
        gridWidth = room.GridWidth,
        gridHeight = room.GridHeight,
        backgroundColor = room.BackgroundColor,
        drawBackgroundColor = room.DrawBackgroundColor,
        tiles = tiles,
        gameObjects = gameObjects,
        views = views,
        gms2TileLayers = gms2TileLayers
    };

    File.WriteAllText(
        Path.Combine(roomsDir, $"{roomName}.json"),
        JsonSerializer.Serialize(roomData, jsonOptions)
    );
}

ScriptMessage("gm2tiled extraction complete.");
