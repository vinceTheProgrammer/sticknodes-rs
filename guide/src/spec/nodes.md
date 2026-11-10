# .nodes File Format Specification
This specification is valid for SN versions <= 423.

## Metadata
|Data Type|Versions|Builds|Description|
|---------|--------|------|-----------|
|32-bit signed int big-endian|Any|Any|Stick Nodes version number. Must be between 0 and the latest version (inclusive).
|32-bit signed int big-endian|>=403|Any|Build number
|32-bit float big-endian|Any|Any|Stickfigure scale
|32-bit signed int big-endian|Any|Any|Stickfigure color. See the [color reference](sticknodes-rs/spec/structs/color.md) for more information.

## Nodes
|Data Type|Versions|Builds|Description|
|---------|--------|------|-----------|
|sequence of [Node](sticknodes-rs/spec/structs/node.md)|Any|Any|See [Node](sticknodes-rs/spec/structs/node.md) for more information.

## Polyfills
|Data Type|Versions|Builds|Description|
|---------|--------|------|-----------|
|32-bit signed int big-endian|>=230|Any|<span style="color:coral">B</span> = Number of polyfills
|sequence with size of <span style="color:coral">B</span> of [Polyfill](sticknodes-rs/spec/structs/polyfill.md)|>=230|Any|See [Polyfill](sticknodes-rs/spec/structs/polyfill.md) for more information.

## Connections
|Data Type|Versions|Builds|Description|
|---------|--------|------|-----------|
|32-bit signed int big-endian|>=403|>=38|<span style="color:coral">C</span> = Number of connector connections
|sequence with size of <span style="color:coral">C</span> of [Connection](sticknodes-rs/spec/structs/connection.md)|>=403|>=38|See [Connection](sticknodes-rs/spec/structs/connection.md) for more information.