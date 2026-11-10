### Node
|Data Type|Versions|Builds|Description|
|---------|--------|------|-----------|
|8-bit signed byte|Any|Any|Node type. See the [node types table](sticknodes-rs/spec/formats/nodes.md#node-types) for more information. Main node must be -1.
|32-bit signed int big-endian|Any|Any|Draw order index. lesser = behind, greater = in front. Acts as ID for node. Every node must have a unique draw order index. Main node must be 0.
|8-bit unsigned byte|Any|Any|Is static boolean
|8-bit unsigned byte|Any|Any|Is stretchy boolean
|8-bit unsigned byte|>=403|>=48|Is floaty boolean
|8-bit unsigned byte|>=248|Any|Is smart stretch boolean
|8-bit unsigned byte|>=252|Any|Do not apply smart stretch boolean
|8-bit unsigned byte|>=403|>=50|Smart stretch, reset impulse boolean
|8-bit unsigned byte|Any|Any|Use segment color boolean
|8-bit unsigned byte|>=256|Any|Use circle outline boolean
|8-bit unsigned byte|>=403|>=21|Circle is hollow boolean
|8-bit unsigned byte|>=176|Any|Use gradient boolean
|8-bit unsigned byte|>=176|Any|Reverse gradient boolean
|16-bit signed int (short big-endian)|>=403|>=20|Gradient mode
|8-bit unsigned byte|Any|Any|Use segment scale boolean
|32-bit float big-endian|Any|Any|Local X. The X offset from the parent node. Positive in rightward direction, negative in the leftward 
|32-bit float big-endian|Any|Any|Local Y. The Y offset from the parent node. Positive in upward direction, negative in the downward direction
|32-bit float big-endian|Any|Any|Scale
|32-bit float big-endian|Any|Any|Default length
|32-bit float big-endian|Any|Any|Length
|32-bit signed int big-endian|Any|Any|Default thickness
|32-bit signed int big-endian|Any|Any|Thickness
|32-bit signed int big-endian|>=320|Any|Segment curve radius & default curve radius
|8-bit unsigned byte|>=403|>=20|Curve "circulization" (circularization) boolean
|16-bit signed int (short) big-endian|>=403|>=21|Segment curve polyfill precision
|8-bit unsigned byte|>=256|Any|Half-arc boolean
|16-bit signed int (short) big-endian|>=256|Any|Right triangle direction
|8-bit unsigned byte|>=300|Any|Triangle upside-down boolean
|<span style="color:lightgreen">if <span style="color:coral">build</span></span> < <span style="color:indianred">64</span><span style="color:lightgreen">:</span><br> 32-bit signed int big-endian<br> <span style="color:lightgreen">else:</span><br> 32-bit float big-endian|>=403|>=36|Trapezoid thickness 1
|<span style="color:lightgreen">if <span style="color:coral">build</span></span> < <span style="color:indianred">64</span><span style="color:lightgreen">:</span><br> 32-bit signed int big-endian<br> <span style="color:lightgreen">else:</span><br> 32-bit float big-endian|>=403|>=36|Trapezoid thickness 2
|32-bit signed int big-endian|>=403|==36|[unused]
|32-bit signed int big-endian|>=403|==36|[unused]
|8-bit unsigned byte|>=403|==36|Use trapezoid thickness 1 boolean (I think?)
|8-bit unsigned byte|>=403|==36|Use trapezoid thickness 2 boolean (I think?)
|32-bit float big-endian|>=256|!=36|Trapezoid top thickness ratio
|8-bit unsigned byte|>=403|>=36|Trapezoid is rounded 1 boolean
|8-bit unsigned byte|>=403|>=36|Trapezoid is rounded 2 boolean
|16-bit signed int (short) big-endian|>=256|Any|Number of polygon vertices
|32-bit float big-endian|>=248|Any|Default local angle
|32-bit float big-endian|Any|Any|Local angle
|32-bit float big-endian|>=248|Any|Default angle
|32-bit signed int big-endian|Any|Any|Color. See [Color](sticknodes-rs/spec/structs/color.md) for more information.
|32-bit signed int big-endian|>=176|Any|Gradient color. See [Color](sticknodes-rs/spec/structs/color.md) for more information.
|32-bit signed int big-endian|>=256|Any|Circle outline color. See [Color](sticknodes-rs/spec/structs/color.md) for more information.
|8-bit unsigned byte|>=403|>=39|Is angle locked boolean. If build <= 50, forced to false.
|32-bit float big-endian|>=403|>=39 && <=50|[unused]
|8-bit unsigned byte|>=403|>=51|Angle lock is main node boolean. If build < 56, 0 means true.
|32-bit float big-endian|>=403|>=51 && <=56|The number being subtracted from to calculate the angle lock offset. (in 10 - 5, it would be 10).
|32-bit float big-endian|>=403|>=51 && <=56|The number being subtracted to calculate the angle lock offset. (in 10 - 5, it would be 5).
|32-bit float big-endian|>=403|>=57|Angle lock offset
|32-bit float big-endian|>=403|>=63|Angle lock relative start
|32-bit float big-endian|>=403|>=67|Angle lock stickfigure start
|8-bit signed byte|>=403|>=63|Angle lock relative multiplier
|<span style="color:lightgreen">if <span style="color:coral">build</span></span> <= <span style="color:indianred">40</span><span style="color:lightgreen">:</span><br> 16-bit signed int (short) big-endian<br> <span style="color:lightgreen">else:</span><br> 8-bit unsigned byte|>=403|>=39|Is drag locked boolean
|16-bit signed int (short) big-endian|>=403|>=41 && <=45|[unused]
|32-bit float big-endian|>=403|>=46|Drag lock angle
|32-bit float big-endian|>=403|>=41|Smart stretch multiplier
|8-bit unsigned byte|>=403|>=41 && <=45|[unused]
|32-bit signed int big-endian|Any|Any|<span style="color:coral">A</span> = Number of child nodes
|sequence with size of <span style="color:coral">A</span> of 32-bit signed int big-endian|>=403|>38|Sequence of ints representing booleans for whether each corresponding node is a connector
|sequence with size of <span style="color:coral">A</span> of [Node Data](sticknodes-rs/spec/formats/nodes.md#node-data)|Any|Any|See [Node Data](sticknodes-rs/spec/formats/nodes.md#node-data) for more information.