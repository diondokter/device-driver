# Language

Device-driver uses a simple, custom, declarative language called DDSL (device-driver specification language).
It consists of only a few building blocks: Nodes, properties and expressions.

## Node

The [node](./language-tokens_ast.html#node) is the foundation. A ddsl file must have one root node and everything else must be defined in it. A node defines an object and in many cases these terms are interchangable. (It's a node in the AST and an object in the MIR)

Nodes have a node type and a name. Additionally a node may have a [repeat](./language-tokens_ast.html#repeat) specifier, ['short' properties](./language-tokens_ast.html#simple-expression) and a [type specifier](./language-tokens_ast.html#type-specifier) outside of the node body and ['long' properties](language-tokens_ast.html#property) and subnodes in the [node body](./language-tokens_ast.html#node-body).

```ddsl
type name[repeat] <expressions> -> <type specifier> {
    // Inside the curly's is the node body

    // Long properties are named expressions
    long-property-name: <expression>,

    subnode-type subnode-name // ...
}
```

The node type determines the final shape of the node that's accepted.
There are a couple of defined node types:

- [Manifest](./language-manifest.md)
- [Device](./language-device.md)
- [Block](./language-block.md)
- [Register](./language-register.md)
- [Command](./language-command.md)
- [Buffer](./language-buffer.md)
- [Fieldset](./language-fieldset.md)
- [Enum](./language-enum.md)
- [Extern](./language-extern.md)
- [Field](./language-field.md)

Any node type not on this list is rejected by the compiler.

## Properties

Properties come in two forms: 'short' and 'long'.

### Short

Short properties are anonymous [(simple) expressions](./language-tokens_ast.html#simple-expression) that appear between the node name and the type specifier, outside the node body. This makes them limited as the type of the expression determines what they mean.

In practice, short properties are mostly used for fields where compact notation is important.

### Long

A [long property](./language-tokens_ast.html#property) is a named expression in the node body. The name and the expression are separated by a colon. They need to be written before any subnode in a node body.

The name of the property determines what the expression is used for. For all node types except enums, the name must match one of the defined properties for that node type. In enums, however, properties are used to define the enum variants and can take any name.

## Namespacing

In DDSL there's one global namespace that all* objects are part of. However, not all names will clash.

There are two buckets a name can be categorized into:
- Operation
- Type

An operation is something that's *done* with a driver. Every operation becomes a method you can call on a device/block.

Meanwhile a type is a data definition or collection of operations and these become structs and enums in the generated code.

As an example, a register "Foo" is allowed to define a fieldset "Foo". The register is an operation and the fieldset is a type.

Operations:
- Manifest
- Block
- Register
- Command
- Buffer

Types:
- Manifest
- Device
- Block
- Fieldset
- Enum
- Extern

Notice how manifests and blocks are part of both.

*: Enum variants and fields are sort of objects, but they are namespaced within their defining enum/fieldset.
