# Embedding in Rust

`penguin::prelude::*` reexports the core and `PengBinaryBuildOptions`; it does not
automatically register functions or modules. Defined in `src/prelude.rs`.

## Running a Script

```rust
use penguin::prelude::*;

fn main() -> Result<(), PengError> {
    let mut env = PengEnv::new();
    let unit = env.load_script_from_source(7, "return 6 * 7")?;
    let result = env.run(unit.require_init()?, &unit)?;

    assert!(matches!(result, PengBinded::Mutable(PengCell::Int(42))));
    Ok(())
}
```

`position_id` (`7` above) is preserved in errors/positions. Defined in
`src/core/env.rs` and `src/core/position.rs`.

## Loading a Program and Calling a Global

A program initializes declarations; run its `init` before calling globals.

```rust
use penguin::prelude::*;

fn main() -> Result<(), PengError> {
    let mut env = PengEnv::new();
    let source = "func double(n: int) -> int { return n * 2 }";
    let unit = env.load_program_from_source(source, 0)?;

    env.run(unit.require_init()?, &unit)?;
    let result = env.run_global_function(
        "double",
        &unit,
        vec![PengBinded::Mutable(PengCell::Int(21))],
    )?;

    assert!(matches!(result, PengBinded::Mutable(PengCell::Int(42))));
    Ok(())
}
```

Defined in `PengEnv::{load_program_from_source, run, run_global_function}` in
`src/core/env.rs`.

## Registering a Native Function

Create a library unit, register the callback, and load source with the `*_using`
variant. The callback receives arguments from the stack through
`PengNativeFunctionCallContext`.

```rust
use penguin::prelude::*;

fn main() -> Result<(), PengError> {
    let mut env = PengEnv::new();
    let mut host = PengUnit::library();

    host.register_immutable_native_function(&mut env, "answer", |_ctx| {
        Ok(PengBinded::Mutable(PengCell::Int(42)))
    })?;

    let unit = env.load_script_from_source_using("return answer()", &host, 0)?;
    let result = env.run(unit.require_init()?, &unit)?;
    assert!(matches!(result, PengBinded::Mutable(PengCell::Int(42))));
    Ok(())
}
```

Native functions have the signature
`Fn(&mut PengNativeFunctionCallContext) -> Result<PengBindedCell, PengError> +
'static`. Registration is in `src/core/unit.rs`; the context is in
`src/core/context.rs`; the wrapper is in `src/core/function.rs`.

The context provides `get_arg_cell[_mut]`, `get_arg_value[_mut]`, `env[_mut]`,
`unit`, `thread`, allocation of values/boxes, and execution of Penguin functions
from source. There is no explicit count parameter in the context; arity belongs to
the current frame (`src/core/context.rs`, `src/core/frame.rs`).

## Values, Objects, and Vectors from the Host

Scalars can be registered directly; boxed values are allocated by the environment
when needed.

```rust
use penguin::prelude::*;

fn build_host_values(env: &mut PengEnv, unit: &mut PengUnit) -> Result<(), PengError> {
    unit.register_immutable_global(
        env,
        "host_answer",
        PengValue::Cell(PengCell::Int(42)),
    )?;

    let key = env.ensure_pooled_name_ptr("name".to_string());
    let text = env.create_heap_value(PengValue::Box(PengBox::String("penguin".into())));
    let mut fields = std::collections::HashMap::new();
    fields.insert(key, PengBinded::Immutable(PengCell::Reference(text)));

    unit.register_mutable_global(
        env,
        "metadata",
        PengValue::Box(PengBox::Object(PengObject::new(fields))),
    )?;
    Ok(())
}
```

APIs are defined in `src/core/unit.rs`, `src/core/env.rs`, `src/core/object.rs`,
`src/core/value.rs`, and `src/core/boxed.rs`. For vectors, use
`PengVector::new(Vec<PengBindedCell>)` or `PengEnv::create_vector_from_cells`,
defined in `src/core/vector.rs` and `src/core/env.rs`.

## Operations, Modules, and Custom Access

`PengUnit` also provides:

* `register_{mutable,immutable}_native_operation`: closure with
  `PengNativeOperationCallContext`, which exposes left/right;
* `register_{mutable,immutable}_module`: converts globals from another unit into
  members;
* `register_custom_access`: callback consulted for custom accesses;
* `register_{mutable,immutable}_global`: any `PengValue`.

All are defined in `src/core/unit.rs`; the contexts are in `src/core/context.rs`.
`PengUnit::use_unit` copies globals and custom access into another unit. When
compiling source, prefer passing the library to `load_*_from_source_using` so the
generator can resolve these names.

Custom access is how a host can attach behavior to values without storing a real
field on every value. The runtime consults it when `GetAttribute` cannot find the
requested attribute directly. The custom access callback returns a callable cell;
the source can then call it like a method:

```penguin
for (var i = 0; i < values.len(); i = i + 1) {
    // len can be supplied by the embedding host
}
```

At runtime this follows the normal method-call convention: `values.len()` resolves
the `len` callable and passes `values` as the first argument.

## Main Public APIs

| Need              | API                                                    | Origin                       |
| ----------------- | ------------------------------------------------------ | ---------------------------- |
| lex               | `lex_source`                                           | `src/lexer/lexer.rs`         |
| parse             | `parse_program`, `parse_script`                        | `src/parser/`                |
| optimize          | `optimize_ast`                                         | `src/optimizer/optimizer.rs` |
| generate unit     | `generate_ast[_using]`                                 | `src/generator/generate.rs`  |
| load source       | `PengEnv::load_{program,script}_from_source[_using]`   | `src/core/env.rs`            |
| execute           | `PengEnv::run`, `run_global_function`, `run_isolated*` | `src/core/env.rs`            |
| compile binary    | `compile_{program,script}_to_binary[_using]`           | `src/core/env.rs`            |
| load binary       | `load_{program,script}_from_binary[_using]`            | `src/core/env.rs`            |
| register host API | `PengUnit` `register_*` methods                        | `src/core/unit.rs`           |

The `run_isolated*` variants create a temporary library unit and execute a
function; `load_function_from_source*` wraps the source as a `func` literal and
returns its pointer. Defined at the end of `src/core/env.rs`.

## Integration Limitations

* The crate does not implement `Display`/`std::error::Error` for `PengError`;
  hosts must inspect `Debug` or format variants on their own (`src/core/error.rs`).
* Native closures use `Rc`, so runtime structures do not promise `Send`/`Sync`
  (`src/core/function.rs`, `src/core/operation.rs`).
* The GC requires care with pointers kept by the host; see
