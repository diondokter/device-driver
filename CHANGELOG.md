## Changelog

### Unreleased

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