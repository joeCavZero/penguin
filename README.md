<div align="center">
  <img src="docs/assets/penguin.png" width="200" />
</div>

<h1 align="center">PENGUIN</h1>

<p align="center">
A modern, embeddable, dynamically typed programming language written in Rust.
</p>

<p align="center">
Fast - Simple - Extensible - Portable
</p>

---

## About

Penguin is a lightweight scripting language designed to be embedded into applications while also supporting standalone execution.

It focuses on simplicity, good runtime performance, and an extensible architecture, making it suitable for:

- scripting
- automation
- game development
- command line tools
- applications
- plugins
- educational projects

The language is implemented entirely in Rust and exposes a clean API for integrating Penguin into native applications.

---

## Features

- Dynamic typing
- Bytecode virtual machine
- Garbage collected runtime
- Object-oriented programming
- Native Rust API
- Binary compilation support
- Cross-platform
- Easy embedding

## Example

An example of penguin code using the [pabble](https://github.com/joeCavZero/pabble) ecosystem
```go
import("io") as io

func factorial(n) {
    if n <= 1 {
        return 1
    }

    return n * factorial(n - 1)
}

func main() {
    io:println(factorial(10))
}
```

## Embedding

Penguin was designed to be embedded inside Rust applications.

```rust
use penguin::prelude::*;
```

Native functions, objects, modules and custom libraries can be registered directly through the runtime API.

## Goals

The main goals of Penguin are:

- Simple syntax
- Small runtime
- Fast execution
- Easy integration
- Predictable behavior
- Extensible architecture
- Clean implementation

## Documentation
- [language](/docs/language.md)
- [archtecture](/docs/archtecture.md)
- [embedding](/docs/embedding.md)
- [binary](/docs/binary.md)
- [errors](/docs/errors.md)