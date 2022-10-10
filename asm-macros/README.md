### ASM support for PULP RISC-V ISA Extension

`asm-macros` provides partial support for using PULP specific instructions in asm blocks without support from the Rust compiler.

It provides special macros that can be used almost like standard mnemonics:
```
// with macros
asm_macros::ror!(x1, x1, x3)

// mnemonic (requires compiler support)
ror x1, x1, x3
```
