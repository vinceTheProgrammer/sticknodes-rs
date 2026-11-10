### Polyfill
|Data Type|Description|
|---------|-----------|
|32-bit signed int big-endian|Draw order index of parent node (becomes the anchor node)
|32-bit signed int big-endian|Color. See the [color reference](/sticknodes-rs/spec/structs/color.md) for more information.
|8-bit unsigned byte|Use polyfill color boolean
|32-bit signed int big-endian|<span style="color:coral">C</span> = Number of polyfill nodes (not counting the anchor node)
|sequence with size of <span style="color:coral">C</span> of 32-bit signed int big-endian|The draw order index of each node that the polyfill is attached to, in order.