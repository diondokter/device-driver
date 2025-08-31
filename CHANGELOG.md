## Changelog

### Unreleased

- Internal numbers are now `i128` instead of `i64`. This also added room to add the `U64` integer as available address type option
- Fixed regression introduced in 1.0.5 where an enum that has both a default and a catch-all would use the default value when converted from a number instead the catch-all like is documented
- Added KDL input support, but not stable yet. Only use experimentally for now.
- FieldSets don't know about reset values anymore. This can now be accessed through the RegisterOperation you can obtain from the device struct.
- FieldSets & Enums can no longer have the same name as another object (since they're full objects of their own now)
- FieldSets are no longer defined in a submodule `field_sets`. They're now put in the same root as the reset of the generated code.
- Removed the 'read_all_registers' functions and the accompanying enums
- Updated `convert_case` to 0.8
- Updated `defmt` to 1.0.1. The crate feature is now called `defmt` instead of `defmt-03`

### 1.0.7 (30-07-25)

- Fix a compilation time explosion issue. The generated `read_all_registers` functions have been simplified.
  On devices with many registers, the async variant would explode in compilation time because of the many awaits.
  The callback name parameter now doesn't include the index anymore for repeated registers.
  https://github.com/rust-lang/rust/issues/144635

### 1.0.6 (18-06-25)

- Fixed regression introduced in 1.0.3 where signed integers were not sign-extended
  and would thus be positive instead of their intended two's-complement negative.

### 1.0.5 (12-06-25)

- Backend code generation switched from quote/syn to askama
- Fixed cfg gates on fields (didn't compile before)
- Fixed WO fields (would cause a compile error in generated code)
- Fixed docstring generation so we don't need to worry about escaping characters
- CLI: When no output file is provided, the output is printed to stdout instead
- CLI: No longer panics when unexpected error output is processed
- Error messages now more consistently use backticks (`) instead of various other quoting characters like (') and (")
- CLI: Added a new generation option. It defaults to Rust, but can now also be used to generate a KDL manifest
  - There's no way to use KDL yet, so this is purely a preview feature and the format might change in the future.
  - In the future this can be used to convert existing specs into KDL.
  - Possibly eventually this can be used to generate C code instead of Rust

### 1.0.4 (28-02-25)

- The driver name input is now a bit less strict about what 'PascalCase' means
  - It now also gives a value based on the input that would be accepted
- Update to edition 2024

### 1.0.3 (08-02-25)

- *(Technically breaking)*: removed the `pub use bitvec;` in the root of the crate.
  - It was an oversight to have that still there without `#[doc(hidden)]`
  - There was no reason for anyone to use it
  - Nothing publicly on github is using it
  - So we're doing 'scream testing'. If this impacts you, message me! I'll yank the release immediately
- Replaced bitvec with custom logic
  - Bitvec doesn't compile for Xtensa arch due to it being unmaintained
  - Custom logic is smaller and saves program size

### 1.0.2 (03-01-25)

- Add more docs to generated code so it passes the `missing_docs` lint

### 1.0.1 (02-01-25)

- Make defmt impls for fieldsets a bit more efficient by using type hints
- Fixed cfg problem in the `read_all_registers` function

### 1.0.0 (28-12-24)

- Enforce PascalCase for device name
- Improve CLI error display

### 1.0.0-rc.1 (29-11-24)

- *Breaking*: Generated fieldsets are now put in a `field_sets` module.
  - Type conversion paths now get `super::` prepended unless they start with `::` or `crate`
- Fixed some broken links
- Made clarifications in the book

### 1.0.0-rc.0 (10-11-24)

- *Breaking*: A lot, complete rewrite
  - Generated fields now use chiptool/embassy-style code
    - Getters have no prefix
    - Setters use `set_` as the prefix
  - Added a global config
    - Address types are now put in there
    - Option to provide defaults for memory ordering
    - Provides a way to change the name normalization options
  - Generated code no longer depends on `num_enum`
  - No more `R` and `W` structs. Everything is done on the main generated fieldset.
  - Formalized blocks
  - Added refs
  - Added repeats
  - Added many analysis steps to reduce the chance of mistakes
  - Fixed memory ordering oddities
  - Added toml support
  - Added an optional CLI to pregenerate the driver instead of using the macros
  - Split the interface from the driver
  - Type conversions are strict by default now, but generated enums are also more likely to qualify
  - Added optional defmt generation
  - Added an option to read all registers of a device (for debugging purposes)
  - All object types (register, command, buffer, block and ref) can now be mixed
    - In yaml and json, this changes the structure a bit and every object must have an extra `type` field to denote its object type

And more! Read the book linked in the readme to find all documentation.

### 0.7.0 (22-08-24)

- *Breaking*: Improved the API for dispatching commands
- Added buffer support
- Added strict mode conversion. This makes the types require `From<primitive>` instead of `TryFrom<primitive>`.
  But reading the register field is then not a result.
- Added byte order option to registers so they can be read and stored as little endian. (When not specified, it still defaults to big endian)
- Added BitAnd, BitOr and BitXor on the register structs. It's not as conistent as I'd like, but it'll be fixed in the next version hopefully.
- Added support for register blocks

### 0.6.0 (26-05-24)

- *Breaking*: Renamed the macros so they don't include the word 'register'
- Added a way to define commands

### 0.5.3 (08-01-24)

- Improved readme with hard links and more explanations
- Generated device register functions are now also documented

### 0.5.2 (08-01-24)

- Async read and modify worked with registers instead of their read and write types

### 0.5.1 (08-01-24)

- Removed accidental dependency that forced std

### 0.5.0 (07-01-24)

- *Breaking*: Built up the crate from the ground up again.
  Now more dependent on the type-system and all macros are proc-macros.
  This makes it a whole lot more maintainable and expandable.
- New register trait
- New device trait
- Almost all old features still supported

### 0.4.1 (13-12-22)
- Accidentally left the async flag on by default for 0.4.0 which caused it not to compile on stable.
### 0.4.0 (13-12-22) (yanked)
- Added async support for the register interfaces. Use the `async` feature flag to activate it.
  When you do, you will have access to the `ll::register_async` module that will generate async code for you.
- Updated dependencies (mainly bitvec to 1.0, which makes this release a technically breaking change)

### 0.3.1 (22-12-21)
- Added docs to low level error ([#14](https://github.com/diondokter/device-driver/pull/10))
### 0.3.0 (02-05-21)
- Added better `Debug` impls to all register `R` that prints the raw value in hex.
  There's now also the option (`#[generate(Debug)]`) to get an even better `Debug` impl that also prints out all the fields,
  but does require all fields to impl `Debug` themselves.
  See ([#10](https://github.com/diondokter/device-driver/pull/10)) to see how it works.
### 0.2.0 (19-04-21)
- All user interaction with a 'W' is now through &mut instead of directly to support more kinds of code structuring ([#7](https://github.com/diondokter/device-driver/pull/7))
### 0.1.0 (28-09-20)
- Original release