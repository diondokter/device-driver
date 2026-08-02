# Manifest

The manifest is the root of a driver and is the input to the compiler.
All objects that make up the driver are defined in it.

To save on boilerplate, if you only have one [device](./language-device.md) in your driver, you can forego specifying the manifest and just have a device as the root object.

All config variables present on devices are available here too and serve as the default config for all devices. Devices can then override them again.

{{#include ../gen-docs/mir-shapes/manifest.md}}

[device]: ./language-device.md
[fieldset]: ./language-fieldset.md
[enum]: ./language-enum.md
[extern]: ./language-extern.md
