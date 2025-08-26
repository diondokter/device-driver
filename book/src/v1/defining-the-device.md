# Defining the device

This toolkit brings three different kinds of concepts you can use to build various aspects of your driver.

- The register is some memory located at an address on the device. It contains fields, may have a reset value and could be restrictive in its read and write access.
- The command can model multiple things. It can be an event to send to the device so it changes state or it could be an RPC-like call. It can send data and receive back an answer.
- The buffer is anything that you'd like to have a `Write` or `Read` interface to. A good example is a fifo buffer in a radio chip.

The registers, commands and buffers can be grouped into blocks.

Except for buffers all of them can be repeated and ref'ed. Repeats take the same object and repeat them for a repeat count with an address stride.
A 'ref' object copies another object and allows to override some values like the address and access.

The registers, commands, buffers, blocks and refs are all called 'objects' in this project.

To configure the driver, there's the global config. In it you can define the address types, various defaults for e.g. byte ordering and the method used for name normalization.

These concepts and how you can use them in your driver are described in more detail in their own chapter.
