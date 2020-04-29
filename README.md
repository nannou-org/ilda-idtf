# ilda-idtf [![Actions Status](https://github.com/nannou-org/ilda-idtf/workflows/ilda-idtf/badge.svg)](https://github.com/nannou-org/ilda-idtf/actions) [![Crates.io](https://img.shields.io/crates/v/ilda-idtf.svg)](https://crates.io/crates/ilda-idtf) [![Crates.io](https://img.shields.io/crates/l/ilda-idtf.svg)](https://github.com/nannou-org/ilda-idtf/blob/master/LICENSE-MIT) [![docs.rs](https://docs.rs/ilda-idtf/badge.svg)](https://docs.rs/ilda-idtf/)

A complete implementation of the ILDA Image Data Transfer Format Specification,
Revision 011, 2014-11-16.

This library provides efficient reading and writing of IDTF. The reader
implementation uses a zero-copy approach where structures are mapped directly to
the memory from which they are read.

## Usage

The [**SectionReader**][1] type can be used to read IDTF sections from any type
implementing `std::io::Read`. This allows for efficiently reading the format
from any byte source (e.g. file, memory, socket, etc).

```rust
let mut reader = ilda_idtf::SectionReader::new(reader);
```

The [**open**][2] function is provided as a convenience for opening a buffered
IDTF **SectionReader** for the file at the given path.

```rust
let mut reader = ilda_idtf::open(path).unwrap();
```

The [**SectionReader::read_next**][3] method allows for iteration over the
sections contained within.

```rust
while let Ok(Some(section)) = reader.read_next() {
    // ...
}
```

Each yielded [**Section**][4] provides access to the [**Header**][5] and the
inner `reader`. The exact `reader` kind is determined via the [**Format**][6]
specified within the header. The user must pattern match on the section's
`reader` field in order to retrieve an instance of the correct subsection reader
type. The user may then read the associated subsection data.

```rust
match section.reader {
    ilda_idtf::SubsectionReaderKind::Coords3dIndexedColor(mut r) => {
        while let Some(point) = r.read_next().unwrap() {
            // ...
        }
    }
    ilda_idtf::SubsectionReaderKind::Coords2dIndexedColor(mut r) => {
        while let Some(point) = r.read_next().unwrap() {
            // ...
        }
    }
    ilda_idtf::SubsectionReaderKind::ColorPalette(mut r) => {
        while let Some(palette) = r.read_next().unwrap() {
            // ...
        }
    }
    ilda_idtf::SubsectionReaderKind::Coords3dTrueColor(mut r) => {
        while let Some(point) = r.read_next().unwrap() {
            // ...
        }
    }
    ilda_idtf::SubsectionReaderKind::Coords2dTrueColor(mut r) => {
        while let Some(point) = r.read_next().unwrap() {
            // ...
        }
    }
}
```

In order to interpret the indexed color data formats, a color palette must be
used. A color palette is an ordered list of RGB colors. While the palette
*should* be specified by a preceding `Section`, this is not always the case. The
ILDA IDTF specification recommends a default palette. This palette is provided
via the [**DEFAULT_PALETTE**][7] constant.

[1]: https://docs.rs/ilda-idtf/latest/ilda_idtf/struct.SectionReader.html
[2]: https://docs.rs/ilda-idtf/latest/ilda_idtf/fn.open.html
[3]: https://docs.rs/ilda-idtf/latest/ilda_idtf/struct.SectionReader.html#method.read_next
[4]: https://docs.rs/ilda-idtf/latest/ilda_idtf/struct.Section.html
[5]: https://docs.rs/ilda-idtf/latest/ilda_idtf/layout/struct.Header.html
[6]: https://docs.rs/ilda-idtf/latest/ilda_idtf/layout/struct.Format.html
[7]: https://docs.rs/ilda-idtf/latest/ilda_idtf/constant.DEFAULT_PALETTE.html


License
-------

Licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

**Contributions**

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.

**Test Files**

The files within the `test_files` directory have been retrieved from the
following site:

http://www.laserfx.com/Backstage.LaserFX.com/Archives/DownloadIndex.html

These files are ***not*** included under the license mentioned above. Each
`test_files/` subdirectory is provided from a different group of laserists.
Please see the `ReadMe.txt` in each for more information on their origins and
conditions of use.
