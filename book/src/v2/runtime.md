# Runtime

Device-driver intentionally doesn't generate *all* code for a driver. The code depends on a small runtime.

The runtime is responsible for defining all the operations (e.g. register reads/writes), the interfaces and the bit manipulation routines.

Each compilation target has its own runtime (or doesn't have any runtime in some cases). See the other chapters for more information about specific targets.
