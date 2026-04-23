# VX Type System

## Overview

VX uses **static, inferred typing**. Every expression has a single type
that is determined at type-check time (before interpretation). You can
annotate a binding with `: <type>` for clarity, but the annotation is
optional — the checker infers the type from the right-hand side.

```vx
let x = 42;        // inferred int
let y: int = 42;   // same, explicit
```

---

## Built-in types

### Scalar types

| Type | Storage | Description |
|---|---|---|
| `int` | `i64` | 64-bit signed integer |
| `float` | `f64` | 64-bit IEEE-754 double |
| `bool` | 1 bit | `true` or `false` |
| `str` | UTF-8 | Immutable string |

### Vector and matrix types

| Type | Components | Field access |
|---|---|---|
| `vec2` | 2 × f32 | `.x` `.y` |
| `vec3` | 3 × f32 | `.x` `.y` `.z` |
| `vec4` | 4 × f32 | `.x` `.y` `.z` `.w` |
| `mat4` | 4×4 × f32 | — |

### Colour type

| Type | Components | Field access |
|---|---|---|
| `color` | 4 × f32 (RGBA) | `.r` `.g` `.b` `.a` |

All components are in the range `[0.0, 1.0]`.

### Geometry type

`geo` is the type of any geometry object produced by the scene DSL:
`text2d { ... }`, `text3d { ... }`, and (future) `mesh { ... }`,
`sphere { ... }`, etc.

### Function type

Functions are first-class values with type `fn(T1, T2, ...) -> R`.
You cannot write this type directly in VX source yet; it is used
internally by the type checker.

---

## Type rules for operators

### Arithmetic (`+` `-` `*` `/` `%`)

| Left | Right | Result |
|---|---|---|
| `int` | `int` | `int` |
| `float` | `float` | `float` |
| `int` | `float` | `float` |
| `float` | `int` | `float` |
| `str` | `str` | `str` (concatenation, `+` only) |

### Power (`**`)

Any numeric operand; always returns `float`.

### Comparison (`==` `!=` `<` `<=` `>` `>=`)

Comparing any two values of compatible numeric type or two `str` values
returns `bool`. `==` and `!=` accept any pair of same-type values.

### Logical (`&&` `||` `!`)

Both operands must be `bool`; result is `bool`.

---

## Coercion rules

VX does **not** silently coerce between types. An `int` will not
automatically become a `float` in a position that expects a `float`.

The single exception is arithmetic: `int OP float` and `float OP int`
are accepted and the `int` is widened to `float` for the operation.

---

## Type-checking text expressions

`text2d { ... }` and `text3d { ... }` always have type `geo`. The
property values inside the block are individually checked:

| Property | Expected type |
|---|---|
| `content` | `str` |
| `font` | `str` |
| `size` | `float` or `int` |
| `color` | `color` or `vec4` |
| `align` | `str` |
| `position` | `vec3` |
| `rotation` | `vec3` |
| `italic` | `bool` |
| `bold` | `bool` |
| `tracking` | `float` or `int` |
| `line_height` | `float` or `int` |
| `wrap_width` | `float` or `int` |
| `depth` | `float` or `int` |
| `bevel_depth` | `float` or `int` |
| `bevel_resolution` | `int` |

---

## Error messages

The type checker produces errors of the form:

```
type error: operator Add cannot be applied to Int and Str
type error: if branches have different types: Int vs Float
type error: function 'area' body type Int does not match declared return type Float
```

All errors carry the byte offset of the offending expression so the
editor can highlight the exact location.
