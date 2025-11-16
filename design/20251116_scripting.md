# The architecture of the scripting language for drones

1. **Overall architecture (NL → JSON AST → execution)**
2. **Core language design**
3. **How to model `TileBox` and coordinates**
4. **JSON AST shape + examples**
5. **Interop with Rust crates + randomness**

---

## 1. Architecture: three layers, one source of truth

Think in terms of three representations:

1. **Surface (what player types)**

   * Arbitrary English like:

     > “Mine all tiles from (10, 5) to (20, 7) and build a wall on the border.”
   * **Never trusted.** It’s just an input to the LLM.

2. **Canonical JSON AST (what the LLM produces)**

   * This is your **source of truth**.
   * Strict schema, versioned, fully typed.
   * All validation, safety checks and execution happen here.

3. **Secondary views**

   * **Textual DSL**: something like a Rust-ish mini language that you generate **from the AST**:

     ```rust
     let area: TileBox = box(10,5 .. 20,7);
     mine(area);
     build_wall(area.border());
     ```
   * **Execution IR**: bytecode, or a lowered IR that your Rust engine interprets or compiles.

Key point: **the LLM never generates source code**, only JSON AST.
Text DSL and execution both come from that AST, so users can’t confuse the compiler with weird syntax.

---

## 2. Core language design

### 2.1. Goals

* **Static, Rust-like types** from day one, even if tiny at first:

  * `Int`, `Float`, `Bool`, `TileCoord`, `TileBox`, maybe `Drone`, `Task`, etc.
* **Expression-based** (like Rust): expressions return values.
* Start with:

  * **Bindings**: `let` (immutable), maybe `mut` later.
  * **Function calls**: calling built-in/native operations.
  * **Simple control-flow**: `if`, `for tile in box`, etc., later.

### 2.2. Minimal initial feature set

For v0 focused on TileBox tasks, you can get away with:

* **Bindings**:

  ```json
  { "node": "Let", "name": "area", "ty": "TileBox", "value": { ... } }
  ```

* **TileBox construction**:

  * From two coordinates
  * From center + radius

* **Loops over tiles**:

  ```json
  {
    "node": "ForIn",
    "var": { "name": "tile", "ty": "TileCoord" },
    "iter": { "node": "TileBoxIter", "box": { ... } },
    "body": [ ... statements ... ]
  }
  ```

* **Function calls**:

  * `mine(tile)`, `build_wall(tile)`, `scan_resources(box)`, etc.
  * Each maps to a Rust function or method.

Once this is solid, you add:

* Primitive math & boolean expressions.
* `if`/`else`.
* User-level functions (or at least macros later).
* Struct types that mirror Rust structs.

---

## 3. Types: `TileCoord` and `TileBox`

You said the first real type is **TileBox** (range of tiles). Design types like this:

### 3.1. Coordinate + box types

Assuming 2D for now:

```rust
struct TileCoord {
    x: i32,
    y: i32,
}

struct TileBox {
    min: TileCoord, // inclusive
    max: TileCoord, // inclusive
}
```

If you do multi-level / 3D later:

```rust
struct TileCoord3 {
    x: i32,
    y: i32,
    z: i32, // level
}

struct TileBox3 {
    min: TileCoord3,
    max: TileCoord3,
}
```

**Invariants**:

* `min.x <= max.x`, `min.y <= max.y`, (`min.z <= max.z`).
* Axis-aligned box on the tile grid.
* Operations:

  * `TileBox::border() -> TileBox` or iterator `border_tiles()`.
  * `intersect(other)`, `expand(dx, dy)`, `contains(tile)`.

In the DSL type system:

* `TileCoord` and `TileBox` are **first-class** value types.
* They mirror your Rust structs **exactly** so interop is trivial.

---

## 4. JSON AST: concrete shape

Give the LLM a **fixed JSON schema**. You can evolve it, but v0 should be crystal clear.

### 4.1. Top-level program

```json
{
  "version": 1,
  "node": "Program",
  "statements": [ ... ]
}
```

### 4.2. Basic statement nodes

**Let-binding a `TileBox`:**

```json
{
  "node": "Let",
  "name": "area",
  "ty": "TileBox",
  "value": {
    "node": "TileBoxFromCoords",
    "min": { "node": "TileCoord", "x": 10, "y": 5 },
    "max": { "node": "TileCoord", "x": 20, "y": 7 }
  }
}
```

**Call a primitive operation (Rust function):**

```json
{
  "node": "ExprStmt",
  "expr": {
    "node": "Call",
    "func": "mine_box",
    "args": [
      { "node": "VarRef", "name": "area" }
    ]
  }
}
```

**Loop over the box:**

```json
{
  "node": "ForIn",
  "var": { "name": "tile", "ty": "TileCoord" },
  "iter": {
    "node": "IterTiles",
    "box": { "node": "VarRef", "name": "area" }
  },
  "body": [
    {
      "node": "ExprStmt",
      "expr": {
        "node": "Call",
        "func": "mine_tile",
        "args": [ { "node": "VarRef", "name": "tile" } ]
      }
    }
  ]
}
```

### 4.3. Example: full small program

Natural language:

> “Mine all tiles from (10, 5) to (20, 7), then build a wall around the border.”

LLM → JSON:

```json
{
  "version": 1,
  "node": "Program",
  "statements": [
    {
      "node": "Let",
      "name": "area",
      "ty": "TileBox",
      "value": {
        "node": "TileBoxFromCoords",
        "min": { "node": "TileCoord", "x": 10, "y": 5 },
        "max": { "node": "TileCoord", "x": 20, "y": 7 }
      }
    },
    {
      "node": "ExprStmt",
      "expr": {
        "node": "Call",
        "func": "mine_box",
        "args": [
          { "node": "VarRef", "name": "area" }
        ]
      }
    },
    {
      "node": "ExprStmt",
      "expr": {
        "node": "Call",
        "func": "build_wall_on_border",
        "args": [
          { "node": "VarRef", "name": "area" }
        ]
      }
    }
  ]
}
```

Then you can **pretty-print** this into a readable DSL:

```rust
let area: TileBox = box((10,5)..(20,7));
mine_box(area);
build_wall_on_border(area);
```

That textual DSL is **output only**, never the source of truth.

---

## 5. Rust interop + randomness

### 5.1. How to call Rust crates

Treat Rust functions as **host functions** exposed to the DSL.

Define a registry in Rust:

```rust
struct HostFunc {
    name: &'static str,
    param_types: &'static [Type],
    return_type: Option<Type>,
    func: fn(&mut VmContext, &[Value]) -> Result<Value, Error>,
}
```

At startup, you register host functions, including those from crates:

```rust
register_host_func(HostFunc {
    name: "mine_box",
    param_types: &[Type::TileBox],
    return_type: None,
    func: |ctx, args| { /* call into your game logic */ },
});

register_host_func(HostFunc {
    name: "rand_int",
    param_types: &[Type::Int, Type::Int],
    return_type: Some(Type::Int),
    func: |ctx, args| { /* use rand crate here */ },
});
```

In the AST:

```json
{
  "node": "Call",
  "func": "rand_int",
  "args": [
    { "node": "IntLiteral", "value": 0 },
    { "node": "IntLiteral", "value": 100 }
  ]
}
```

Your VM/engine:

1. **Type-checks** that `rand_int` exists and the argument types match.
2. Executes by calling the registered Rust function with the given args.

Over time, you can add:

* Structs / enums that mirror Rust ones exactly.
* Generic-ish types (e.g. `List<TileCoord>`) if needed.
* Possibly a stable ABI or WASM boundary later.

### 5.2. Random generators

Randomness should be **host-provided**, not DSL-internal:

* DSL has a simple `rand_int(min, max)` or `rand_tile_in(box)`.
* Implementation in Rust uses `rand` or whatever RNG crate.
* For determinism: each drone has its own RNG seeded from the world seed.

This way:

* The DSL itself stays **pure-ish** (no global RNG state).
* All side effects (world modifications, randomness) are in host Rust.

---

## 6. Validation & evolution

A few practical rules to keep it sane:

* **Version your AST**:

  ```json
  { "version": 1, "node": "Program", ... }
  ```
* Separate passes:

  1. **Schema validation** (is the JSON structurally correct?).
  2. **Name resolution** (variables exist, functions exist).
  3. **Type checking** (types of ops and calls are consistent).
  4. **Lowering** to a simpler IR (optional).
* If LLM output fails any check, you:

  * Return error to the player (“I couldn’t understand this command”), or
  * Ask the LLM to **fix** the AST given the error message (self-healing).

