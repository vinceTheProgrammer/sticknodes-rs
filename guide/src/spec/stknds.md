# .stknds File Format Specified
This specification is valid for SN versions <= 423.

::: warning
The .stknds format is compressed when saved and decompressed when loaded to reduce its file size. It is compressed and decompressed using the GZIP compression and decompression algorithms with a buffer size of 4096. The following information assumes that the .stknds file is in its uncompressed state.
:::

## Metadata
|Data Type|Versions|Builds|Description|
|---------|--------|------|-----------|
|32-bit signed int big-endian|Any|Any|Stick Nodes version number
|32-bit signed int big-endian|>=401|Any|Build number
|32-bit signed int big-endian|Any|Any|<span style="color:coral">A</span> = Length of project name (prob causes weird behavior or error if length of project name length is <= 0)
|sequence with size of <span style="color:coral">A</span> of text bytes|Any|Any|Project name

## Options
|Data Type|Versions|Builds|Description|
|---------|--------|------|-----------|
|8-bit unsigned byte|>=401|>=16|Is YouTube short boolean
|8-bit unsigned byte|>=244|Any|Watermark enabled boolean
|32-bit signed int big-endian|>=220|Any|<span style="color:coral">B</span> = Length of watermark text (watermark text bytes are not read if this is <= 0)
|sequence with size of <span style="color:coral">B</span> of text bytes|>=220|Any|Watermark text
|32-bit signed int big-endian|>=220|Any|Watermark color. See [Color](/sticknodes-rs/spec/structs/color.md) for more information.
|32-bit signed int big-endian|Any|Any|FPS
|32-bit signed int big-endian|>=176|Any|Number of tweened frames
|8-bit unsigned byte|>=127|Any|Has shown loop message boolean
|8-bit unsigned byte|>=113|Any|Has shown panning mode message boolean
|8-bit unsigned byte|Any|Any|Has shown tweening mode message boolean
|8-bit unsigned byte|>=220|Any|Has shown magnifier message boolean
|8-bit unsigned byte|Any|Any|Tweening enabled boolean
|8-bit unsigned byte|>=127|Any|Is loop boolean
|8-bit unsigned byte|>=140 && < 200|Any|[empty byte]

## Unique IDs
Probably used to keep track of the last assigned unique ID(?)
|Data Type|Versions|Builds|Description|
|---------|--------|------|-----------|
|32-bit signed int big-endian|Any|Any|Unique figure ID
|32-bit signed int big-endian|>=176|Any|Unique textfield box ID
|32-bit signed int big-endian|>=300|Any|Unique movieclip ID
|32-bit signed int big-endian|>=300|Any|Frames container UID

## Fonts
|Data Type|Versions|Builds|Description|
|---------|--------|------|-----------|
|32-bit signed int big-endian|>=400|Any|<span style="color:coral">C</span> = Number of fonts to check if loaded? Does NOT include the default font. Should be equal to whatever the greatest index of all loaded fonts is.
|sequence with size of <span style="color:coral">C</span> of 8-bit unsigned bytes (booleans)|>=400|Any|Loaded fonts booleans. See [font booleans reference](/sticknodes-rs/spec/misc/font_booleans.md) for the order of the booleans. Do NOT write a boolean for font index 0 (the default font)

## Hidden Stickfigure Library IDs
|Data Type|Versions|Builds|Description|
|---------|--------|------|-----------|
|32-bit signed int big-endian|>=300|Any|<span style="color:coral">D</span> = Number of hidden stickfigure library IDs
|sequence with size of <span style="color:coral">D</span> of 32-bit signed int big-endian|>=300|Any|Hidden stickfigure library IDs

## Hidden Movieclip Library IDs
|Data Type|Versions|Builds|Description|
|---------|--------|------|-----------|
|32-bit signed int big-endian|>=300|Any|<span style="color:coral">E</span> = Number of hidden movieclip library IDs
|sequence with size of <span style="color:coral">E</span> of 32-bit signed int big-endian|>=300|Any|Hidden movieclip library IDs

## Hidden Sprite Library IDs
|Data Type|Versions|Builds|Description|
|---------|--------|------|-----------|
|32-bit signed int big-endian|>=400|Any|<span style="color:coral">F</span> = Number of hidden sprite library IDs
|sequence with size of <span style="color:coral">F</span> of 32-bit signed int big-endian|>=400|Any|Hidden sprite library IDs

## ID Conversion Maps
|Data Type|Versions|Builds|Description|
|---------|--------|------|-----------|
|[ID Conversion Maps](/sticknodes-rs/spec/structs/id_conversion_maps.md)|>=300|Any|See [ID Conversion Maps](/sticknodes-rs/spec/structs/id_conversion_maps.md) for more information.

## Background Image
|Data Type|Versions|Builds|Description|
|---------|--------|------|-----------|
|8-bit unsigned byte|>=140 && <400|Any|Background image enabled boolean
|8-bit unsigned byte|>=140 && <170|Any|Some boolean related to bg image scaling? Something vaguely related to being strict 1920x1080 when false and scaling to canvas width and height when true?
|32-bit float big-endian|>=170 && <400|Any|Scale background image x
|32-bit float big-endian|>=170 && <400|Any|Scale background image y
|32-bit signed int big-endian|>=140 && <400|Any|<span style="color:coral">W</span> = Background image width
|32-bit signed int big-endian|>=140 && <400|Any|<span style="color:coral">H</span> = Background image height
|sequence with size of (<span style="color:coral">W</span> * <span style="color:coral">H</span>) of [RGB888](/sticknodes-rs/spec/structs/rgb888.md)|>=140 && <400|Any|Image data

## Stickfigure Library
|Data Type|Versions|Builds|Description|
|---------|--------|------|-----------|
|32-bit signed int big-endian|Any|Any|<span style="color:coral">G</span> = Number of stickfigures in library
|sequence of size <span style="color:coral">G</span> of [Stickfigure Header](/sticknodes-rs/spec/formats/stknds.md#stickfigure-header)|Any|Any|See [Stickfigure Header](/sticknodes-rs/spec/formats/stknds.md#stickfigure-header) for more information.

## Stickfigure Header
|Data Type|Versions|Builds|Description|
|---------|--------|------|-----------|
|32-bit signed int big-endian|Any|Any|<span style="color:coral">H</span> = Length of stickfigure name
|sequence with size of <span style="color:coral">H</span> of text bytes|Any|Any|Stickfigure name
|[Stickfigure Data](/sticknodes-rs/spec/formats/nodes.md)|Any|Any|See [Stickfigure Data](/sticknodes-rs/spec/formats/nodes.md) for more information.

## Sound Library
|Data Type|Versions|Builds|Description|
|---------|--------|------|-----------|
|32-bit signed int big-endian|>=160|Any|<span style="color:coral">I</span> = Number of sounds
|sequence with size of <span style="color:coral">I</span> of [Sound Header](/sticknodes-rs/spec/formats/stknds.md#sound-header)|>=160|Any|See [Sound Header](/sticknodes-rs/spec/formats/stknds.md#sound-header) for more information.

## Sound Header
|Data Type|Versions|Builds|Description|
|---------|--------|------|-----------|
|32-bit signed int big-endian|>=160|Any|<span style="color:coral">J</span> = Length of sound name
|sequence with size of <span style="color:coral">J</span> of text bytes|>=160|Any|Sound name

## Sprite Library
|Data Type|Versions|Builds|Description|
|---------|--------|------|-----------|
|32-bit signed int big-endian|>=400|Any|<span style="color:coral">K</span> = Number of sprites
|sequence with size of <span style="color:coral">K</span> of [Sprite Header](/sticknodes-rs/spec/formats/stknds.md#sprite-header)|>=400|Any|See [Sprite Header](/sticknodes-rs/spec/formats/stknds.md#sprite-header) for more information

## Sprite Header
|Data Type|Versions|Builds|Description|
|---------|--------|------|-----------|
|8-bit unsigned byte|>=400|Any|<span style="color:lightgreen">if <span style="color:indianred">1</span>: </span><br><span style="color:coral">L</span> = <span style="color:indianred">sprite group source</span> <span style="color:lightgreen"><br> else if <span style="color:indianred">0</span>: </span><br> <span style="color:coral">L</span> = <span style="color:indianred">sprite source</span>
|32-bit signed int big-endian|>=400|Any|<span style="color:coral">M</span> = Length of sprite name
|sequence with size of <span style="color:coral">M</span> of text bytes|>=400|Any|Sprite name
|<span style="color:lightgreen">if <span style="color:coral">L</span></span> == <span style="color:indianred">sprite group source</span><span style="color:lightgreen">:</span><br> [Sprite Group Library Data](/sticknodes-rs/spec/structs/sprite_group_source.md)<br> <span style="color:lightgreen">else if <span style="color:coral">L</span></span> == <span style="color:indianred">sprite source</span><span style="color:lightgreen">:</span><br> [Sprite Library Data](/sticknodes-rs/spec/structs/sprite_source.md)|>=400|Any|See [Sprite Group Library Data](/sticknodes-rs/spec/structs/sprite_group_source.md) or [Sprite Library Data](/sticknodes-rs/spec/structs/sprite_source.md) for more information.

## Movieclip Library
|Data Type|Versions|Builds|Description|
|---------|--------|------|-----------|
|32-bit signed int big-endian|>=300|Any|<span style="color:coral">N</span> = Number of movieclips
|sequence with size of <span style="color:coral">N</span> of [Movieclip Header](/sticknodes-rs/spec/formats/stknds.md#movieclip-header)|>=300|Any|See [Movieclip Header](/sticknodes-rs/spec/formats/stknds.md#movieclip-header) for more information.

## Movieclip Header
|Data Type|Versions|Builds|Description|
|---------|--------|------|-----------|
|32-bit signed int big-endian|>=300|Any|<span style="color:coral">O</span> = Length of movieclip name
|sequence with size of <span style="color:coral">O</span> of text bytes|>=300|Any|Movieclip name
|[Movieclip Library Data](/sticknodes-rs/spec/structs/movieclip_library_data.md)|>=300|Any|See [Movieclip Library Data](/sticknodes-rs/spec/structs/movieclip_library_data.md) for more information.
|32-bit signed int big-endian|>=300|Any|Movieclip UID

## Frames
|Data Type|Versions|Builds|Description|
|---------|--------|------|-----------|
|32-bit signed int big-endian|Any|Any|<span style="color:coral">P</span> = Number of frames in project
|sequence of [Frame](/sticknodes-rs/spec/structs/frame.md) of length <span style="color:coral">P</span>|Any|Any|See [Frame](/sticknodes-rs/spec/structs/frame.md) for more information.

## Auto Camera
|Data Type|Versions|Builds|Description|
|---------|--------|------|-----------|
|32-bit signed int big-endian|>=237|Any|<span style="color:coral">Q</span> =  Number of auto-cams
|sequence with size of <span style="color:coral">Q</span> of [Auto Camera](/sticknodes-rs/spec/structs/auto_camera.md)|>=237|Any|See [Auto Camera](/sticknodes-rs/spec/structs/auto_camera.md) for more information.

## Session Save Data
|Data Type|Versions|Builds|Description|
|---------|--------|------|-----------|
|[Session Save Data](/sticknodes-rs/spec/structs/session_save_data.md)|>=251|Any|See [Session Save Data](/sticknodes-rs/spec/structs/session_save_data.md) for more information.

## Movieclip Cache
|Data Type|Versions|Builds|Description|
|---------|--------|------|-----------|
|32-bit signed int big-endian|>=319|Any|<span style="color:coral">R</span> =  Number of cached movieclips
|sequence with size of <span style="color:coral">R</span> of [Cached Movieclip](/sticknodes-rs/spec/structs/cached_movieclip.md)|>=319|Any|See [Cached Movieclip](/sticknodes-rs/spec/structs/cached_movieclip.md) for more information.