### ID Conversion Maps Header
|Data Type|Versions|Builds|Description|
|---------|--------|------|-----------|
|32-bit signed int big-endian|>=300|Any|<span style="color:coral">G</span> = Number of destination containers
|sequence with size of <span style="color:coral">G</span> of [outer loop data](/spec/formats/stknds.md#ID-Conversion-Maps-Outer-Loop)|>=300|Any|See [outer loop data](/spec/formats/stknds.md#ID-Conversion-Maps-Outer-Loop) for more information.

### ID Conversion Maps Outer Loop (for each destination container)
|Data Type|Versions|Builds|Description|
|---------|--------|------|-----------|
|32-bit signed int big-endian|>=300|Any|<span style="color:coral">H</span> = Number of source containers
|32-bit signed int big-endian|>=300|Any|Destination container (timeline) UID
|sequence with size of <span style="color:coral">H</span> of [middle loop data](/spec/formats/stknds.md#ID-Conversion-Maps-Middle-Loop)|>=300|Any|See [middle loop data](/spec/formats/stknds.md#ID-Conversion-Maps-Middle-Loop) for more information.

### ID Conversion Maps Middle Loop (for each source container)
|Data Type|Versions|Builds|Description|
|---------|--------|------|-----------|
|32-bit signed int big-endian|>=300|Any|<span style="color:coral">I</span> = Number of ID mappings
|32-bit signed int big-endian|>=300|Any|Source container (timeline) UID
|sequence with size of <span style="color:coral">I</span> of [inner loop data](/spec/formats/stknds.md#ID-Conversion-Maps-Inner-Loop)|>=300|Any|See [inner loop data](/spec/formats/stknds.md#ID-Conversion-Maps-Inner-Loop) for more information.

### ID Conversion Maps Inner Loop (for each ID mapping)
|Data Type|Versions|Builds|Description|
|---------|--------|------|-----------|
|32-bit signed int big-endian|>=300|Any|Source container figure ID
|32-bit signed int big-endian|>=300|Any|Destination container figure ID

#### Explanation
When you paste a figure from one [container](/spec/formats/stknds.md#container) to another, Stick Nodes converts the ID of the figure from the source container to an ID within the destination container. If the figure was never pasted to the destination container before, it will assign a new unique ID to the figure within the destination container. If the figure was previously pasted to the destination container, it looks at the ID conversion map to know what ID to assign the figure within the destination container.

For a visual aid to help understand ID conversion maps, see [ID converstion maps visual aid](/spec/formats/stknds.md#id-conversion-maps-visual-aid).

#### Visual Aids
You can imagine the map datatypes of an example ID conversion map like this:

```
{
    int: {
        int: {
            int: int,
            int: int,
        },
        int: {
            int: int
        }
    },
    int: {
        int: {
            int: int,
            int: int,
            int: int
        }
    }
}
```

For a visual of what each value could represent:

```
{
    ID_of_main_animation_container: {
        ID_of_movieclip_1_container: {
            ID_of_figure_1_when_in_movieclip_1: ID_of_figure_1_when_in_main_animation,
            ID_of_figure_2_when_in_movieclip_1: ID_of_figure_2_when_in_main_animation,
        },
        ID_of_movieclip_2_container: {
            ID_of_figure_1_when_in_movieclip_2: ID_of_figure_1_when_in_main_animation
        }
    },
    ID_of_movieclip_1_container: {
        ID_of_main_animation_container: {
            ID_of_figure_1_when_in_main_animation: ID_of_figure_1_when_in_movieclip_1,
            ID_of_figure_2_when_in_main_animation: ID_of_figure_2_when_in_movieclip_1,
            ID_of_figure_3_when_in_main_animation: ID_of_figure_3_when_in_movieclip_1
        }
    }
}
```

For an idea of what actual data might look like:

```
{
    0: {
        1: {
            420: 69,
            1: 3,
        },
        2: {
            20: 20
        }
    },
    1: {
        0: {
            1: 3000,
            343: 222,
            10: 8
        }
    }
}
```