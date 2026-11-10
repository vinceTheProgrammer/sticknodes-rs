### Sprite Library Data
|Data Type|Builds|Description|
|---------|------|-----------|
|8-bit unsigned byte|Any|<span style="color:coral">A</span> = Sprite is transparent boolean
|8-bit unsigned byte|Any|Sprite is anti-alias boolean
|32-bit float big-endian|Any|Sprite default origin X
|32-bit float big-endian|Any|Sprite default origin Y
|32-bit float big-endian|Any|Sprite internal scale X
|32-bit float big-endian|Any|Sprite internal scale Y
|8-bit unsigned byte|>=9|Sprite has raw rgb data boolean
|32-bit signed int big-endian|Any|Sprite image width
|32-bit signed int big-endian|Any|Sprite image height
|32-bit signed int big-endian|Any|<span style="color:coral">B</span> = Number of sprite pixels
|sequence with size of <span style="color:coral">B</span> of<br> <span style="color:lightgreen">if <span style="color:coral">A</span></span> == <span style="color:indianred">true</span><span style="color:lightgreen">:</span> <br>[RGBA8888 bytes](/)<br> <span style="color:lightgreen">else if <span style="color:coral">A</span></span> == <span style="color:indianred">false</span><span style="color:lightgreen">:</span> <br>[RGB888 bytes](/)|Any|See [RGBA8888 bytes](/) or [RGB888 bytes](/) for more information.