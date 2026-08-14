# Migrating v1 to v2

If you want to migrate an existing v1 driver to v2, you'll read some of the steps here and some of the issues you may encounter.

## Convert

The `ddc` cli can do a lot of the mechanical conversion for you.

```sh
ddc convert device-driver-v1 --sub-format yaml -s ./device.yaml -o ./device.ddsl
```

Change out the paths as you like and change the subformat to the format you've used. If you've been using the `create_device!` with the inline DSL, first copy the DSL to a file and feed that to the tool.

If your install of `ddc` doesn't have the convert command, you need to reinstall it with the `--features converter-dd-v1` flag.

The convert command converts the formats very straightforwardly and often doesn't do a perfect job, so some hand tuning is still required.

### Cfg

Cfg doesn't exist anymore and there's currently no replacement. If you really need this, you may need to keep your driver on v1. There are plans to build template support in the language: [issue](https://github.com/diondokter/device-driver/issues/58), so if you really need this and have ideas, please contribute there!

The converter ignores cfg values.

### Naming

In v2, the names of objects is a little stricter. You may find that before you had a register, field and enum all with the same name. This would now clash since a register now has a named fieldset. Enums and fieldsets are types and they must not have the same name. See the section on [namespacing](./v2/language.md#namespacing).

The converter copies the names as they were and does not fix them up. You'll have to figure out alternative names.

### Ref objects

Ref objects don't exist anymore. Ref registers and ref commands have good alternatives, though.

Instead of having overrides, you simply copy the values as though it's a fully separate register or command. But then instead of defining a new fieldset, you can reuse a fieldset. The converter does this for you.

For ref blocks you either have to do a bunch of copying manually, or use a (enum) repeat. The converter tool just copies everything (which will likely lead to name collisions you'll have to fix).

### Extern types

In v1 you could reference something that wasn't defined in the manifest. The generated code would then just use the name as is. This must now be done much more explicit as everything must be self-contained in the manifest.

You must define an [extern](./v2/language-extern.md) object for every type reference that used to refer to a rust object. The converter does not do this for you as it can't know some of the metadata.

### Formatting

The converter doesn't know about number formatting and outputs everything in decimal.

### Bit order

Bit order doesn't exist anymore. It was a very hard feature to support for something that's barely ever used. Everything is now `lsb0`.

If the converter finds something with `msb0`, it bails. Remove or change it in the v1 manifest and run the converter again.

## Crate

Now that we've converted the v1 yaml, json, toml or DSL to DDSL, we can focus on our Rust crate that requires some changes too.

1. Update the device-driver dependency to 2.0.0 (or newer)
   - If you're using the macro to compile the source, enable the `macros` feature on the crate
2. If you're using macro to compile the source, update the macro:
   ```rust
    - device_driver::create_device!(
    -     device_name: Device,
    -     manifest: "device.yaml"
    - );
    + device_driver::compile!(
    +     options: "--rust-defmt-feature=defmt",
    +     manifest: "device.ddsl"
    + );
   ```
   Note that the device name is now specified in the manifest and the defmt option at the compiler invocation.
3. If you have a `build.rs`, check it and update any reference to the old manifest to the new file name.
4. Update your interface types
   - They now all have a base trait for the error and address types
   - Size-bits is no longer given
   - A metadata object is now given
   - All buffers are now mutable for the case you need to do bit swaps
5. Fieldsets are not generated into a module anymore. You may need to change some imports to remove the `field_sets` module from the path. The Rust compiler will tell you where this needs to happen.
6. Fieldsets don't have a constructor anymore. If you've called the `new_zero()` on fieldsets before, you'll now need to use the `ZERO` associated const (for which you'll need to import the `Fieldset` trait from the device-driver crate).
7. Repeat index parameter has moved. It used to be a parameter on the operation getter function. Now you specify it on the operation. For example:
   ```rust
   - device.foo(index).read()?;
   + device.foo().read_at(index)?;
   ```
