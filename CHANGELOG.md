## Changelog

### Unreleased

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