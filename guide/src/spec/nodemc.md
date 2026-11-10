# .nodemc File Format Specified
This specification is valid for SN versions <= 423.

::: warning
The .nodemc format is compressed when saved and decompressed when loaded to reduce its file size. It is compressed and decompressed using the GZIP compression and decompression algorithms with a buffer size of 4096. The following information assumes that the .nodemc file is in its uncompressed state.
:::

## Metadata
|Data Type|Versions|Builds|Description|
|---------|--------|------|-----------|
|32-bit signed int big-endian|Any|Any|Stick Nodes version number
|32-bit signed int big-endian|>=402|Any|Build number
|32-bit signed int big-endian|Any|Any|<span style="color:coral">A</span> = Length of movieclip name (prob causes weird behavior or error if length of movieclip name length is <= 0)
|sequence with size of <span style="color:coral">A</span> of text bytes|Any|Any|Movieclip name

## Options
|Data Type|Versions|Builds|Description|
|---------|--------|------|-----------|
|32-bit signed int big-endian|Any|Any|<span style="color:coral">B</span> = Number of frames
|32-bit signed int big-endian|Any|Any|<span style="color:coral">C</span> = Number of stickfigures
|32-bit signed int big-endian|>=400|Any|<span style="color:coral">D</span> = Number of sprites
|32-bit signed int big-endian|Any|Any|<span style="color:coral">E</span> = Number of sounds
|32-bit signed int big-endian|Any|Any|<span style="color:coral">F</span> = Preview frame count
|32-bit signed int big-endian|Any|Any|<span style="color:coral">W</span> = Preview resolution width
|32-bit signed int big-endian|Any|Any|<span style="color:coral">H</span> = Preview resolution height
|sequence with size of (<span style="color:coral">W</span> * <span style="color:coral">H</span> * <span style="color:coral">F</span>) of [RGBA8888](/sticknodes-rs/spec/formats/stknds.md#rgb888)|Any|Any|Preview image data
|32-bit signed int big-endian|Any|Any|FPS
|32-bit signed int big-endian|Any|Any|Number of tweened frames
|8-bit unsigned byte|Any|Any|Tweening enabled boolean
|8-bit unsigned byte|Any|Any|Tween loop protection enabled boolean
|32-bit signed int big-endian|Any|Any|Unique figure ID

## Stickfigure Library
|Data Type|Versions|Builds|Description|
|---------|--------|------|-----------|
|sequence of size <span style="color:coral">C</span> of [Stickfigure Header](/sticknodes-rs/spec/formats/stknds.md#stickfigure-header)|Any|Any|See [Stickfigure Header](/sticknodes-rs/spec/formats/stknds.md#stickfigure-header) for more information.

## Stickfigure Header
|Data Type|Versions|Builds|Description|
|---------|--------|------|-----------|
|32-bit signed int big-endian|Any|Any|<span style="color:coral">G</span> = Length of stickfigure name
|sequence with size of <span style="color:coral">G</span> of text bytes|Any|Any|Stickfigure name
|[Stickfigure Data](/sticknodes-rs/spec/formats/nodes.md)|Any|Any|See [Stickfigure Data](/sticknodes-rs/spec/formats/nodes.md) for more information.

## Sprite Library
|Data Type|Versions|Builds|Description|
|---------|--------|------|-----------|
|sequence with size of <span style="color:coral">D</span> of [Sprite Header](/sticknodes-rs/spec/formats/stknds.md#sprite-header)|>=400|Any|See [Sprite Header](/sticknodes-rs/spec/formats/stknds.md#sprite-header) for more information

## Sprite Header
|Data Type|Versions|Builds|Description|
|---------|--------|------|-----------|
|8-bit unsigned byte|>=400|Any|<span style="color:lightgreen">if <span style="color:indianred">1</span>: </span><br><span style="color:coral">H</span> = <span style="color:indianred">sprite group source</span> <span style="color:lightgreen"><br> else if <span style="color:indianred">0</span>: </span><br> <span style="color:coral">H</span> = <span style="color:indianred">sprite source</span>
|32-bit signed int big-endian|>=400|Any|<span style="color:coral">I</span> = Length of sprite name
|sequence with size of <span style="color:coral">I</span> of text bytes|>=400|Any|Sprite name
|32-bit signed int big-endian|>=400|Any|Sprite ID?
|<span style="color:lightgreen">if <span style="color:coral">H</span></span> == <span style="color:indianred">sprite group source</span><span style="color:lightgreen">:</span><br> [Sprite Group Library Data](/sticknodes-rs/)<br> <span style="color:lightgreen">else if <span style="color:coral">H</span></span> == <span style="color:indianred">sprite source</span><span style="color:lightgreen">:</span><br> [Sprite Library Data](/sticknodes-rs/)|>=400|Any|See [Sprite Group Library Data](/sticknodes-rs/) or [Sprite Library Data](/sticknodes-rs/) for more information.

## Sound Library
|Data Type|Versions|Builds|Description|
|---------|--------|------|-----------|
|sequence with size of <span style="color:coral">E</span> of [Sound Header](/sticknodes-rs/spec/formats/stknds.md#sound-header)|Any|Any|See [Sound Header](/sticknodes-rs/spec/formats/stknds.md#sound-header) for more information.

## Sound Header
|Data Type|Versions|Builds|Description|
|---------|--------|------|-----------|
|32-bit signed int big-endian|Any|Any|<span style="color:coral">J</span> = Length of sound name
|sequence with size of <span style="color:coral">J</span> of text bytes|Any|Any|Sound name

## Frames
|Data Type|Versions|Builds|Description|
|---------|--------|------|-----------|
|sequence of [Movieclip Frame](/sticknodes-rs/spec/formats/stknds.md#frame-data) of length <span style="color:coral">F</span>|Any|Any|See [Movieclip Frame](/sticknodes-rs/spec/formats/stknds.md#frame-data) for more information.