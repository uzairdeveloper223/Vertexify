# VX Syntax Reference

## Structure of a VX file

A `.vx` file is a sequence of top-level declarations. Every declaration
is either a `let` binding or a `fn` definition.

```
Module  ::= Decl*
Decl    ::= LetDecl | FnDecl
```

---

## Declarations

### `let` — variable binding

```vx
let <name> [: <type>] = <expr>;
```

```vx
let radius: float = 2.5;
let label: str    = "hello";
let active: bool  = true;
```

### `fn` — function definition

```vx
fn <name>(<param>*) [-> <type>] <block>
```

```vx
fn area(r: float) -> float {
    3.14159 * r * r;
}
```

The value of the last expression in a block is its return value.
An explicit `return` also works:

```vx
fn clamp(v: float, lo: float, hi: float) -> float {
    if v < lo { return lo; }
    if v > hi { return hi; }
    v;
}
```

---

## Types

| Keyword | Description |
|---|---|
| `int` | 64-bit signed integer |
| `float` | 64-bit floating-point |
| `bool` | `true` or `false` |
| `str` | UTF-8 string |
| `vec2` | 2-component float vector |
| `vec3` | 3-component float vector |
| `vec4` | 4-component float vector |
| `mat4` | 4×4 float matrix |
| `color` | RGBA colour (4 floats, 0.0–1.0) |
| `geo` | Geometry object (mesh, text, primitive) |

---

## Expressions

### Literals

```vx
42          // int
3.14        // float
true        // bool
"hello"     // str
null        // null
```

### Arithmetic

| Op | Meaning |
|---|---|
| `+` | add (also string concat) |
| `-` | subtract |
| `*` | multiply |
| `/` | divide |
| `%` | remainder |
| `**` | power |
| `-x` | negate |

### Comparison

`==`  `!=`  `<`  `<=`  `>`  `>=`

### Logical

`&&`  `||`  `!`

### Compound assignment

`+=`  `-=`  `*=`  `/=`

### Field access

```vx
let v: vec3 = some_vec3;
let x: float = v.x;
let y: float = v.y;
let z: float = v.z;
```

Supported fields: `vec2.{x,y}` · `vec3.{x,y,z}` · `vec4.{x,y,z,w}` · `color.{r,g,b,a}`

### Function call

```vx
let result: float = area(2.5);
```

### Block expression

A block evaluates each statement in order. The value of the block is
the value of its last bare expression (without a trailing `;`), or
the argument to the first `return` statement encountered.

```vx
let doubled: int = {
    let x: int = 21;
    x * 2;
};
```

---

## Control flow

### `if` / `else`

```vx
let msg: str = if score > 90 {
    "excellent";
} else if score > 60 {
    "pass";
} else {
    "fail";
};
```

### `for` loop

```vx
for i in 10 {
    // body executes 10 times with i in 0..9
}
```

### `return`

```vx
fn sign(x: float) -> int {
    if x > 0.0 { return 1; }
    if x < 0.0 { return -1; }
    0;
}
```

---

## Text geometry

### `text2d` — flat 2-D text

Places a flat text object in the world. Returns `geo`.

```vx
let label: geo = text2d {
    content:     "Hello, World!",
    font:        "Inter",
    size:        24.0,
    color:       color(1.0, 1.0, 1.0, 1.0),
    align:       "center",
    position:    vec3(0.0, 0.0, 0.0),
    italic:      false,
    bold:        false,
    tracking:    1.0,
    line_height: 1.2,
    wrap_width:  0.0,
};
```

### `text3d` — extruded 3-D text

Creates solid extruded text geometry. All `text2d` keys are also valid plus:

```vx
let title: geo = text3d {
    content:          "Vertexify",
    font:             "Inter Bold",
    size:             1.0,
    depth:            0.2,
    bevel_depth:      0.02,
    bevel_resolution: 4,
    color:            color(0.8, 0.4, 0.1, 1.0),
    align:            "center",
    position:         vec3(0.0, 2.0, 0.0),
    rotation:         vec3(0.0, 0.0, 0.0),
};
```

#### All `text2d` / `text3d` properties

| Property | Type | Default | Description |
|---|---|---|---|
| `content` *(required)* | `str` | — | Text to render |
| `font` | `str` | `"Inter"` | Font family |
| `size` | `float` | `1.0` | Size (pts for 2-D, world units for 3-D) |
| `color` | `color` | white | RGBA fill colour |
| `align` | `str` | `"center"` | `"left"` \| `"center"` \| `"right"` |
| `position` | `vec3` | origin | World-space position |
| `italic` | `bool` | `false` | Italic style |
| `bold` | `bool` | `false` | Bold weight |
| `tracking` | `float` | `1.0` | Letter-spacing multiplier |
| `line_height` | `float` | `1.0` | Line-height multiplier |
| `wrap_width` | `float` | `0.0` | Word-wrap width; `0` = no wrap |
| `depth` *(3-D only)* | `float` | `0.1` | Extrusion depth |
| `bevel_depth` *(3-D only)* | `float` | `0.0` | Bevel radius |
| `bevel_resolution` *(3-D only)* | `int` | `4` | Bevel segments (1–16) |
| `rotation` *(3-D only)* | `vec3` | zero | Euler rotation in radians |

---

## Comments

```vx
// single-line comment

/* multi-line
   block comment */
```

---

## Lexical rules

- Identifiers: `[a-zA-Z_][a-zA-Z0-9_]*`
- Integers: `[0-9]+`
- Floats: `[0-9]+\.[0-9]+([eE][+-]?[0-9]+)?`
- Strings: `"..."` with `\"` `\\` `\n` `\t` escape sequences
- Keywords are reserved and cannot be used as identifiers.
