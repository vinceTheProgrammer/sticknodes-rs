### Session Save Data
Data representing the state of where the user left off in their last session in the project when saving.

|Data Type|Versions|Builds|Description|
|---------|--------|------|-----------|
|32-bit signed int big-endian|>251|Any|Frame index
|32-bit float big-endian|>251|Any|Guideline X (gets multiplied by App.assetScaling)
|32-bit float big-endian|>251|Any|Guideline Y (gets multiplied by App.assetScaling)
|32-bit float big-endian|>251|Any|Zoom
|32-bit float big-endian|>251|Any|Zoom position x (gets multiplied by App.assetScaling)
|32-bit float big-endian|>251|Any|Zoom position Y (gets multiplied by App.assetScaling)
|32-bit signed int big-endian|>251|Any|UV quality
|8-bit unsigned byte|>=300|Any|Only draw main nodes boolean
|8-bit unsigned byte|>=300|Any|Show figure IDs boolean
|8-bit unsigned byte|>251|Any|Onion skin is normal boolean
|8-bit unsigned byte|>251|Any|Is showing outline boolean
|8-bit unsigned byte|>=300|Any|Is showing outline creative boolean
|8-bit unsigned byte|>=310|Any|Is showing neighbor nodes boolean
|8-bit unsigned byte|>=310|Any|Is showing neighbor nodes creative boolean
|8-bit unsigned byte|>251|Any|Is showing guides boolean