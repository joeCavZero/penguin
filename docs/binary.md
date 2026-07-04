# Binary Format

Penguin has its own binary format, implemented in `src/binary/`. It serializes the
generated unit; it is not an operating system executable.

## Header

`PengBinaryHeader` is defined in `src/binary/header.rs`:

| Field        | Encoding            | Current value                         |
| ------------ | ------------------- | ------------------------------------- |
| magic        | 4 bytes             | ASCII `peng`                          |
| version      | little-endian `u16` | `1`                                   |
| architecture | `u8`                | `usize::BITS` of the compiler machine |
| endian tag   | `u8`                | `0` little, `1` big                   |
| flags        | little-endian `u32` | `0`                                   |

Although the enum contains `Big`, validation rejects any endianness other than
little. `arch_bits` and `flags` are written, but the current validation does not
enforce specific values. Defined in `src/binary/header.rs` and validated in
`src/binary/codec.rs`.

After the header, the codec writes: entry kind (`Program` or `Script`), heap table,
globals, and optional `init` reference. Size/index integers are encoded as `u32`;
strings are encoded as length + UTF-8 bytes. The exact order is in
`peng_binary_encode`/`peng_binary_decode` in `src/binary/codec.rs`.

## Serializable Model

`PengBinaryFile` contains `header`, `entry_kind`, `heap`, `globals`, and `init`.
References can be:

* `Heap(PengBinaryHeapId)`, a contiguous ID inside the file;
* `External(String)`, resolved against the unit provided when loading.

Values, cells, types, and opcodes have their own binary enums in
`src/binary/format.rs`. Native closures are not serialized; host unit dependencies
become external references by name during collection. The unit ↔ file conversion is
performed by `peng_binary_from_unit*` and `peng_binary_to_unit`, in the same file.

## High-Level API

```rust
use penguin::prelude::*;

fn round_trip() -> Result<(), PengError> {
    let mut compiler = PengEnv::new();
    let bytes = compiler.compile_script_to_binary("return 42", 0)?;

    let mut runtime = PengEnv::new();
    let unit = runtime.load_script_from_binary(&bytes)?;
    let result = runtime.run(unit.require_init()?, &unit)?;
    assert!(matches!(result, PengBinded::Mutable(PengCell::Int(42))));
    Ok(())
}
```

Defined in `src/core/env.rs`. For native/external dependencies, use the `*_using`
variants both when compiling and when loading, passing a compatible `PengUnit`.

## Options and Validation

`PengBinaryBuildOptions` has `strip_positions` (default `false`) and
`keep_debug_names` (default `false`), defined in `src/binary/options.rs`. The first
removes positions from the bytecode. The second exists in the API, but is not
consulted by the collector in the current code; it should be considered
reserved/experimental.

`peng_binary_validate_file` checks magic, version, endianness, contiguous IDs,
references, globals, `init`, constant indices, jump targets, and consistency
between instructions and positions. The decoder also rejects trailing bytes.
Defined in `src/binary/codec.rs`.

## Portability Limitations

* Runtime cells use `isize`/`usize`, but the file uses `i64`/`u64`; loading casts
  them to the native size (`src/binary/format.rs`). The header records
  `arch_bits`, but validation does not block a different architecture.
* Only little-endian is accepted.
* The supported version is exactly 1; there is no migration between versions.
* Native functions/operations must be provided again as externals.
* There is no file extension or CLI command defined in this repository.
