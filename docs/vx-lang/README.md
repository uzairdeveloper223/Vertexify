# VX Language — Reference Manual

VX is the domain-specific scripting language built into Vertexify.
It lets you describe 3-D scenes, geometry operations, material definitions,
and procedural modelling logic in a concise, type-safe syntax.

## Quick navigation

| Document | Contents |
|---|---|
| [syntax.md](syntax.md) | Full grammar, operators, control flow |
| [types.md](types.md) | Built-in types, type rules, coercion |
| [builtins.md](builtins.md) | Built-in functions and scene keywords |
| [examples.md](examples.md) | Runnable example programs |

## Design goals

- **Readable first** — VX code should read like pseudocode; no ceremony.
- **Type-safe** — every expression has a static type; no implicit null or undefined.
- **iGPU friendly** — the language drives geometry/material construction that is
  uploaded to the GPU once and re-evaluated only when the script changes.
- **Sandboxed** — recursion depth, iteration count, and evaluation time are all
  bounded at runtime; a buggy script cannot hang or crash the host process.

## Hello world

```vx
// 1. A constant
let radius: float = 1.0;

// 2. A helper function (uses built-in PI)
fn sphere_volume(r: float) -> float {
    (4.0 / 3.0) * PI * r ** 3.0;
}

// 3. Compute a value
let vol: float = sphere_volume(radius);

// 4. A 2-D text label using the result
let label: geo = text2d {
    content:  "Vol: " + to_str(vol),
    font:     "Inter",
    size:     18.0,
    color:    color(1.0, 1.0, 1.0, 1.0),
    align:    "center",
    position: vec3(0.0, 0.0, 0.0),
};
```

The label automatically updates whenever you change `radius` and
press **Run** (or enable live evaluation in Settings).

## Execution model

1. The script is **lexed** → token stream.
2. The token stream is **parsed** → AST (`Module` of `Decl`s).
3. The AST is **type-checked** (`vx-lang::types::check_module`).
4. The checked AST is **interpreted** (`vx-lang::interpreter::eval_module`).
5. Any geometry/scene calls made during evaluation are queued into the
   `vx-scene` graph and rendered by `vx-render`.

## Limits (runtime safety)

| Guard | Default |
|---|---|
| Maximum call depth | 64 frames |
| Maximum loop iterations | 1 000 000 |
| Evaluation timeout | 30 s |
| Maximum triangle count | 10 000 000 |
