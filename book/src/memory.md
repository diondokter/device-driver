# Memory

Memory is quite easy. But assigning meaning to it is where all complexity comes from.
This page describes all the different levels of memory and what this crate does.
The goal is to leave you with a better understanding of how memory is handled and to serve as a quick reference if or when
confusion ensues.

- [Memory](#memory)
  - [Concepts](#concepts)
    - [Byte order](#byte-order)
    - [Bit order](#bit-order)
    - [Together](#together)
  - [The memories of device-driver](#the-memories-of-device-driver)
    - [Example LIS3DH - Multi-register LE, LSB0](#example-lis3dh---multi-register-le-lsb0)
    - [Example s2-lp - Multi-register BE, LSB0](#example-s2-lp---multi-register-be-lsb0)
    - [Example DW1000 - Single-register LE, LSB0](#example-dw1000---single-register-le-lsb0)

## Concepts

### Byte order

Also known as endianness. It describes what the first byte is in an array of bytes.
There are two options generally:

- Little endian (LE)
  - The smallest or first byte is at the front
  - I.E. `[10, 11, 12, 13]` where indexing at 0 would yield `10`
  - Lower indices are in lower memory addresses, higher indices are in higher memory addresses
- Big endian (BE)
  - The smallest or first byte is at the back
  - I.E. `[10, 11, 12, 13]` where indexing at 0 would yield `13`
  - Lower indices are in higher memory addresses, higher indices are in lower memory addresses

### Bit order

There is also order on the bit level. We get to decide which bit is the smallest of the 8 in a byte.
There's two options:

- Least significant bit 0 (LSB0)
  - The bit at index 0 is the lowest bit
  - I.E. the number `1` is coded as `0b0000_0001` or `0x01`
- Most significant bit 0 (MSB0)
  - The bit at index 0 is the highest bit
  - I.E. the number `1` is coded as `0b1000_0000` or `0x80`

### Together

> [!IMPORTANT]
> Together, the bit and byte order determine where a given bit is in an array of bytes.

Bit `0` is defined as the `0th` bit on the `0th` byte.  
Bit `10` is defined as the `2nd` bit on the `1st` byte.

Here are the options for when only bit 0 is high in a 2-byte array:

```
LE, LSB0:
  [0b0000_0001, 0b0000_0000] or [0x01, 0x00]
             ^            ^  <- Bits 0
   ^^^^^^^^^^^               <- Byte 0

LE, MSB0:
  [0b1000_0000, 0b0000_0000] or [0x80, 0x00]
     ^            ^          <- Bits 0
   ^^^^^^^^^^^               <- Byte 0

BE, LSB0:
  [0b0000_0000, 0b0000_0001] or [0x00, 0x01]
             ^            ^  <- Bits 0
                ^^^^^^^^^^^  <- Byte 0

BE, MSB0:
  [0b0000_0000, 0b1000_0000] or [0x00, 0x80]
     ^            ^          <- Bits 0
                ^^^^^^^^^^^  <- Byte 0
```

Here are the options for when only bit 10 is high in a 2-byte array:

```
LE, LSB0:
  [0b0000_0000, 0b0000_0100] or [0x00, 0x04]
           ^            ^    <- Bits 2
                ^^^^^^^^^^^  <- Byte 1

LE, MSB0:
  [0b0000_0000, 0b0010_0000] or [0x00, 0x20]
       ^            ^        <- Bits 2
                ^^^^^^^^^^^  <- Byte 1

BE, LSB0:
  [0b0000_0100, 0b0000_0000] or [0x04, 0x00]
           ^            ^    <- Bits 2
   ^^^^^^^^^^^               <- Byte 1

BE, MSB0:
  [0b0010_0000, 0b0000_0000] or [0x20, 0x00]
       ^            ^        <- Bits 2
   ^^^^^^^^^^^               <- Byte 1
```

## The memories of device-driver

> [!IMPORTANT]
> Here's the tricky part. The data of a register can be present in three places:
> 
> - On the device
> - On the transport bus (e.g. while writing/reading it over SPI)
> - In RAM on your microcontroller

The first two we don't have any influence over. But we can create our own model with this crate that fits the device.

Let's do some examples for real existing devices.
If there's a device that does things a bit different, feel free to PR this file!

> [!TIP]
> If all registers have the same behaviour (which is the case usually), you can set the bit and byte orders in the global config too so it applies to all registers that don't explicitly have it set.

### Example LIS3DH - Multi-register LE, LSB0

The LIS3DH accelerometer is one byte per register, but there are some multi-register values that we may want to model as one two-byte register. This can be done because the address will auto increment after any reads/writes.

In the datasheet we find the registers:

| Name    | Access | Address | Value       |
| ------- | ------ | ------- | ----------- |
| OUT_X_L | ro     | 0x28    | `X [0..8]`  |
| OUT_X_H | ro     | 0x29    | `X [8..16]` |

And the transport schema:

```
CS : ¯¯\___________________________________________________________/¯¯¯¯
SPC: ¯¯¯\_/¯\_/¯\_/¯\_/¯\_/¯\_/¯\_/¯\_/¯\_/¯\_/¯\_/¯\_/¯\_/¯\_/¯\_/¯¯¯¯¯
SDI: ===x===x===x===x===x===x===x===x===x===x===x===x===x===x===x===x===
         R!W                         DI7 DI6 DI5 DI4 DI3 DI2 DI1 DI0
             M!S AD5 AD4 AD3 AD2 AD0
SDO: -------------------------------x===x===x===x===x===x===x===x===x---
                                     DO7 DO6 DO5 DO4 DO3 DO2 DO1 DO0
```

Let's analyze:

Byte order
- We will make one register out of the two starting at address 0x28
- The first byte will be from `OUT_X_L` and the second byte will be from `OUT_X_H`
- So, low index is low byte and high index is high byte
- Thus this combined register is **little endian (LE)**

Bit order
- Depends on the hardware settings of the SPI. We set it to most significant bit first to match the datasheet.
- The 0th bit is the last and least significant one of the byte
- Thus this is **Least Significant Bit 0 (LSB0)**

And so we get our register definition:
```rust
register OutX {
    const ADDRESS = 0x68; // Including bit for multi-register ops
    const SIZE_BITS = 16;
    type ByteOrder = LE;
    type BitOrder = LSB0;

    value: int = 0..16,
}
```

### Example s2-lp - Multi-register BE, LSB0

This is a radio chip and just like the LIS3DH can combine multiple registers in one read/write.

In the datasheet we find the registers:

| Name  | Address | Bits | Value       |
| ----- | ------- | ---- | ----------- |
| SYNT3 | 05      | 7:5  | PLL_CP_ISEL |
|       |         | 4    | BS          |
|       |         | 3:0  | SYNT[27:24] |
| SYNT2 | 06      | 7:0  | SYNT[23:16] |
| SYNT1 | 07      | 7:0  | SYNT[15:8]  |
| SYNT0 | 08      | 7:0  | SYNT[7:0]   |

And the transport schema (for writes):

```
CSn : ¯¯\_______________________________________________________________________________________________/¯¯¯¯
SCLK: ¯¯¯\_/¯\_/¯\_/¯\_/¯\_/¯\_/¯\_/¯\_/¯\_/¯\_/¯\_/¯\_/¯\_/¯\_/¯\_/¯\_/¯\_/¯\_/¯\_/¯\_/¯\_/¯\_/¯\_/¯\_/¯¯¯¯¯
MOSI: ---x===x===x===x===x===x===x===x===x===x===x===x===x===x===x===x===x===x===x===x===x===x===x===x===x---
          A/C  0   0   0   0   0   0  W/R  A7  A6  A5  A4  A3  A2  A1  A0  D7  D6  D5  D4  D3  D2  D1  D0
         | header                        | address                       | data
```

Let's analyze:

Byte order
- We will make one register out of this starting at address 0x05
- The first byte will contain `SYNT[27:24]` and the last byte will contain `SYNT[7:0]`
- So, low index is high byte and high index is low byte
- Thus this combined register is **big endian (BE)**

Bit order
- Depends on the hardware settings of the SPI. We set it to most significant bit first to match the datasheet.
- The 0th bit is the last/least significant one
- Thus this is **Least Significant Bit 0 (LSB0)**

```rust
register OutX {
    const ADDRESS = 0x05;
    const SIZE_BITS = 32;
    type ByteOrder = BE;
    type BitOrder = LSB0;

    synt: uint = 0..=27,
    bs: bool = 28,
    pll_cp_isel: uint = 29..=31
}
```

### Example DW1000 - Single-register LE, LSB0

This chip doesn't have multi register reads, but it does have registers bigger than a byte.
So even a single register must take care of byte ordering.

Luckily for us, the user manual spells out the modes (along to the diagrams):
- > Note: The octets of a multi-octet value are transferred on the SPI interface in octet order beginning with the low-order octet.
- Diagram example: Register `0x00` contains `0xDECA0130` and is sent as `[0x30, 0x01, 0xCA, 0xDE]`
  - Thus **little endian (LE)**
- > Note: The octets are physically presented on the SPI interface data lines with the high order bit sent first in time.
  - Depends on the hardware settings of the SPI. We set it to most significant bit first to match the datasheet.
  - Thus **Least Significant Bit 0 (LSB0)** (assuming your SPI master also sees the first bit as the LSB)

```rust
register DevId {
    const ADDRESS = 0x00;
    const SIZE_BITS = 32;
    type ByteOrder = LE;
    type BitOrder = LSB0;

    r_id_tag: uint = 16..32,
    model: uint = 8..16,
    ver: uint = 4..8,
    rev: uint = 0..4
}
```
