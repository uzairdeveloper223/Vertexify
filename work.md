# work.md — vertexify development log

## done

- expanded `plan.md` from rough sketch to full architectural specification.
- iGPU strategy rewritten: from "CPU primary with SIMD fallback" to "iGPU-first Vulkan/wgpu pipeline
  with explicit design constraints for shared-memory bandwidth, thermal throttling, and tile-based
  rendering architectures."
- removed software rasterizer plan — it was a crutch. wgpu with adaptive render scale covers the
  low-end case without a second rendering path to maintain.
- expanded VX DSL to a full language spec including text2d/text3d, primitives, transforms, CSG,
  mesh ops, procedural scatter/array, curves, surfaces, UVs, attributes, materials, selection types,
  built-in math, control flow, closures, recursion, scene output.
- defined full VX type system: value types, geometry types, compound types.
- specced the interpreter pipeline: logos lexer → recursive descent parser → bidirectional type
  checker → dependency resolver → tree-walk interpreter with rayon-parallel branch evaluation.
- specced incremental re-evaluation via content hashing (xxhash3) and a dependency DAG.
- defined the full reliability architecture: failure taxonomy, VX runtime safety guards (depth limit,
  eval timeout, geometry size cap, loop iteration cap, panic isolation).
- defined project persistence: autosave, crash recovery, undo/redo as VX checkpoint ring buffer,
  versioned `.vxs` binary format with atomic write.
- defined material system: PBR Cook-Torrance, IBL split-sum, texture atlas packing, texture cache.
- defined geometry subsystem: half-edge mesh for topology ops, indexed face set for GPU upload,
  SAH-binned BVH, Cork/manifold for CSG.
- defined all I/O formats: OBJ, STL, PLY, glTF 2.0, VXS, VX source.
- defined configuration schema: `vertexify.toml` with render, vx, ui, io sections.
- phased implementation plan: 6 phases over 24 weeks.
- pinned all key dependencies with justification.
- explicit non-goals section (animation, physics, RT, macOS/Windows).
- **Phase 0 complete:** 7-crate Cargo workspace scaffolded and compiling clean.
  - `vx-lang`, `vx-geometry`, `vx-render`, `vx-scene`, `vx-ui`, `vx-io`, `vx-app` all present.
  - `vx-app`: egui 0.34 + wgpu 29 + winit 0.30 window renders; iGPU adapter enumerated and logged.
  - `vx-app/config.rs`: `vertexify.toml` parsed via serde at startup; XDG config path resolution.
  - CI workflow: `cargo fmt --check`, `cargo clippy --workspace -- -D warnings`, `cargo test --workspace`.
  - `cargo fmt --all` applied; `cargo check --workspace` → 0 errors, 0 clippy failures.
- **Phase 1.5 complete:** built-in math/string functions and global constants fully implemented, tested, and documented.
  - `interpreter.rs`: `Value::NativeFn { name, arity }` variant dispatched through `call_fn → dispatch_native`.
  - **Math (1-arg):** `abs`, `sqrt`, `floor`, `ceil`, `round`, `sin`, `cos`, `tan`, `log`, `exp`.
  - **Math (2-arg):** `pow`, `atan2`, `min`, `max`.
  - **Math (3-arg):** `clamp`, `lerp`.
  - **String:** `len`, `upper`, `lower`, `trim`, `to_str`, `to_int`, `to_float`.
  - **Constants:** `PI`, `TAU`, `E`, `INF` injected via `register_builtins(env)`.
  - `types.rs`: `register_builtin_types(env)` registers all built-in function signatures and constant
    types so the type checker accepts built-in calls without error.
  - 21 new tests in `builtin_tests` module; total 67/67 passing, 0 clippy warnings.
  - Docs updated: `builtins.md` rewritten with all items marked implemented; `examples.md` expanded
    to 20 examples including built-in math/string examples and combined text scenes;
    `README.md` hello world updated to use `PI` and `to_str`.
  - `lexer.rs`: 50+ tokens via logos 0.15 — keywords, types, scene words, text2d/text3d, literals,
    operators, delimiters; line and block comment skipping; 20 unit tests including span positions.
  - `ast.rs`: complete AST — Spanned<T>, Ty, Lit, BinOp, UnOp, AssignOp, Expr (incl. Text2d/Text3d),
    Stmt, Param, TextProps (15 documented properties), Decl, Module.
  - `parser.rs`: recursive descent — full operator precedence chain (or/and/cmp/add/mul/unary/
    postfix/primary), block expressions, if-else chains, for loops, let/return/assign statements,
    function calls, indexing, field access, text2d/text3d property blocks; 6 parser tests.
  - `error.rs`: LangError covering UnexpectedChar, UnterminatedString, BadInt, BadFloat,
    ParseError, UnexpectedEof — all with thiserror Display.
  - `interpreter.rs`: tree-walk interpreter — Value enum (Int, Float, Bool, Str, Vec2/3/4, Mat4,
    Color, Text2d, Text3d, Null, Fn), Env with frame stack and MAX_CALL_DEPTH=64 guard, Snapshot
    for closure capture, full eval_expr/eval_stmt/eval_block/eval_module/call_fn implementation,
    TextData with 14 properties + TextAlign, eval_text_props with per-key validation and defaults;
    12 interpreter tests.
  - `types.rs`: TypeEnv with lexical frames, infer() for all Expr variants including Text2d/Text3d
    → Ty::Geo, check_binary/check_unary type rules, infer_field for vec/color swizzles,
    check_module for full module type checking; 7 type tests.
  - `docs/vx-lang/`: 5 well-documented Markdown files — README, syntax reference, type system
    reference, built-in functions, and 11 runnable example programs (incl. text2d/text3d examples).

## next

- **Phase 2 — `vx-geometry`:** half-edge mesh, primitive generators (cube, sphere, cylinder, plane).
- **Phase 3 — `vx-render`:** PBR pipeline: depth prepass, main pass, SSAO, grid, wireframe overlay.
- **Phase 3 — WGSL shaders:** PBR Cook-Torrance, IBL split-sum, HDRI→cubemap compute, irradiance SH,
  specular prefilter compute, SSAO, bilateral blur.
- **Phase 4 — `vx-scene`:** ECS, scene graph, object selection, transform gizmos.
- **Phase 5 — `vx-ui`:** editor layout, viewport, code editor panel, property inspector, outliner.
- **Phase 5 — VX syntax highlighting** for `syntect`.
- **Phase 6 — `vx-io`:** OBJ/STL/PLY/glTF import-export, autosave, crash recovery, `.vxs` format.
- Cork/manifold FFI wrapper for CSG (`bindgen` or pure-Rust port).
- AppImage and `.deb` packaging scripts.
- Benchmarking on Intel UHD 620 to validate render budget assumptions.

## learned

- iGPU rendering bottleneck is almost never shader ALU — it's memory bandwidth, draw call count, and
  render pass breaks. the design must treat these as the primary budget, not shader complexity.
- Cork for CSG requires manifold input. need a mesh validation step before any CSG op in the VX runtime,
  not after. catching it early gives a clean error instead of corrupt output.
- incremental VX re-evaluation requires the dependency graph to be built during parsing/type-checking,
  not at runtime. the AST must carry binding dependency edges as a first-class structure.
- undo as "checkpoint the VX source text" is simpler and more correct than undo as "reverse scene
  mutations". scene mutations are not trivially reversible (CSG, decimate are lossy). source text
  checkpoints + re-evaluation is the right model.
- **wgpu 29 API changes** (verified from crate source, not docs):
  - `InstanceDescriptor` has no `Default` impl → use `InstanceDescriptor::new_without_display_handle()`.
  - `DeviceDescriptor` gained `experimental_features` and `trace` fields → `..Default::default()` fills them.
  - `request_device` now takes only 1 argument (no trace path arg).
  - `surface.get_current_texture()` returns `CurrentSurfaceTexture` enum (not `Result`) with variants
    `Success`, `Suboptimal`, `Outdated`, `Lost`, `Timeout`, `Occluded`, `Validation`.
  - `RenderPassColorAttachment` requires new `depth_slice: None` field.
  - `RenderPassDescriptor` requires new `multiview_mask: None` field.
  - `egui_wgpu::Renderer::render` requires `RenderPass<'static>` → call `.forget_lifetime()` after
    `encoder.begin_render_pass(...)`.
- **egui 0.34 API changes**:
  - `Context::run` deprecated → use `Context::run_ui(raw_input, |ui: &mut Ui| { ... })`.
  - `CentralPanel::show` deprecated → use `CentralPanel::show_inside(ui, |ui| { ... })`.
  - `egui_winit::State::new` takes 6 args: `(ctx, viewport_id, display_target, native_ppp, theme, max_tex)`.
  - `egui_wgpu::Renderer::new` takes 3 args: `(device, format, RendererOptions)`.
  - `pixels_per_point` is a free function in `egui_winit`: `egui_winit::pixels_per_point(&ctx, &window)`.
- **logos 0.15 quirks**:
  - No `Default` impl on error types — must add `#[derive(Default)]` manually.
  - `Range<usize>` (returned as Span) does not implement `Copy` → clone before dual-closure capture.
  - Float regex must be listed before integer regex to prevent `1.0` being lexed as `Int("1")` then `Dot`.
  - Block comment regex `r"/\*([^*]|\*[^/])*\*/"` handles non-nested comments correctly.
- **text support design:**
  - text2d and text3d are parsed as property-block expressions (`text2d { key: expr, ... }`), not
    function calls, because they have many optional named parameters with sensible defaults.
  - `color` is a type keyword in the lexer but also valid as a property key — the parser accepts
    `Token::TyColor` as a key name in `parse_text_props`.
  - `TextData` uses `Default` derive so every unspecified property gets a sensible default without
    explicit `None` handling in eval_text_props.
  - `content` is the only required property; the interpreter returns a `LangError` if it is missing
    or empty.

## remaining

- Phase 1.5: built-in math/string functions; global constants (PI, TAU, E, INF).
- Phase 2: `vx-geometry` half-edge mesh, primitive generators (cube, sphere, cylinder, plane).
- Phase 3: `vx-render` PBR pipeline: depth prepass, main pass, SSAO, grid, wireframe overlay.
- Phase 3: WGSL shaders: PBR Cook-Torrance, IBL split-sum, HDRI→cubemap compute, irradiance SH,
  specular prefilter compute, SSAO, bilateral blur.
- Phase 4: `vx-scene` ECS, scene graph, object selection, transform gizmos.
- Phase 5: `vx-ui` editor layout, viewport, code editor panel, property inspector, outliner.
- Phase 5: VX syntax highlighting for `syntect`.
- Phase 6: `vx-io` OBJ/STL/PLY/glTF import-export, autosave, crash recovery, `.vxs` binary format.
- Cork/manifold FFI wrapper for CSG (`bindgen` or pure-Rust port).
- AppImage and `.deb` packaging scripts.
- Benchmarking on Intel UHD 620 to validate render budget assumptions.