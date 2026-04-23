# VX Language — Example Programs

All examples below are valid VX scripts ready to paste into the Vertexify
code editor. They are grouped by feature area and build in complexity.

---

## Basics

### 1 — Arithmetic and variables

```vx
let a: int   = 10;
let b: int   = 3;
let sum: int = a + b;   // 13
let rem: int = a % b;   // 1
let pow: float = 2.0 ** 8.0;  // 256.0
```

### 2 — String operations

```vx
let first: str = "Hello";
let last:  str = "World";
let msg:   str = first + ", " + last + "!";
let n:     int = len(msg);        // 13
let up:    str = upper(first);    // "HELLO"
```

### 3 — Boolean logic

```vx
fn is_valid(v: float) -> bool {
    v > 0.0 && v <= 100.0;
}

let ok:  bool = is_valid(50.0);   // true
let bad: bool = is_valid(-1.0);   // false
```

---

## Math built-ins

### 4 — Constants and trig

```vx
let full:    float = TAU;               // 6.2831...
let half:    float = PI;                // 3.1415...
let opp:     float = sin(PI / 6.0);    // 0.5
let adj:     float = cos(PI / 3.0);    // 0.5
let euler:   float = E;                 // 2.7182...
```

### 5 — Rounding and clamping

```vx
let f: float = 3.7;
let lo:    float = floor(f);              // 3.0
let hi:    float = ceil(f);               // 4.0
let near:  float = round(f);             // 4.0
let safe:  float = clamp(f, 0.0, 3.5);  // 3.5
```

### 6 — Interpolation and extremes

```vx
let mid:   float = lerp(0.0, 100.0, 0.25);  // 25.0
let small: float = min(7.0, 3.0);            // 3.0
let big:   float = max(7.0, 3.0);            // 7.0
```

### 7 — Geometry helpers

```vx
// Hypotenuse of a 3-4-5 right triangle
fn hyp(a: float, b: float) -> float {
    sqrt(a ** 2.0 + b ** 2.0);
}

let c: float = hyp(3.0, 4.0);   // 5.0

// Sphere surface area using PI
fn surface(r: float) -> float {
    4.0 * PI * r ** 2.0;
}

let area: float = surface(1.0);  // 12.566...
```

### 8 — Volume with built-in PI

```vx
fn sphere_volume(r: float) -> float {
    (4.0 / 3.0) * PI * r ** 3.0;
}

let vol: float = sphere_volume(2.0);  // 33.510...
```

---

## String built-ins

### 9 — Conversion and formatting

```vx
let radius: float = 2.5;
let tag:    str   = "r=" + to_str(radius);     // "r=2.5"
let parsed: float = to_float("3.14");           // 3.14
let count:  int   = to_int("42");               // 42
```

### 10 — Normalising input

```vx
fn normalise(s: str) -> str {
    trim(lower(s));
}

let clean: str = normalise("  VERTEXIFY  ");  // "vertexify"
```

---

## Control flow

### 11 — Conditional with built-ins

```vx
fn classify(score: float) -> str {
    let s: float = clamp(score, 0.0, 100.0);
    if s >= 90.0 { "excellent"; }
    else if s >= 60.0 { "pass"; }
    else { "fail"; }
}

let grade: str = classify(85.0);   // "pass"
```

### 12 — For loop with accumulator

```vx
fn factorial(n: int) -> float {
    let result: float = 1.0;
    for i in n {
        result = result * (i + 1);
    }
    result;
}

let f10: float = factorial(10);   // 3628800.0
```

### 13 — Recursive function

```vx
fn fib(n: int) -> int {
    if n <= 1 { n; }
    else { fib(n - 1) + fib(n - 2); }
}

let f7: int = fib(7);   // 13
```

---

## Text geometry

### 14 — Simple 2-D label

```vx
let label: geo = text2d {
    content:  "Hello, Vertexify!",
    font:     "Inter",
    size:     24.0,
    color:    color(1.0, 1.0, 1.0, 1.0),
    align:    "center",
    position: vec3(0.0, 0.0, 0.0),
};
```

### 15 — Dynamic 2-D label using built-ins

```vx
let radius: float = 2.0;
let vol:    float = (4.0 / 3.0) * PI * radius ** 3.0;
let label: geo = text2d {
    content:  "r=" + to_str(radius) + "  V=" + to_str(round(vol * 100.0) / 100.0),
    font:     "Inter",
    size:     18.0,
    color:    color(0.9, 0.9, 1.0, 1.0),
    align:    "center",
    position: vec3(0.0, -1.5, 0.0),
    bold:     true,
};
```

### 16 — Multi-line italic subtitle

```vx
let sub: geo = text2d {
    content:     "A 3D modeling\nplatform for Linux.",
    font:        "Inter Light",
    size:        18.0,
    color:       color(0.75, 0.75, 0.9, 1.0),
    align:       "center",
    position:    vec3(0.0, -0.6, 0.0),
    italic:      true,
    line_height: 1.4,
    wrap_width:  8.0,
};
```

### 17 — 3-D extruded logo

```vx
let logo: geo = text3d {
    content:          "VX",
    font:             "Inter Bold",
    size:             2.0,
    depth:            0.3,
    bevel_depth:      0.04,
    bevel_resolution: 6,
    color:            color(0.72, 0.45, 0.20, 1.0),
    align:            "center",
    position:         vec3(0.0, 1.5, 0.0),
    rotation:         vec3(0.0, 0.0, 0.0),
};
```

### 18 — Combined 2-D + 3-D title scene

```vx
// 3-D brand name
let brand: geo = text3d {
    content:  "Vertexify",
    font:     "Inter Bold",
    size:     1.5,
    depth:    0.25,
    color:    color(1.0, 0.6, 0.1, 1.0),
    position: vec3(0.0, 2.0, 0.0),
};

// 2-D tagline below
let tagline: geo = text2d {
    content:  "3D Modeling Platform",
    font:     "Inter",
    size:     20.0,
    color:    color(0.85, 0.85, 0.85, 1.0),
    position: vec3(0.0, 0.6, 0.0),
    align:    "center",
};

// 2-D version label bottom-right
let version: geo = text2d {
    content:  "v0.1.0",
    font:     "Inter Light",
    size:     12.0,
    color:    color(0.5, 0.5, 0.5, 1.0),
    align:    "right",
    position: vec3(4.5, -3.0, 0.0),
};
```

---

## Scientific / advanced

### 19 — Quadratic formula

```vx
fn quadratic_pos(a: float, b: float, c: float) -> float {
    (-b + sqrt(b ** 2.0 - 4.0 * a * c)) / (2.0 * a);
}

fn quadratic_neg(a: float, b: float, c: float) -> float {
    (-b - sqrt(b ** 2.0 - 4.0 * a * c)) / (2.0 * a);
}

// x^2 - 5x + 6 = 0  →  roots 2 and 3
let r1: float = quadratic_pos(1.0, -5.0, 6.0);   // 3.0
let r2: float = quadratic_neg(1.0, -5.0, 6.0);   // 2.0
```

### 20 — Degrees / radians helpers

```vx
fn deg_to_rad(d: float) -> float { d * PI / 180.0; }
fn rad_to_deg(r: float) -> float { r * 180.0 / PI; }

let angle_r: float = deg_to_rad(45.0);           // 0.785...
let angle_d: float = rad_to_deg(PI / 4.0);       // 45.0
```
