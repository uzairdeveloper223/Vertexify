# VX Built-in Functions and Scene Keywords

This document lists all built-in identifiers, constructor functions,
and scene-domain keywords available in every VX script.

> All functions and constants on this page are **fully implemented** as of Phase 1.5.

---

## Global constants

| Name | Type | Value |
|---|---|---|
| `PI` | `float` | 3.141592653589793 |
| `TAU` | `float` | 6.283185307179586 |
| `E` | `float` | 2.718281828459045 |
| `INF` | `float` | +∞ |

```vx
let half_turn: float = PI;
let full_turn: float = TAU;
```

---

## Constructor functions

### `vec2(x, y)`
```vx
let origin: vec2 = vec2(0.0, 0.0);
```

### `vec3(x, y, z)`
```vx
let up: vec3 = vec3(0.0, 1.0, 0.0);
```

### `vec4(x, y, z, w)`
```vx
let q: vec4 = vec4(0.0, 0.0, 0.0, 1.0);
```

### `color(r, g, b, a)`
All components are `float` in `[0.0, 1.0]`.
```vx
let red:  color = color(1.0, 0.0, 0.0, 1.0);
let semi: color = color(0.0, 0.4, 1.0, 0.75);
```

---

## Math functions

### 1-argument

| Function | Result type | Description |
|---|---|---|
| `abs(x)` | `float` | Absolute value (also accepts `int`) |
| `sqrt(x)` | `float` | Square root |
| `floor(x)` | `float` | Round down |
| `ceil(x)` | `float` | Round up |
| `round(x)` | `float` | Round to nearest |
| `sin(x)` | `float` | Sine (radians) |
| `cos(x)` | `float` | Cosine (radians) |
| `tan(x)` | `float` | Tangent (radians) |
| `log(x)` | `float` | Natural logarithm |
| `exp(x)` | `float` | e^x |

```vx
let hyp: float = sqrt(3.0 ** 2.0 + 4.0 ** 2.0);  // 5.0
let s:   float = sin(PI / 6.0);                    // 0.5
```

### 2-argument

| Function | Description |
|---|---|
| `pow(x, n)` | x raised to the power n (same as `x ** n`) |
| `atan2(y, x)` | Two-argument arctangent |
| `min(a, b)` | Smaller of two values |
| `max(a, b)` | Larger of two values |

```vx
let angle: float = atan2(1.0, 1.0);   // PI / 4
let small: float = min(3.0, 7.0);     // 3.0
```

### 3-argument

| Function | Description |
|---|---|
| `clamp(v, lo, hi)` | Clamp `v` to `[lo, hi]` |
| `lerp(a, b, t)` | Linear interpolation: `a + (b - a) * t` |

```vx
let clamped:    float = clamp(150.0, 0.0, 100.0);  // 100.0
let midpoint:   float = lerp(0.0, 10.0, 0.5);      // 5.0
```

---

## String functions

| Function | Signature | Description |
|---|---|---|
| `len(s)` | `str → int` | Length in characters |
| `upper(s)` | `str → str` | Uppercase |
| `lower(s)` | `str → str` | Lowercase |
| `trim(s)` | `str → str` | Strip leading/trailing whitespace |
| `to_str(v)` | `any → str` | Convert any value to its string representation |
| `to_int(s)` | `str → int` | Parse integer from string |
| `to_float(s)` | `str → float` | Parse float from string |

```vx
let n:    int   = len("hello");               // 5
let up:   str   = upper("vertexify");         // "VERTEXIFY"
let tag:  str   = "radius=" + to_str(1.5);   // "radius=1.5"
let val:  int   = to_int("42");               // 42
```

---

## Scene keywords

These are VX **language keywords** (not functions). They appear as the
leading token of text and geometry expressions.

| Keyword | Type returned | Description |
|---|---|---|
| `text2d { ... }` | `geo` | Flat 2-D text in world space |
| `text3d { ... }` | `geo` | Extruded 3-D text geometry |
| `mesh` | — | *(Phase 2 — not yet implemented)* |
| `material` | — | *(Phase 2 — not yet implemented)* |
| `object` | — | *(Phase 4 — not yet implemented)* |
| `light` | — | *(Phase 4 — not yet implemented)* |
| `camera` | — | *(Phase 4 — not yet implemented)* |
| `scene` | — | *(Phase 4 — not yet implemented)* |
| `import` | — | *(Phase 6 — not yet implemented)* |
| `export` | — | *(Phase 6 — not yet implemented)* |

See [syntax.md](syntax.md) for the full `text2d`/`text3d` property reference.
