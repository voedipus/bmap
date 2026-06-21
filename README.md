# bmap

A fast, native GUI for inspecting linker MAP files. Built with [iced](https://iced.rs/).

Load a `.map` file produced by GNU ld, Clang LLD, or Metrowerks, and explore your binary's memory layout through sortable tables and drill-down views.

## Features

- **Source Files** — grouped by source file with archive module, searchable
- **Modules** — grouped by `.a` archive file, searchable
- **Sections** — consolidated section categories (.text, .data, .rodata, .bss) expandable to sub-types
- **Drill-down** — click any row to see individual symbols with size, address, and percentage
- **Debug symbols filter** — toggle to show/hide debug sections (.debug_*, .comment, .note, .ARM.*)
- **Summary** — total size and breakdown by Code, Data, BSS, and Other
- **System library filter** — automatically excludes symbols from libc, libm, libgcc, etc.
- **Light/dark theme** — follows system preference automatically
- **Internationalization** — English and Russian translations

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

MIT
