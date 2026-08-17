# Architecture and Runtime

## From Source to Execution

The `PengEnv::load_*_from_source_using` functions implement the real flow:

```text
source
  -> lex_source                      src/lexer/lexer.rs
  -> parse_program | parse_script    src/parser/
  -> optimize_ast(aggressive)        src/optimizer/
  -> generate_ast_using              src/generator/
  -> PengUnit                        src/core/unit.rs
  -> PengEnv::run / run_global_function
  -> scheduler -> step_thread -> execute_instruction
                                     src/core/env.rs, src/core/runtime.rs
```

`PengPosition { id, line, column }` accompanies tokens, AST nodes, and instructions.
`id` is provided by the host to identify the source; lines and columns start at 1.
A line containing a tab loses its column (`None`). Defined in
`src/core/position.rs` and `src/lexer/lexer.rs`.

## AST, Optimization, and Generation

`PengAST` has only `Program(Vec<PengBindedDeclaration>)` and
`Script(Vec<PengPositionedStatement>)`. Statements, expressions, and literals are
in `src/parser/parser.rs`.

The optimizer configurations are `none`, `safe`, and `aggressive`. Normal source
loading uses `aggressive`, enabling constant folding, boolean simplification,
dead-code/empty-block removal, and constant propagation. Defined in
`src/optimizer/config.rs`; the pass order is in `src/optimizer/optimizer.rs`.

The generator keeps scopes, local indices, external globals, constants, and jump
patches in `PengGeneratorContext` (`src/generator/generator_utils.rs`). Bytecode
functions and operations store their instructions, positions, constants, and
captured `using_values` (`src/core/function.rs`, `src/core/operation.rs`).

The constant pool is part of the generated function/operation. Constants are
deduplicated only when the stored runtime representation is the same enough for
bytecode reuse. In particular, numeric runtime equality does not mean constant
pool identity: `Int(1)` and `Float32(1.0)` must remain distinct constants because
later instructions may depend on the exact cell variant.

## Locals, Scopes, and Temporary Slots

Local indices are frame-relative. `PushLocal(n)`, `StoreLocal(n)`, and
`ReserveLocal(n)` all address `frame.base + n` in the current thread stack. The
runtime does not perform name lookup; the generator is responsible for assigning
correct local indices.

`PengGeneratorContext` tracks:

* lexical scopes: names visible at the current source position;
* `next_local`: next index available in the current lexical depth;
* `allocated_locals`: highest slot needed by the frame;
* `frame_prefix_locals`: slots that already exist when the frame starts, normally
  function parameters.

Entering a block or loop body records the current local cursor. Leaving that scope
removes names and restores the cursor, allowing legitimate slot reuse after the
scope has ended. Slots whose lifetimes overlap must not share an index.

Temporary locals are created by `generate_reserved_temporary_local`. They are not
inserted into lexical scopes because user code cannot name them, but they still
consume local indices and contribute to `allocated_locals`. The generator may emit
`ReserveLocal` at the expression site for a new temporary, and function/operation
finalization also prepends reserves for all frame locals not covered by
parameters. Runtime reservation is idempotent per frame, so this duplication is
intentional: all slots exist at function entry, and expression-local reserves
remain valid for bytecode paths that require the slot.

This contract keeps locals strict. The runtime should return `LocalNotFound` or
`InvalidInstruction(StoreLocal(_))` when bytecode addresses a slot that the
generator did not allocate; it should not synthesize missing locals or reinterpret
values to hide generator bugs.

## Unit, Environment, and Heap

`PengUnit` groups:

* `init: Option<PengHeapPtr>`: initializer function;
* `globals`: interned name to mutable/immutable heap pointer;
* `custom_access`: native callbacks used to resolve custom accesses.

Defined in `src/core/unit.rs`. `PengUnit::library()` has no `init`; units produced
by program/script do have one (`require_init` validates this).

`PengEnv` is the shared state: heap, name pool, active threads, pinned roots, and
GC interval. Pointers are logical indices `PengHeapPtr(usize)` and
`PengNamePoolPtr(usize)`, defined in `src/core/utils.rs`. Names are not strings
inside runtime structures; they are interned by the environment in
`src/core/env.rs`.

## Values, Cells, and Binded Cells

`PengCell` is the representation that fits in the stack/fields: `Nil`, numbers,
`Bool`, or `Reference(PengHeapPtr)`. `PengValue` is `Cell(PengCell)` or
`Box(PengBox)`. `PengBox` covers string, object, vector, type, module, thread,
function, and operation. Defined in `src/core/cell.rs`,
`src/core/value.rs`, and `src/core/boxed.rs`.

`PengBinded<T>` marks any content as `Mutable(T)` or `Immutable(T)`.
`PengBindedCell` is only the alias `PengBinded<PengCell>`. The VM preserves this
mark when moving cells, and checking it before store/mutation produces errors such
as `CannotAssignImmutable`. Defined in `src/core/binding.rs`, `src/core/cell.rs`,
and executed in `src/core/runtime.rs`.

Objects and modules are maps from interned names to `PengBindedCell`; vectors are
`Vec<PengBindedCell>`. Defined in `src/core/object.rs`, `src/core/module.rs`, and
`src/core/vector.rs`.

## Stack, Frames, and Calls

Each `PengThread` has a stack of binded cells, a stack of `PengFrame`, state,
result, and quantum. States include running, finished, paused, waiting, cancelled,
failed, and sleeping. Defined in `src/core/thread.rs`.

A frame contains:

* `program_counter`: next instruction;
* `base`: base of the locals in the stack;
* `procedure`: pointer to function/operation;
* `params_count`;
* `is_try`: indicates a recoverable call;
* `reserved_locals`: slots not yet initialized;
* `call_position`: position of the call for diagnostics.

Defined in `src/core/frame.rs`. `execute_function_call` validates the callable and
arity, creates/pushes a frame for bytecode, or builds a context and calls the
closure for a native function (`src/core/env.rs`). Variadic functions receive the
extra arguments gathered into a vector (`src/core/env.rs`).

The call stack convention is:

```text
... | callable | arg0 | arg1 | ... | argN
```

`FunctionCall(N)` treats the value `N` cells below the top as the callable. For
bytecode functions, arguments are copied into the callee frame starting at its
base. For native functions, a `PengNativeFunctionCallContext` is created with the
argument cells. On return, the callee frame is removed, the thread stack is
truncated back to the caller area, and the returned cell is pushed for the caller.

Method calls use the same convention. `object.method(a, b)` is generated by
evaluating the receiver once, resolving `method`, then passing the receiver as the
first argument:

```text
callable := object.method
FunctionCall(1 + explicit_arg_count)
```

The method body receives that receiver as its first parameter, normally named
`self`.

`Return` removes the frame, truncates its stack area, and returns a cell to the
caller. The scheduler switches active threads according to `quantum`; `yield_now`
pauses the current quantum. Executed in `src/core/runtime.rs`,
`src/core/env.rs`, and exposed in the contexts in `src/core/context.rs`.

## Existing Instructions

The normative enum is `PengInstruction` in `src/core/instruction.rs`:

* data/locals: `PushConst`, `MakeImmutable`, `PushLocal`, `ReserveLocal`,
  `StoreLocal`, `PushHeap`, `PushHeapRef`, `StoreHeap`, `PushString`;
* construction: `CreateEmptyObject`, `CreateEmptyModule`, `CreateVector`,
  `CreateSuperType`, `CreateTypedObject`, `Convert`;
* stack: `Duplicate`, `Pop`, `Swap`;
* calculation: `Add`, `Subtract`, `Multiply`, `Divide`, `Power`, `Remainder`,
  `Negate`, `Concat`, `And`, `Or`, `Not`, and six comparisons;
* calls: `OperationCall`, `TryOperationCall`, `FunctionCall`,
  `FunctionCallSpread`, `TryFunctionCall`, `TryFunctionCallSpread`;
* access: `GetIndex`, `SetIndex`, `GetAttribute`, `SetAttribute`, `GetMember`,
  `SetMember`;
* control: `Jump`, `JumpIfTrue`, `JumpIfFalse`, `Return`.

Each semantic behavior is implemented by the `execute_instruction` match in
`src/core/runtime.rs`. The parallel serializable list is `PengBinaryOpcode` in
`src/binary/format.rs`.

## Garbage Collection

The collector is tricolor mark-and-sweep (`White`, `Gray`, `Black`). Roots include
pinned pointers, active threads, and reachable references; boxed structures have
their references traversed. The scheduler calls the collector when the
environment interval expires — 120 seconds by default. Defined in
`src/core/garbage_collector.rs`, `src/core/colour.rs`, and `src/core/env.rs`.

Operational limitation: a Rust-side pointer kept only by the host must be treated
as a root by the host via `env.pinned_mut()` while it needs to survive a
collection. This conclusion follows from the root set in
`src/core/garbage_collector.rs` and the public `PengEnv` API.
