# BMAP

A fast, native GUI for inspecting linker MAP files. Built with [iced](https://iced.rs/).

Load a `.map` file produced by GNU ld, Clang LLD, or Metrowerks, and explore your binary's memory layout through sortable tables and drill-down views.

## Features

- **All Symbols** — flat table of every symbol with name, address, size, and % of total
- **By Module** — grouped by object file, showing source file and module name per group
- **By Section** — grouped by section type (.text, .data, .bss, .rodata, etc.)
- **Drill-down** — click any module or section to see only its symbols, with a search bar
- **Debug filter** — toggle to show/hide debug sections (.debug_*, .comment, .note, .ARM.*)
- **Summary** — total size and breakdown by Code, Data, BSS, and Other
- **System library filter** — automatically excludes symbols from libc, libm, libgcc, etc.

## Usage

```
cargo run --release
```

Then click **Open** and select a `.map` file.

## Build

```
cargo build --release
```

The binary will be at `target/release/bmap`.

## License

MPL-2.0
