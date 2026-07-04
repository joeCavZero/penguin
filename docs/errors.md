# Errors and Diagnostics

The entire public chain uses `Result<_, PengError>`. The complete enum is in
`src/core/error.rs`.

## Structure

Errors can be wrapped with context:

* `Position(PengPosition)`;
* `PositionedMessage { message, position }`;
* `PositionedError { error, position }`;
* `Stack(Vec<PengError>)`;
* `Raised(Box<PengError>)` for an error raised during execution.

`PengError::push` turns an error into a `Stack` or adds another layer. The parser
and generator use this to preserve the specific error and add the stage that
failed. Defined in `src/core/error.rs` and widely used in `src/parser/` and
`src/generator/`.

`PengPosition` contains `id`, `line`, and `column: Option<usize>`. The host chooses
`id` when loading source; this makes it possible to map the error to the correct
file/document. Defined in `src/core/position.rs` and filled by
`src/lexer/lexer.rs`.

## Existing Categories

| Category        | Examples                                                      | Main origin            |
| --------------- | ------------------------------------------------------------- | ---------------------- |
| internal state  | `InternalError`, `NotImplemented`, `InvalidState`             | entire core            |
| names/scope     | `NameNotFound`, `LocalNotFound`, `AttributeNotFound`          | generator/runtime      |
| heap/references | `HeapValueNotFound`, `InvalidReference`, `DanglingReference`  | `src/core/env.rs`      |
| mutability      | `CannotAssignImmutable`, `CannotReadUninitialized`            | generator/runtime      |
| types           | `TypeMismatch`, `InvalidConversion`, `ExpectedFunction`       | value/runtime          |
| operations      | `DivisionByZero`, `ArithmeticOverflow`, invalid operation     | `src/core/runtime.rs`  |
| indices         | `IndexOutOfBounds`, `InvalidIndexTypeValue`                   | `src/core/runtime.rs`  |
| calls           | `WrongArgumentCount`, `CannotCallValue`, `MissingReturnValue` | env/runtime            |
| VM              | `StackUnderflow`, `FrameNotFound`, PC out of bounds           | env/runtime            |
| frontend        | `SyntaxError`, `UnexpectedToken`, `BreakOutsideLoop`          | lexer/parser/generator |
| user            | `Raised`                                                      | runtime                |

The exact list and payloads are defined in `src/core/error.rs`.

## Call Stack and Positions

Bytecode function instructions have a parallel vector of positions. Before a call,
the runtime obtains the position of the instruction and records `call_position` in
the new `PengFrame`. When a thread fails, these data are attached to the error as
the frames are handled. Defined in `src/core/function.rs`, `src/core/frame.rs`,
`src/core/env.rs`, and `src/core/runtime.rs`.

Positions can be removed from a binary with `strip_positions`; in that case, source
diagnostics naturally lose those points. Defined in `src/binary/options.rs` and
`src/binary/format.rs`.

## Host-Side Inspection

`PengError` does not implement `Display` or `std::error::Error` in the current code.
Pattern-match on the variants and traverse wrappers/stacks:

```rust
use penguin::prelude::*;

fn positions(error: &PengError, out: &mut Vec<PengPosition>) {
    match error {
        PengError::Position(position) => out.push(position.clone()),
        PengError::PositionedMessage { position, .. } => out.push(position.clone()),
        PengError::PositionedError { error, position } => {
            out.push(position.clone());
            positions(error, out);
        }
        PengError::Stack(errors) => {
            for error in errors {
                positions(error, out);
            }
        }
        PengError::Raised(error) => positions(error, out),
        _ => {}
    }
}
```

## Diagnostic Procedure

1. Preserve a distinct `position_id` per source when calling `load_*_from_source`.
2. Traverse `Stack`, `PositionedError`, and `Raised`; do not consider only the
   outermost variant.
3. Use `id`, `line`, and `column` to locate the token; accept a missing column on
   lines containing tabs.
4. In name errors, convert `PengNamePoolPtr` with
   `PengEnv::get_pooled_name` (`src/core/env.rs`).
5. For external bytecode, call `peng_binary_decode`/`peng_binary_validate_file`
   before executing (`src/binary/codec.rs`).
6. For VM debugging, correlate `PengFrame::program_counter` with
   `PengBytecodeFunction.positions` and `bytecode` (`src/core/frame.rs`,
   `src/core/function.rs`).

The project does not yet provide a formatter, source excerpt, diagnostic CLI, or
integration with `std::error::Error`; these resources should not be assumed.