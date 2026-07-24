# Device-driver toolkit
[![crates.io](https://img.shields.io/crates/v/device-driver.svg)](https://crates.io/crates/device-driver) [![Documentation](https://docs.rs/device-driver/badge.svg)](https://docs.rs/device-driver)

> A toolkit to write better device drivers, faster.

This book aims to guide you to write your own device drivers using the device-driver toolkit.
For runtime docs, visit [docs.rs](https://docs.rs/device-driver).

Device-driver uses a small custom language named DDSL (device driver specification language) as its input. It's made so creating drivers is direct, easy and to-the-point.

Example of a DDSL register:
```ddsl
register SYNT {
    address: 0x05,
    reset: 0x42162762,
    fields: fieldset _ {
        size-bytes: 4,
        byte-order: BE,
        
        /// Set the charge pump current according to the XTAL frequency
        /// (see Table 37. Table 34).
        field PLL_CP_ISEL 31:29 -> uint,
        /// Synthesizer band select. This parameter selects the out-of loop
        /// divide factor of the synthesizer:
        /// - false: 4, band select factor for high band
        /// - true: 8, band select factor for middle band
        /// (see Section 5.3.1 RF channel frequency settings).
        field BS 28 -> bool,
        /// The PLL programmable divider
        /// (see Section 5.3.1 RF channel frequent settings).           
        field SYNT 27:0 -> uint
    }
}
```

> [!NOTE]
> The name `device-driver` consists of two parts:
> - `driver`: Code to enable the use of hardware.
> - `device`: A chip or peripheral you can talk to over a bus.
>
> Examples of good targets for using this toolkit:
> - An I2C accelerometer
> - A SPI radio transceiver
> - A screen/display with parallel bus
>
> The driver is usable in any no-std context and can be made to work with the `embedded-hal` crate or any custom interfaces.

## Book overview:

The book contains documentation for multiple versions.
Go to the version you're using and continue to read there.
If you're new, then the most recent version is recommended.

The book has sections on usage, tutorials and language specs.

The addendums contain useful background information.

> [!CAUTION]
> It's hard to keep a book like this up-to-date with reality. Small errors might creep in despite my best effort.
> If you do find something out of place, missing or simply wrong, please open an issue or PR, even if it's just for a typo! I'd really appreciate it and helps out everyone.

## Known drivers using the toolkit:

It's nice to have examples:

V2:

- None yet

V1:

- [S2-LP radio](https://github.com/diondokter/s2lp)
- [Nordic nPM1300 Power Management IC](https://github.com/thermigo/npm1300-rs)
- [iqs323 inductive/capacitive sensing controller](https://github.com/tactile-eng/iqs323-driver)
- [AXP192 Power Management IC](https://github.com/okhsunrog/axp192-dd)
- [ONSEMI FUSB302B USB-PD PHY](https://github.com/okhsunrog/fusb302b)
- [iC-Haus iC-MD 48bit quadrature counter](https://github.com/trappitsch/ic-md)
- [STMicroelectronics LIS2DE12 3-axis accelerometer](https://github.com/leftger/lis2de12)
- [ISSI IS25LP128F 128Mbit SPI NOR Flash](https://github.com/leftger/is25lp128f)
- [ON Semiconductor CAT25040 4kbit SPI EEPROM](https://github.com/cesardtamayo/cat25040)
- [TI BQ27441 Battery Fuel Gauge IC](https://github.com/leftger/bq27441)
- [TI BQ25887 2-Cell Battery Charger IC](https://github.com/leftger/bq25887)
- [TI ADC Bus Expanders](https://github.com/activexray/ti-adc-expander)

Feel free to add to this list!
