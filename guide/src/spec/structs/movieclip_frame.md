### Movieclip Frame Data
|Data Type|Versions|Builds|Description|
|---------|--------|------|-----------|
|32-bit float big-endian|>=300|Any|Frame sound volume
|32-bit float big-endian|>=310|Any|Frame sound pan
|32-bit float big-endian|>=310|Any|Frame sound pitch
|32-bit float big-endian|>=300|Any|Frame sound library ID
|32-bit signed int big-endian|>=300|Any|<span style="color:coral">V</span> = 
|sequence with size of <span style="color:coral">V</span> of [movieclip frame asset data](/)|>=300|Any|See [movieclip frame asset data](/) for more information.

### Movieclip Frame Asset Data
|Data Type|Versions|Builds|Description|
|---------|--------|------|-----------|
|32-bit signed int big-endian|>=400|Any|<span style="color:coral">W</span> = asset type. (0 = stickfigure, 3 = sprite group, 2 = sprite)
|<span style="color:cornflowerblue">if <span style="color:coral">W</span> == 0: </span>[movieclip frame stickfigure](/)<br><span style="color:cornflowerblue">else if <span style="color:coral">W</span> == 3: </span>[movieclip frame sprite group](/)<br><span style="color:cornflowerblue">else if <span style="color:coral">W</span> == 2: </span>[movieclip frame sprite](/)|>=300|Any|See [movieclip frame stickfigure](/), [movieclip frame sprite group](/), or [movieclip frame sprite](/) for more information.

### Movieclip Frame Stickfigure
|Data Type|Versions|Builds|Description|
|---------|--------|------|-----------|
|32-bit signed int big-endian|>=300|Any|Library stickfigure ID

### Stickfigure Positional Data
|Data Type|Versions|Builds|Description|
|---------|--------|------|-----------|
|32-bit signed int big-endian|Any|Any|Stickfigure ID
|32-bit float big-endian|Any|Any|Stickfigure scale
|32-bit float big-endian|>=220|Any|Stickfigure transparency
|32-bit float big-endian|>=220|Any|Blur|Stickfigure blur
|8-bit unsigned byte|>=401|>=16|Is motion blur boolean
|8-bit unsigned byte|>=401|>=16|Motion blur is one direction boolean
|32-bit signed int big-endian|>=401|>=16|d,nvsakf

### Sprite Group Positional Data
|Data Type|Versions|Builds|Description|
|---------|--------|------|-----------|

### Sprite Positional Data
|Data Type|Versions|Builds|Description|
|---------|--------|------|-----------|

### Movieclip Frame Sprite Group
|Data Type|Versions|Builds|Description|
|---------|--------|------|-----------|
|32-bit signed int big-endian|>=300|Any|Library sprite group ID


### Movieclip Frame Sprite
|Data Type|Versions|Builds|Description|
|---------|--------|------|-----------|
|32-bit signed int big-endian|>=300|Any|Library sprite ID