# vertexify — expanded architecture and implementation plan

vertexify is a 3D static modeling workspace for Linux. it is iGPU-first by design — delivering
hardware-accelerated rendering on integrated graphics (Intel UHD, AMD Radeon Vega, Apple M-series GPU)
without degrading on dGPUs. it pairs a traditional GUI with `vx` (vertex script), a domain-specific
language for procedural geometry generation. animation is out of scope by design — no rigs, no keyframes,
no timeline, no overhead from systems that don't serve static modeling.

---

## 1. guiding philosophy

- **iGPU-first, dGPU-compatible.** the primary target is Intel UHD 620/770, AMD Radeon 680M, and
  equivalents. all rendering decisions optimize for the shared-memory, bandwidth-constrained,
  thermally-limited constraints of integrated graphics. a dGPU will simply run faster, not differently.
- **predictable performance over peak performance.** frame delivery must be consistent. a 45fps
  steady-state is better than 120fps with 200ms hitches during CSG evaluation.
- **the DSL is a first-class citizen.** `vx` is not a scripting afterthought. it is the primary
  authoring interface. the GUI is its complement.
- **zero animation overhead.** no skeleton systems, no interpolation buffers, no timeline state machines.
  the scene graph is static. this removes an entire class of runtime complexity.

---

## 2. core architecture

### 2.1 language and runtime

- **rust 2021 edition.** safe concurrency, predictable allocation, no GC pauses, zero-cost abstractions.
  SIMD intrinsics available via `std::arch` or `wide`/`packed_simd2` for geometry hot paths.
- **single-binary target.** no electron, no nodejs, no external runtime. one binary, statically linked
  where possible, targeting `x86_64-unknown-linux-gnu` and `aarch64-unknown-linux-gnu`.
- **workspace layout:**
  ```
  vertexify/
  ├── crates/
  │   ├── vx-lang/        # lexer, parser, AST, interpreter, type system
  │   ├── vx-geometry/    # mesh types, CSG, primitives, spatial structures
  │   ├── vx-render/      # wgpu render pipeline, shader management
  │   ├── vx-scene/       # ECS, scene graph, material system
  │   ├── vx-ui/          # egui panels, viewport, editor widgets
  │   ├── vx-io/          # OBJ/STL/VXS import-export, project serialization
  │   └── vx-app/         # binary entry point, event loop, config
  └── Cargo.toml          # workspace root
  ```

### 2.2 GUI — egui

- immediate-mode, embeds directly into the `wgpu` render loop via `egui-wgpu`.
- no separate UI thread. no double-buffering of UI state. UI reads scene state directly.
- panels: viewport (primary), VX code editor, scene hierarchy, properties inspector, console/diagnostics.
- custom egui widgets for: material preview swatches, transform gizmo overlay, mesh stats bar.
- theming via `egui::Visuals` — dark-first, high-contrast accent for selection/hover states.
- font: monospace for the VX editor (`JetBrains Mono` embedded as a compiled bytes asset), proportional
  sans for all other UI.

### 2.3 rendering — wgpu (iGPU-optimized)

wgpu abstracts Vulkan, Metal, and DX12. on Linux the primary backend is **Vulkan**. the pipeline is
designed around the constraints of iGPU hardware:

#### 2.3.1 iGPU constraint model

| constraint | iGPU reality | design response |
|---|---|---|
| shared VRAM/RAM bandwidth | 25–60 GB/s vs 400+ GB/s dGPU | minimize GPU↔CPU transfers; stage uploads once |
| smaller L2/L3 cache | 4–8 MB vs 32–80 MB | tile geometry, use mesh shaders where available |
| thermal throttling under sustained load | clock drops 30–50% after 10–30s | adaptive LOD to keep GPU load under throttle threshold |
| no dedicated VRAM | OS manages memory pressure | use `wgpu::MemoryHints::MemoryUsage` for pressure hints |
| power budget | 15–28W TDP shared with CPU | workloads must be burstable, not continuous |

#### 2.3.2 render pipeline design

- **geometry pipeline:** indexed triangle lists. vertex buffer interleaved: `[position: vec3f, normal: vec3f, uv: vec2f, tangent: vec4f]`. 32-byte stride aligns to cache lines.
- **shader model:** WGSL throughout. no GLSL/SPIRV hand-written shaders. wgpu compiles WGSL→SPIRV at startup.
- **draw call batching:** all meshes with the same material are instanced into a single draw call via `draw_indexed_indirect`. iGPUs suffer more from draw call overhead than dGPUs.
- **uniform buffers over push constants:** iGPU drivers have inconsistent push constant performance. use uniform buffer objects with `wgpu::BindGroup` for all per-frame data.
- **texture atlasing:** material albedo/roughness/normal maps packed into texture atlases at load time. reduces bind group switches which are expensive on iGPU.
- **depth prepass:** single depth-only pass before the main shading pass. eliminates overdraw cost on complex scenes, which matters more when the shader units are shared with the CPU's execution units.
- **tile-based rendering awareness:** Intel UHD and AMD RDNA iGPUs use tile-based or hybrid tile/IMR renderers. avoid render pass breaks and mid-pass readbacks — they force tile flushes.
- **render resolution scaling:** expose a `render_scale` setting (0.5–1.0). native 1.0 for basic scenes, 0.75 for complex. bilinear upscale as a final blit. effectively doubles framerate budget on iGPU.
- **frame pacing:** target 60fps with a frame time budget of 16.6ms. if a frame would exceed 14ms, defer expensive operations (BVH rebuild, CSG remesh) to the next idle frame.
- **idle rendering:** when the scene is not modified and the camera is not moving, stop submitting frames. submit only on input event or scene change. iGPU thermal profile improves dramatically.

#### 2.3.3 shading model

- **PBR (Cook-Torrance BRDF):** GGX normal distribution, Smith-Schlick-GGX geometry, Schlick Fresnel.
  kept GPU-side at 40 ALU ops per fragment — within iGPU budget.
- **IBL (image-based lighting):** precomputed split-sum approximation. diffuse irradiance from
  9-coefficient SH, specular from 6-mip RGBE cubemap. no real-time light bounces — this is static modeling.
- **point lights:** maximum 16 per scene, packed into a UBO. forward rendering, no deferred — deferred
  G-buffers are bandwidth-heavy and hurt iGPU.
- **ambient occlusion:** screen-space AO (SSAO) via 8-sample hemisphere kernel in a half-res pass, then
  bilateral upscale. toggleable; costs ~0.8ms on Intel UHD 620.
- **wireframe overlay:** rendered as a second pass with polygon offset, not geometry shader lines.
  geometry shaders are slow on iGPU.
- **unlit mode:** available for pure geometry inspection. zero shading overhead.
- **matcap mode:** sphere-mapped matcap for fast clay-like preview. one texture sample per fragment.

### 2.4 scene management — custom ECS

a lightweight entity-component-system with no external ECS framework dependency (no `bevy_ecs`, no `hecs`).
vertexify's scene is static and small (<10,000 objects for realistic use) so a simple archetype-based
ECS without dynamic query planning is sufficient.

- **entities:** `u32` handles. dense array, no generation counter needed for a non-game scene graph.
- **components:** `Transform`, `Mesh`, `Material`, `Visibility`, `Name`, `VxBinding` (links to VX object ID).
- **scene graph:** parent-child transform hierarchy stored as a flat array of `(entity, parent_entity)` pairs.
  world-space transforms computed once per frame with a single DFS pass when any transform is dirty.
- **dirty tracking:** bitset per component type. only recompute world transforms and GPU uploads for dirty entities.
- **serialization:** scene serializes to `scene.vxs` (a versioned binary format using `bincode`) and/or
  a human-readable `.vx` project file that is the VX script itself.

### 2.5 math — glam

- `glam 0.25+` for all vector/matrix/quaternion math. SIMD-backed on x86_64 with AVX2.
- `Vec3`, `Mat4`, `Quat` — never raw arrays for geometry math.
- `glam::f32` for all rendering math. `glam::f64` only for precision-sensitive CSG operations.
- all transforms are column-major mat4, matching WGSL memory layout directly.

---

## 3. vertex script (vx) — the custom DSL

### 3.1 design principles

- **immutable-first.** operations return new geometry. no in-place mutation by default.
- **statically typed.** the type checker runs before the interpreter. type errors are reported with
  source spans, not runtime panics.
- **human-readable error messages.** the error system is inspired by Rust's `rustc` diagnostics —
  caret-pointed source excerpts with a clear explanation and a suggestion where possible.
- **deterministic.** given the same script, the same geometry is produced. no hidden randomness unless
  the `rand()` built-in is explicitly called with a seed.
- **incremental.** the interpreter tracks which VX expressions contributed to each mesh. when a line
  changes, only the affected subgraph is re-evaluated.

### 3.2 type system

primitive value types:
```
float       # f64 internally, f32 at GPU boundary
int         # i64
bool        # true / false
string      # UTF-8, immutable
vec2        # (x: float, y: float)
vec3        # (x: float, y: float, z: float)
vec4        # (x: float, y: float, z: float, w: float)
color       # (r: float, g: float, b: float, a: float) — also accepts "#RRGGAA" hex
```

geometry types (reference-counted handles to mesh data):
```
Mesh        # indexed triangle mesh — base geometry type
Solid       # CSG solid — lazy tree of union/difference/intersection nodes
Curve       # 2D curve for sweep/extrude operations
Surface     # parametric surface patch (Bezier, NURBS)
```

compound types:
```
list<T>     # growable array
map<K, V>   # string-keyed dictionary
option<T>   # Some(value) | None
```

### 3.3 full syntax reference

#### 3.3.1 declarations and binding

```vx
let name = expr                    # immutable binding (always prefer)
var name = expr                    # mutable binding (use only for accumulators)
const PI = 3.14159265              # compile-time constant — must be a literal expression
fn name(param: Type, ...) -> Type  # named function
```

#### 3.3.2 primitives

```vx
cube(w: float, h: float, d: float)
  -> Mesh

sphere(r: float, segments: int, rings: int)
  -> Mesh

cylinder(r: float, h: float, segments: int)
  -> Mesh

cone(r_base: float, r_top: float, h: float, segments: int)
  -> Mesh

plane(w: float, d: float, subdivisions_w: int, subdivisions_d: int)
  -> Mesh

torus(r_major: float, r_minor: float, segments_major: int, segments_minor: int)
  -> Mesh

capsule(r: float, h: float, segments: int)
  -> Mesh

tetrahedron(size: float) -> Mesh
octahedron(size: float)  -> Mesh
icosphere(r: float, subdivisions: int) -> Mesh   # geodesic sphere, max subdivisions: 6
grid(cols: int, rows: int, cell_size: float) -> Mesh
```

#### 3.3.3 transforms

transforms are non-destructive. they attach to a mesh and are composed lazily.

```vx
mesh.translate(x: float, y: float, z: float) -> Mesh
mesh.rotate(axis: vec3, angle_deg: float)    -> Mesh
mesh.rotate_euler(x_deg: float, y_deg: float, z_deg: float) -> Mesh
mesh.scale(x: float, y: float, z: float)    -> Mesh
mesh.scale_uniform(factor: float)            -> Mesh
mesh.mirror(axis: vec3)                      -> Mesh
mesh.align(from: vec3, to: vec3)             -> Mesh   # rotates mesh so `from` aligns with `to`
mesh.reset_origin()                          -> Mesh   # moves origin to geometry centroid
mesh.set_origin(pos: vec3)                   -> Mesh
```

transform matrix composition:
```vx
let mat = transform_matrix(
  translate: vec3(1.0, 0.0, 0.0),
  rotate_euler: vec3(0.0, 45.0, 0.0),
  scale: vec3(1.0, 2.0, 1.0)
)
let shaped = mesh.apply_matrix(mat)
```

#### 3.3.4 constructive solid geometry (CSG)

CSG operations build a lazy evaluation tree. the mesh is not computed until `spawn()` or
`bake()` is called. this allows the engine to optimize the tree before evaluation.

```vx
union(a: Solid, b: Solid, ...) -> Solid        # additive merge
difference(a: Solid, b: Solid) -> Solid        # subtracts b from a
intersection(a: Solid, b: Solid) -> Solid      # keeps only overlapping volume
xor(a: Solid, b: Solid)        -> Solid        # symmetric difference (union minus intersection)

as_solid(mesh: Mesh) -> Solid                  # promotes a Mesh to a CSG Solid
bake(solid: Solid)   -> Mesh                   # forces CSG evaluation, returns triangle mesh
```

CSG is implemented via the **Cork** algorithm (vertex-exact boolean operations on triangle meshes)
wrapped in a Rust port. the implementation guarantees manifold output on manifold input. non-manifold
inputs produce a compile-time warning and a runtime fallback to approximate booleans.

#### 3.3.5 mesh operations

```vx
mesh.subdivide(levels: int)                    -> Mesh   # Catmull-Clark subdivision
mesh.triangulate()                             -> Mesh   # converts quads/ngons to tris
mesh.smooth_normals(angle_threshold_deg: float)-> Mesh   # auto-smooth by angle
mesh.flat_normals()                            -> Mesh   # face normals, hard edges
mesh.weld_vertices(threshold: float)           -> Mesh   # merges verts within distance
mesh.decimate(target_ratio: float)             -> Mesh   # quadric error mesh simplification
mesh.invert_normals()                          -> Mesh
mesh.fill_holes()                              -> Mesh   # fills boundary loops as fans
mesh.solidify(thickness: float, offset: float) -> Mesh   # shell offset
mesh.bevel_edges(width: float, segments: int, profile: float) -> Mesh
mesh.bevel_vertices(width: float, segments: int) -> Mesh
mesh.extrude_faces(face_sel: FaceSelection, offset: vec3) -> Mesh
mesh.loop_cut(edge_sel: EdgeSelection, cuts: int, slide: float) -> Mesh
```

#### 3.3.6 procedural generation

```vx
# scatter instances across a surface
scatter(
  target: Mesh,
  source: Mesh,
  count: int,
  seed: int,
  align_to_normal: bool,
  min_distance: float
) -> list<Mesh>

# linear array
array_linear(
  mesh: Mesh,
  count: int,
  offset: vec3,
  merge_at_threshold: option<float>
) -> Mesh

# radial array
array_radial(
  mesh: Mesh,
  count: int,
  center: vec3,
  axis: vec3,
  angle_deg: float,
  merge: bool
) -> Mesh

# curve array — distribute copies along a curve
array_curve(
  mesh: Mesh,
  path: Curve,
  count: int,
  align_to_curve: bool
) -> Mesh

# noise displacement
displace(
  mesh: Mesh,
  strength: float,
  scale: float,
  seed: int,
  noise_type: NoiseType   # Perlin | Simplex | Voronoi | White
) -> Mesh

# vertex color from noise
vertex_color_noise(
  mesh: Mesh,
  attribute: string,
  scale: float,
  seed: int
) -> Mesh
```

#### 3.3.7 curves and surfaces

```vx
bezier(points: list<vec3>, resolution: int) -> Curve
nurbs(points: list<vec3>, weights: list<float>, degree: int, resolution: int) -> Curve
polyline(points: list<vec3>) -> Curve
circle_curve(r: float, segments: int) -> Curve

curve.extrude(profile: Curve) -> Mesh           # sweep profile along curve
curve.loft(profile_a: Curve, profile_b: Curve, steps: int) -> Mesh
curve.revolve(axis: vec3, angle_deg: float, segments: int) -> Mesh
curve.to_mesh(bevel_depth: float, bevel_resolution: int) -> Mesh
```

#### 3.3.8 UV and attributes

```vx
mesh.uv_project(method: UVMethod) -> Mesh
  # UVMethod: Planar(axis: vec3) | Cylindrical | Spherical | Box | Smart | Unwrap

mesh.uv_scale(u: float, v: float) -> Mesh
mesh.uv_offset(u: float, v: float) -> Mesh
mesh.uv_rotate(angle_deg: float)  -> Mesh

mesh.set_vertex_attribute(name: string, values: list<float>)  -> Mesh
mesh.set_face_attribute(name: string, values: list<float>)    -> Mesh
mesh.get_vertex_count()     -> int
mesh.get_face_count()       -> int
mesh.get_edge_count()       -> int
mesh.bounding_box()         -> (min: vec3, max: vec3)
mesh.centroid()             -> vec3
mesh.surface_area()         -> float
mesh.volume()               -> float                # manifold meshes only
```

#### 3.3.9 materials

```vx
material(
  id: string,
  color: color,
  roughness: float,         # 0.0 (mirror) – 1.0 (fully diffuse)
  metallic: float,          # 0.0 – 1.0
  emission: color,          # emission color; emission_strength scales it
  emission_strength: float,
  ior: float,               # index of refraction (glass: 1.45, water: 1.33)
  transmission: float,      # 0.0 (opaque) – 1.0 (fully transmissive)
  normal_map: option<string>,   # file path or embedded asset id
  roughness_map: option<string>,
  albedo_map: option<string>
) -> Material

mesh.set_material(mat: Material) -> Mesh
mesh.set_face_material(face_sel: FaceSelection, mat: Material) -> Mesh
```

#### 3.3.10 selection

```vx
select_faces_by_normal(mesh: Mesh, direction: vec3, threshold_deg: float) -> FaceSelection
select_faces_by_material(mesh: Mesh, mat_id: string) -> FaceSelection
select_boundary_edges(mesh: Mesh) -> EdgeSelection
select_sharp_edges(mesh: Mesh, angle_deg: float) -> EdgeSelection
select_vertices_by_attribute(mesh: Mesh, attr: string, min: float, max: float) -> VertexSelection
invert_selection(sel: FaceSelection | EdgeSelection | VertexSelection) -> same
```

#### 3.3.11 control flow and functions

```vx
# conditionals
if condition {
  expr
} else if condition {
  expr
} else {
  expr
}

# match on values (exhaustive on enums)
match value {
  Pattern => expr,
  Pattern => expr,
  _ => expr
}

# loops
for item in list {
  expr
}

for i in 0..count {
  expr
}

while condition {
  expr
}

# early exit from loop
break
continue

# functions
fn make_pillar(height: float, radius: float) -> Mesh {
  let shaft = cylinder(r: radius, h: height * 0.85, segments: 16)
  let cap   = sphere(r: radius * 1.15, segments: 16, rings: 8)
              .scale_uniform(0.3)
              .translate(0.0, height * 0.85, 0.0)
  bake(union(as_solid(shaft), as_solid(cap)))
}

# closures
let scale_fn = |m: Mesh, factor: float| -> Mesh { m.scale_uniform(factor) }

# recursion is permitted up to a depth of 64
fn sierpinski(mesh: Mesh, depth: int) -> Mesh {
  if depth == 0 { return mesh }
  let s = mesh.scale_uniform(0.5)
  let a = s.translate(-0.5, 0.0, 0.0)
  let b = s.translate(0.5, 0.0, 0.0)
  let c = s.translate(0.0, 1.0, 0.0)
  union_all([as_solid(sierpinski(a, depth - 1)),
             as_solid(sierpinski(b, depth - 1)),
             as_solid(sierpinski(c, depth - 1))]).bake()
}
```

#### 3.3.12 built-in math and utilities

```vx
abs(x)    sin(x)    cos(x)    tan(x)    asin(x)   acos(x)   atan2(y, x)
sqrt(x)   pow(x, e) log(x)    log2(x)   exp(x)
floor(x)  ceil(x)   round(x)  fract(x)
min(a, b) max(a, b) clamp(x, lo, hi)   lerp(a, b, t)   smoothstep(e0, e1, x)
sign(x)   mod(x, y) remap(v, in_lo, in_hi, out_lo, out_hi)
dot(a, b) cross(a, b) normalize(v) length(v) distance(a, b) reflect(v, n)
rand(seed: int) -> float                    # deterministic PRNG, always seeded
rand_vec3(seed: int) -> vec3
noise_perlin(pos: vec3, scale: float) -> float
noise_voronoi(pos: vec3, scale: float) -> float
deg_to_rad(d) rad_to_deg(r)
```

#### 3.3.13 scene output

```vx
spawn(mesh: Mesh | Solid, pos: vec3, name: option<string>)
spawn_many(meshes: list<Mesh>, base_pos: vec3)
group(meshes: list<Mesh>, name: string) -> Group
export_obj(mesh: Mesh, path: string)
export_stl(mesh: Mesh, path: string, binary: bool)
export_ply(mesh: Mesh, path: string)
export_gltf(group: Group, path: string)     # binary glTF 2.0
print(value: any)                           # outputs to the console panel
assert(condition: bool, msg: string)        # hard error if false — useful for procedural validation
```

#### 3.3.14 full example — production-style VX script

```vx
const TOWER_HEIGHT    = 18.0
const FLOOR_COUNT     = 6
const FLOOR_HEIGHT    = TOWER_HEIGHT / float(FLOOR_COUNT)
const BASE_RADIUS     = 2.5
const WINDOW_W        = 0.35
const WINDOW_H        = 0.55

let mat_concrete = material(
  id: "concrete",
  color: color(0.62, 0.60, 0.58, 1.0),
  roughness: 0.9,
  metallic: 0.0,
  emission: color(0.0, 0.0, 0.0, 1.0),
  emission_strength: 0.0,
  ior: 1.5,
  transmission: 0.0,
  normal_map: None,
  roughness_map: None,
  albedo_map: None
)

let mat_glass = material(
  id: "glass",
  color: color(0.7, 0.85, 0.9, 0.15),
  roughness: 0.05,
  metallic: 0.0,
  emission: color(0.0, 0.0, 0.0, 1.0),
  emission_strength: 0.0,
  ior: 1.52,
  transmission: 0.92,
  normal_map: None,
  roughness_map: None,
  albedo_map: None
)

fn make_floor_slab(y: float) -> Mesh {
  cube(w: BASE_RADIUS * 2.2, h: 0.18, d: BASE_RADIUS * 2.2)
    .translate(0.0, y, 0.0)
    .set_material(mat_concrete)
}

fn make_window(x: float, z: float, y: float) -> Mesh {
  cube(w: WINDOW_W, h: WINDOW_H, d: 0.06)
    .translate(x, y, z)
    .set_material(mat_glass)
}

let column = cylinder(r: 0.18, h: TOWER_HEIGHT, segments: 8)
              .set_material(mat_concrete)

var floors: list<Mesh> = []
var i = 0
while i < FLOOR_COUNT {
  let y = float(i) * FLOOR_HEIGHT
  floors = floors + [make_floor_slab(y + FLOOR_HEIGHT)]

  for angle in [0.0, 90.0, 180.0, 270.0] {
    let rad = deg_to_rad(angle)
    let wx = cos(rad) * (BASE_RADIUS - 0.05)
    let wz = sin(rad) * (BASE_RADIUS - 0.05)
    floors = floors + [make_window(wx, wz, y + FLOOR_HEIGHT * 0.5)]
  }

  i = i + 1
}

let base = cube(w: BASE_RADIUS * 2.4, h: 0.5, d: BASE_RADIUS * 2.4)
            .set_material(mat_concrete)
let roof = cube(w: BASE_RADIUS * 1.8, h: 0.3, d: BASE_RADIUS * 1.8)
            .translate(0.0, TOWER_HEIGHT + 0.15, 0.0)
            .set_material(mat_concrete)

let tower_group = group(
  [base, column] + floors + [roof],
  name: "procedural_tower"
)

spawn_many(tower_group.meshes, vec3(0.0, 0.0, 0.0))
export_gltf(tower_group, "tower_export.glb")
```

### 3.4 interpreter architecture

#### 3.4.1 pipeline

```
source text
    │
    ▼
┌──────────┐
│  Lexer   │  logos 0.14 — zero-copy, SIMD-accelerated token scanning
└──────────┘
    │ token stream
    ▼
┌──────────┐
│  Parser  │  hand-written recursive descent — better error recovery than generated parsers
└──────────┘
    │ untyped AST
    ▼
┌──────────────┐
│  Type Checker│  bidirectional type inference — Hindley-Milner lite
└──────────────┘
    │ typed AST
    ▼
┌────────────────────┐
│  Dependency Resolver│  builds a DAG of geometry operations
└────────────────────┘
    │ ordered evaluation plan
    ▼
┌─────────────┐
│  Interpreter │  tree-walk — geometry ops are the bottleneck, not the walk overhead
└─────────────┘
    │ scene mutations
    ▼
  scene graph → renderer
```

#### 3.4.2 error reporting

every error includes: file path, line, column, caret-pointed source excerpt, and a human-readable
message. suggestions are emitted when the wrong type is close to the expected type or when a function
is called with wrong argument count.

```
error[E0042]: type mismatch
  --> scene.vx:14:28
   |
14 |   let shaft = cylinder(r: "2.0", h: height, segments: 16)
   |                            ^^^^^ expected `float`, found `string`
   |
   = suggestion: remove the quotes — `r: 2.0`
```

#### 3.4.3 incremental re-evaluation

- each VX binding gets a `content_hash: u64` (xxhash3 of the expression text).
- a `DependencyGraph` maps each binding to the set of bindings it reads.
- on source change, the editor diffs the token stream, identifies changed bindings, and marks their
  downstream dependents dirty.
- only dirty nodes are re-evaluated. a `spawn()` of an unmodified mesh is a no-op.
- CSG trees are re-evaluated lazily: changing a leaf parameter re-evaluates only the minimal subtree.

#### 3.4.4 parallel evaluation

- independent branches of the dependency DAG are dispatched to `rayon` thread pool.
- CSG evaluation of `difference(a, b)` when `a` and `b` are independent spawns two tasks.
- maximum parallelism is capped at `(logical_core_count / 2)` to leave headroom for the render thread.
- geometry operations hold no locks. they take owned `Mesh` values and produce new owned `Mesh` values.

---

## 4. geometry subsystem

### 4.1 mesh data structure

half-edge mesh for all topological operations (subdivision, bevel, fill holes, extrude).
indexed face set (triangle soup) for rendering upload. conversion between the two is explicit.

```rust
struct HalfEdgeMesh {
    vertices:  Vec<Vertex>,
    half_edges: Vec<HalfEdge>,
    faces:     Vec<Face>,
}

struct Vertex {
    position:  Vec3,
    normal:    Vec3,
    uv:        Vec2,
    tangent:   Vec4,
}

struct HalfEdge {
    vertex:    u32,
    next:      u32,
    prev:      u32,
    twin:      Option<u32>,
    face:      u32,
}
```

indexed face set for GPU upload:
```rust
struct GpuMesh {
    vertices: Vec<GpuVertex>,    // 32 bytes/vertex: pos(12) + norm(12) + uv(8)
    indices:  Vec<u32>,
}
```

### 4.2 spatial acceleration — BVH

- **build algorithm:** SAH (surface area heuristic) binned BVH. O(n log n) build.
- **node layout:** 32-byte cache-line-aligned BVH nodes. left child implicit (index+1), right child stored.
- **use cases:** ray-mesh intersection for mouse picking, frustum culling, CSG proximity tests.
- **rebuild policy:** full BVH rebuild only on mesh add/remove. incremental refit on transform change.
- **storage:** one BVH per static mesh group, one top-level BVH over all instance AABBs.

### 4.3 CSG implementation

- backend: port of the **Cork** boolean mesh library to Rust (or `manifold` via its C API via `bindgen`).
- CSG trees evaluate bottom-up. leaf nodes are `Mesh`. interior nodes are `union | difference | intersection | xor`.
- triangle intersection uses exact arithmetic (integer arithmetic on quantized coordinates) to avoid
  floating-point inconsistencies that produce non-manifold output.
- post-CSG: mesh is re-welded, normals recomputed, and topology validated (manifold check) before
  returning to VX.

### 4.4 mesh operations — implementation notes

- **Catmull-Clark subdivision:** half-edge traversal. each level quadruples face count. warn at level 4
  (typically 16× input faces).
- **quadric error simplification (decimate):** Garland-Heckbert QEM. builds a `priority_queue` of
  edge collapse costs. target ratio is respected within 5%.
- **scatter:** surface area-weighted random face sampling (reservoir sampling), then project to face center
  + normal * offset. minimum distance enforced via a `k-d tree` of placed positions.
- **noise displacement:** Perlin/Simplex via `noise` crate. per-vertex displacement along normal direction.
- **bevel:** half-edge split and slide. profile curve is sampled at `segments` intervals for custom
  concave/convex bevel shapes.

---

## 5. material and asset system

### 5.1 material model

materials are PBR-compliant. internal representation:
```rust
struct Material {
    id:                String,
    base_color:        Vec4,        // linear RGBA
    roughness:         f32,
    metallic:          f32,
    emission:          Vec3,
    emission_strength: f32,
    ior:               f32,
    transmission:      f32,
    albedo_map:        Option<TextureHandle>,
    roughness_map:     Option<TextureHandle>,
    normal_map:        Option<TextureHandle>,
    ao_map:            Option<TextureHandle>,
}
```

### 5.2 texture management

- textures uploaded to GPU once via `wgpu::Queue::write_texture`.
- all textures packed into a 4096×4096 atlas per material group on scene load. atlas uses shelf packing.
- mipmap generation on the GPU via `wgpu` with a blit chain.
- texture cache keyed by file path hash. duplicate paths reuse the same `wgpu::Texture`.
- formats: PNG, JPEG, EXR (for HDR environment maps). loaded via `image` crate.

### 5.3 environment lighting

- HDRI environment map loaded as equirectangular, converted to cubemap on GPU via a compute shader.
- diffuse irradiance SH9 coefficients precomputed from cubemap in a 32-sample compute pass at startup.
- specular GGX prefiltered environment mip-chain generated in a compute pass (6 mip levels, roughness 0→1).
- BRDF LUT (split-sum) is precomputed once and embedded as a compiled asset (128×128 float16 texture).

---

## 6. reliability and error handling architecture

### 6.1 failure taxonomy

| failure class | examples | strategy |
|---|---|---|
| VX user error | type mismatch, undefined variable | structured diagnostic, no panic |
| VX logic error | CSG on non-manifold, recursion depth exceeded | runtime error with source span |
| geometry invariant violation | degenerate mesh after op | sanitize + warn, never crash |
| GPU resource exhaustion | OOM on texture atlas | evict LRU textures, log, degrade gracefully |
| file I/O failure | missing asset, corrupt scene file | `Result<>` propagation, user-facing error panel |
| rendering pipeline failure | wgpu device lost | attempt device recovery, fall back to software |
| internal assertion | programmer error, unreachable code | panic with full backtrace in debug, abort in release |

### 6.2 VX runtime safety

- **stack depth limit:** VX function call depth capped at 64. exceeding it returns a structured error,
  not a Rust stack overflow.
- **evaluation timeout:** a background timer cancels VX evaluation after 30 seconds. user is notified.
  the scene is left in its last valid state.
- **geometry size limit:** meshes over 10M triangles emit a warning at 8M and refuse further subdivision
  or CSG ops at 10M. the limit is configurable in `vertexify.toml`.
- **infinite loop detection:** loops that have executed more than `1_000_000` iterations without
  terminating emit a warning at 900k and hard-stop at 1M. the iteration limit is configurable.
- **panic isolation:** VX evaluation runs inside `std::panic::catch_unwind`. a Rust panic in a geometry
  op (e.g., in the Cork FFI) is caught, logged, and converted to a VX runtime error. the main thread
  is never taken down by a geometry operation.
- **input validation:** all VX built-ins validate inputs before passing to Rust. `cylinder(r: -1.0)`
  returns a VX error before any Rust geometry code runs.

### 6.3 rendering reliability

- **wgpu device lost recovery:** on `wgpu::SurfaceError::Lost` or `DeviceLost`, the renderer attempts
  a full device re-initialization. all GPU buffers are re-uploaded from the in-memory scene state.
  the scene state is always authoritative; GPU state is derived.
- **frame timeout watchdog:** if a frame takes longer than 500ms to submit, the renderer logs a warning
  and skips pending work. this prevents UI lockup during heavy CSG evaluation on iGPU.
- **graceful shader compilation failure:** if a WGSL shader fails to compile (driver bug, unsupported
  feature), the renderer falls back to an unlit flat-color pipeline rather than crashing.
- **adaptive quality:** a rolling frame time average is computed over 60 frames. if the P95 frame time
  exceeds 20ms (sub-50fps), `render_scale` is automatically reduced by 0.1 (floor: 0.5) and the user
  is notified. it recovers when P95 drops below 14ms.

### 6.4 project persistence and crash recovery

- **autosave:** the VX script source is written to `~/.local/share/vertexify/autosave.vx` every 30
  seconds and on every successful evaluation. this is the ground truth; the binary scene cache is
  derived.
- **crash recovery:** on startup, if `autosave.vx` is newer than `last_clean_exit` timestamp, the
  recovery dialog is shown.
- **undo/redo:** the VX script is the undo stack. each evaluation checkpoint stores the full source
  text (not a diff). 100-checkpoint ring buffer. undo reverts to the previous checkpoint and
  re-evaluates.
- **scene versioning:** `.vxs` binary scene format includes a `schema_version: u32` field. forward
  compatibility: unknown fields are ignored. backward compatibility: migrations are run for older versions.

### 6.5 logging and diagnostics

- structured logging via `tracing` crate with `tracing-subscriber`.
- log targets: `vertexify::render`, `vertexify::vx`, `vertexify::geometry`, `vertexify::io`.
- in release builds: `INFO` and above written to `~/.local/share/vertexify/vertexify.log` with
  rotation at 10MB (3 files kept).
- in debug builds: `TRACE` and above to stderr with file/line.
- the console panel in the UI shows VX `print()` output and runtime warnings at `INFO` level.
- GPU diagnostics: frame time, draw call count, VRAM used, triangle count are always tracked and
  available in the diagnostics overlay (toggled with `F2`).
- **telemetry:** opt-in, disabled by default. if enabled, sends anonymous crash reports (backtrace +
  OS + GPU vendor) to a self-hosted endpoint. no scene data ever leaves the machine.

---

## 7. I/O and interoperability

### 7.1 export formats

| format | use case | notes |
|---|---|---|
| OBJ + MTL | universal interchange | materials as MTL, per-object groups |
| STL (binary) | 3D printing slicers | manifold validation before export |
| PLY | point clouds, color data | per-vertex attributes preserved |
| glTF 2.0 (binary .glb) | game engines, Blender, web viewers | PBR materials, groups as nodes |
| VXS (native binary) | native scene format | `bincode`-serialized, versioned |
| VX script (.vx) | source-of-truth project file | human-readable, version-controlled |

### 7.2 import formats

| format | notes |
|---|---|
| OBJ | geometry + MTL materials |
| STL | binary and ASCII |
| PLY | binary and ASCII |
| glTF 2.0 | geometry only in v1; materials in v1.1 |

imported meshes are exposed in VX as `Mesh` values via:
```vx
let imported = import_mesh("path/to/model.obj")
```

### 7.3 project format — VXS

`.vxs` is the binary project format. it stores:
- the full VX source text
- the evaluated scene graph (mesh geometry, transforms, materials)
- the BVH structures (for fast load without recompute)
- the camera state
- viewport configuration

on save: scene is serialized to a temp file, then atomically renamed to the target path to avoid
corruption on write failure.

---

## 8. viewport and camera

- **orbit camera:** tumble (left drag), pan (middle drag / shift+left drag), dolly (scroll).
- **camera projection:** perspective (default, 35mm equivalent FOV 60°) and orthographic.
- **navigation shortcuts:** numpad 1/3/7 for front/side/top ortho views. numpad 5 toggles
  persp/ortho. numpad 0 for camera view. `F` to frame selected object.
- **gizmos:** transform gizmo (translate/rotate/scale handles) rendered as a second egui layer over
  the viewport. selection via ray-BVH intersection.
- **object picking:** ray cast against top-level BVH on mouse click. result highlights in scene
  hierarchy and loads properties into the inspector panel.
- **multi-selection:** shift+click adds to selection. box select (right-drag).
- **viewport grid:** rendered as a full-screen quad with a WGSL grid shader (analytical, no geometry).
  major/minor grid lines at configurable intervals.
- **view modes:** solid (PBR), matcap, wireframe, wireframe-on-solid, unlit, normals, UVs.

---

## 9. UI panel layout

```
┌────────────────────────────────────────────────────────┬─────────────────┐
│                                                        │  Scene Hierarchy│
│                                                        │  ─────────────  │
│                   3D Viewport                          │  ▶ tower_group  │
│                   (wgpu surface)                       │    ├ base       │
│                                                        │    ├ column     │
│                                                        │    └ ...        │
│                                                        ├─────────────────┤
│                                                        │  Properties     │
│                                                        │  ─────────────  │
│                                                        │  Transform      │
│                                                        │  Material       │
│                                                        │  Mesh Stats     │
├─────────────────────────────┬──────────────────────────┴─────────────────┤
│  VX Code Editor             │  Console / Diagnostics                     │
│  (syntax-highlighted)       │  [INFO] Evaluated in 42ms                  │
│                             │  [WARN] mesh 'base' has 8M tris            │
└─────────────────────────────┴────────────────────────────────────────────┘
```

panels are resizable. layout is persisted to `~/.config/vertexify/layout.toml`.

---

## 10. configuration

`~/.config/vertexify/vertexify.toml`:

```toml
[render]
render_scale     = 1.0          # 0.5–1.0; reduced automatically if framerate drops
idle_rendering   = true         # stop submitting frames when nothing changes
msaa_samples     = 1            # 1 | 4. iGPU: keep at 1. dGPU: 4 is fine.
ssao             = true
ssao_samples     = 8
max_lights       = 16

[vx]
eval_timeout_s      = 30
max_recursion_depth = 64
max_iterations      = 1000000
max_triangle_count  = 10000000

[ui]
font_size        = 14
theme            = "dark"       # "dark" | "light"
autosave_interval_s = 30
undo_history     = 100

[io]
default_export_format = "obj"
```

---

## 11. implementation phases

### phase 0 — workspace bootstrap (week 1)
- cargo workspace with all crates stubbed out.
- CI pipeline: `cargo clippy -- -D warnings`, `cargo fmt --check`, `cargo test`.
- `vx-app` opens a blank egui window backed by a wgpu surface. iGPU device enumeration logged.
- `vertexify.toml` parsing via `toml` + `serde`.

### phase 1 — VX language (weeks 2–4)
- `vx-lang`: lexer (`logos`), parser (recursive descent), AST, type checker, interpreter.
- built-in types and math functions. no geometry ops yet — return placeholder empty meshes.
- error reporting with source spans. test suite: 200+ unit tests covering edge cases.
- REPL mode: `vertexify --eval 'print(sin(1.5))'` for headless VX testing.

### phase 2 — geometry core (weeks 5–8)
- `vx-geometry`: half-edge mesh, indexed face set, conversion between them.
- all primitive generators: cube, sphere, cylinder, cone, plane, torus, capsule, icosphere.
- basic transforms applied to meshes.
- CSG: Cork/manifold integration. union, difference, intersection, xor.
- BVH build and query.
- OBJ and STL import/export.

### phase 3 — rendering (weeks 9–12)
- `vx-render`: wgpu device init, surface config, depth prepass, PBR main pass.
- iGPU-aware pipeline: batched indirect draw, texture atlas, uniform buffers.
- IBL: HDRI load, cubemap conversion, irradiance SH, specular mip-chain.
- SSAO half-res pass.
- grid shader, wireframe overlay.
- render scale control.

### phase 4 — UI and integration (weeks 13–16)
- `vx-ui`: egui panels. code editor with VX syntax highlighting (`egui_extras` + syntect).
- scene hierarchy panel. properties inspector. console panel.
- orbit camera, gizmos, object picking via BVH ray cast.
- incremental VX re-evaluation connected to the editor. dirty tracking working.

### phase 5 — advanced geometry (weeks 17–20)
- subdivide (Catmull-Clark), bevel, solidify, decimate, fill holes.
- scatter, array_linear, array_radial, array_curve.
- noise displacement, vertex color attributes.
- curve types: Bezier, NURBS, polyline. extrude, loft, revolve.
- UV projection modes.

### phase 6 — advanced I/O and polish (weeks 21–24)
- glTF 2.0 export (`gltf` crate).
- PLY import/export.
- `.vxs` binary scene format.
- crash recovery, autosave, undo/redo.
- adaptive render quality, idle rendering.
- diagnostics overlay, telemetry opt-in.
- `vertexify.toml` hot-reload.
- package as AppImage and `.deb` for distribution.

---

## 12. key dependencies

| crate / library | version policy | purpose |
|---|---|---|
| `wgpu` | latest stable | GPU abstraction, Vulkan backend |
| `egui` + `egui-wgpu` | match wgpu version | immediate-mode UI |
| `glam` | latest stable | SIMD math |
| `logos` | latest stable | lexer |
| `rayon` | latest stable | parallel geometry evaluation |
| `serde` + `bincode` | latest stable | scene serialization |
| `toml` | latest stable | config parsing |
| `tracing` + `tracing-subscriber` | latest stable | structured logging |
| `image` | latest stable | texture loading (PNG/JPEG/EXR) |
| `thiserror` | latest stable | error types in libraries |
| `anyhow` | latest stable | error handling in binary |
| `xxhash-rust` | latest stable | content hashing for incremental eval |
| `noise` | latest stable | Perlin/Simplex/Voronoi noise |
| `winit` | match wgpu | window creation, input events |
| manifold/Cork (FFI) | pinned | CSG boolean operations |

all dependency versions are pinned exactly in `Cargo.lock`. `cargo audit` runs in CI.

---

## 13. non-goals (explicit out-of-scope)

- animation, rigging, skinning, morph targets — not now, not planned.
- physics simulation.
- real-time ray tracing (RT cores not available on iGPU targets, and this is a modeling tool).
- node-based material editor (material system is defined in VX code).
- Python or Lua scripting — VX is the only scripting language.
- network collaboration or cloud sync.
- mobile or W*ndows or macOS ports (Linux only; macOS Metal via wgpu may work as an unofficial target).
- plugin SDK in v1 (internal plugin interface may exist, but no public ABI commitment until v2).