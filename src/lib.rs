//! A complete implementation of the ILDA Image Data Transfer Format Specification, Revision 011,
//! 2014-11-16.

#[macro_use]
extern crate bitflags;

use std::{
    io::{self, Read},
    mem,
    path::Path,
};

pub mod layout;

/// A helper trait for producing and working with precisely sized buffers for IDTF layout.
pub trait LayoutBuffer: zerocopy::FromBytes {
    type Buffer;
    fn empty() -> Self::Buffer;
    fn slice(buffer: &Self::Buffer) -> &[u8];
    fn slice_mut(buffer: &mut Self::Buffer) -> &mut [u8];
}

/// Reads a sequence of frames from the ILDA IDTF spec from a stream of bytes.
///
/// Reads `Section`s.
pub struct SectionReader<R> {
    reader: R,
    buffer: [u8; mem::size_of::<layout::Header>()],
}

/// Contains a verified **Header** and a reader for the section contents.
pub struct Section<'a, R>
where
    R: Read,
{
    pub header: &'a layout::Header,
    pub reader: SubsectionReaderKind<R>,
}

/// Reads `len` consecutive subsections of type `T`.
pub struct SubsectionReader<R, T>
where
    R: Read,
    T: LayoutBuffer,
{
    reader: R,
    len: u16,
    buffer: T::Buffer,
    subsection_layout: std::marker::PhantomData<T>,
}

pub type Coords3dIndexedColorReader<R> = SubsectionReader<R, layout::Coords3dIndexedColor>;
pub type Coords2dIndexedColorReader<R> = SubsectionReader<R, layout::Coords2dIndexedColor>;
pub type ColorPaletteReader<R> = SubsectionReader<R, layout::ColorPalette>;
pub type Coords3dTrueColorReader<R> = SubsectionReader<R, layout::Coords3dTrueColor>;
pub type Coords2dTrueColorReader<R> = SubsectionReader<R, layout::Coords2dTrueColor>;

/// The subsection reader kind determined via the header's `format` field.
pub enum SubsectionReaderKind<R>
where
    R: Read,
{
    Coords3dIndexedColor(Coords3dIndexedColorReader<R>),
    Coords2dIndexedColor(Coords2dIndexedColorReader<R>),
    ColorPalette(ColorPaletteReader<R>),
    Coords3dTrueColor(Coords3dTrueColorReader<R>),
    Coords2dTrueColor(Coords2dTrueColorReader<R>),
}

impl<R> SectionReader<R>
where
    R: Read,
{
    /// Read ILDA IDTF sections from the given reader.
    pub fn new(reader: R) -> Self {
        let buffer = [0u8; mem::size_of::<layout::Header>()];
        SectionReader { reader, buffer }
    }

    /// Begin reading the next **Section**.
    ///
    /// A successfully read **Section** contains a verified **Header** and a reader for the section
    /// contents.
    pub fn read_next(&mut self) -> io::Result<Option<Section<&mut R>>> {
        let SectionReader {
            ref mut buffer,
            ref mut reader,
        } = *self;

        // Buffer the header bytes.
        if let Err(err) = reader.read_exact(buffer) {
            if let io::ErrorKind::UnexpectedEof = err.kind() {
                return Ok(None);
            }
        }

        // Verify the header layout.
        let header: &layout::Header = zerocopy::LayoutVerified::new(&buffer[..])
            .map(zerocopy::LayoutVerified::into_ref)
            .ok_or_else(|| {
                let err_msg = "could not verify the layout of `Header`";
                io::Error::new(io::ErrorKind::InvalidData, err_msg)
            })?;

        // Validate header by ascii "ILDA".
        if header.ilda != layout::Header::ILDA {
            let err_msg = "could not verify `Header` due to invalid ILDA ascii";
            return Err(io::Error::new(io::ErrorKind::InvalidData, err_msg));
        }

        // Determine the format.
        let len = header.num_records.get();
        let reader = match header.format {
            layout::Format::COORDS_3D_INDEXED_COLOR => {
                Coords3dIndexedColorReader::new(reader, len).into()
            }
            layout::Format::COORDS_2D_INDEXED_COLOR => {
                Coords2dIndexedColorReader::new(reader, len).into()
            }
            layout::Format::COLOR_PALETTE => ColorPaletteReader::new(reader, len).into(),
            layout::Format::COORDS_3D_TRUE_COLOR => {
                Coords3dTrueColorReader::new(reader, len).into()
            }
            layout::Format::COORDS_2D_TRUE_COLOR => {
                Coords2dTrueColorReader::new(reader, len).into()
            }
            _ => {
                let err_msg = "could not verify the layout of `Header`";
                return Err(io::Error::new(io::ErrorKind::InvalidData, err_msg));
            }
        };

        Ok(Some(Section { header, reader }))
    }
}

impl<R, T> SubsectionReader<R, T>
where
    R: Read,
    T: LayoutBuffer,
{
    fn new(reader: R, len: u16) -> Self {
        let buffer = T::empty();
        let subsection_layout = std::marker::PhantomData;
        Self {
            reader,
            len,
            buffer,
            subsection_layout,
        }
    }

    /// The number of remaining subsections expected.
    pub fn len(&self) -> u16 {
        self.len
    }

    /// Read the next subsection.
    pub fn read_next(&mut self) -> io::Result<Option<&T>> {
        match self.len {
            0 => return Ok(None),
            ref mut n => *n -= 1,
        }
        self.reader.read_exact(T::slice_mut(&mut self.buffer))?;
        let subsection = zerocopy::LayoutVerified::new(T::slice(&self.buffer))
            .map(zerocopy::LayoutVerified::into_ref)
            .ok_or_else(|| {
                let err_msg = "could not verify the layout of `Header`";
                io::Error::new(io::ErrorKind::InvalidData, err_msg)
            })?;
        Ok(Some(subsection))
    }
}

impl<R, T> Drop for SubsectionReader<R, T>
where
    R: Read,
    T: LayoutBuffer,
{
    fn drop(&mut self) {
        while let Ok(Some(_)) = self.read_next() {}
    }
}

impl LayoutBuffer for layout::Coords3dIndexedColor {
    type Buffer = [u8; mem::size_of::<Self>()];
    fn empty() -> Self::Buffer {
        [0u8; mem::size_of::<Self>()]
    }
    fn slice(buffer: &Self::Buffer) -> &[u8] {
        &buffer[..]
    }
    fn slice_mut(buffer: &mut Self::Buffer) -> &mut [u8] {
        &mut buffer[..]
    }
}

impl LayoutBuffer for layout::Coords2dIndexedColor {
    type Buffer = [u8; mem::size_of::<Self>()];
    fn empty() -> Self::Buffer {
        [0u8; mem::size_of::<Self>()]
    }
    fn slice(buffer: &Self::Buffer) -> &[u8] {
        &buffer[..]
    }
    fn slice_mut(buffer: &mut Self::Buffer) -> &mut [u8] {
        &mut buffer[..]
    }
}

impl LayoutBuffer for layout::ColorPalette {
    type Buffer = [u8; mem::size_of::<Self>()];
    fn empty() -> Self::Buffer {
        [0u8; mem::size_of::<Self>()]
    }
    fn slice(buffer: &Self::Buffer) -> &[u8] {
        &buffer[..]
    }
    fn slice_mut(buffer: &mut Self::Buffer) -> &mut [u8] {
        &mut buffer[..]
    }
}

impl LayoutBuffer for layout::Coords3dTrueColor {
    type Buffer = [u8; mem::size_of::<Self>()];
    fn empty() -> Self::Buffer {
        [0u8; mem::size_of::<Self>()]
    }
    fn slice(buffer: &Self::Buffer) -> &[u8] {
        &buffer[..]
    }
    fn slice_mut(buffer: &mut Self::Buffer) -> &mut [u8] {
        &mut buffer[..]
    }
}

impl LayoutBuffer for layout::Coords2dTrueColor {
    type Buffer = [u8; mem::size_of::<Self>()];
    fn empty() -> Self::Buffer {
        [0u8; mem::size_of::<Self>()]
    }
    fn slice(buffer: &Self::Buffer) -> &[u8] {
        &buffer[..]
    }
    fn slice_mut(buffer: &mut Self::Buffer) -> &mut [u8] {
        &mut buffer[..]
    }
}

impl<R> From<Coords3dIndexedColorReader<R>> for SubsectionReaderKind<R>
where
    R: Read,
{
    fn from(r: Coords3dIndexedColorReader<R>) -> Self {
        Self::Coords3dIndexedColor(r)
    }
}

impl<R> From<Coords2dIndexedColorReader<R>> for SubsectionReaderKind<R>
where
    R: Read,
{
    fn from(r: Coords2dIndexedColorReader<R>) -> Self {
        Self::Coords2dIndexedColor(r)
    }
}

impl<R> From<ColorPaletteReader<R>> for SubsectionReaderKind<R>
where
    R: Read,
{
    fn from(r: ColorPaletteReader<R>) -> Self {
        Self::ColorPalette(r)
    }
}

impl<R> From<Coords3dTrueColorReader<R>> for SubsectionReaderKind<R>
where
    R: Read,
{
    fn from(r: Coords3dTrueColorReader<R>) -> Self {
        Self::Coords3dTrueColor(r)
    }
}

impl<R> From<Coords2dTrueColorReader<R>> for SubsectionReaderKind<R>
where
    R: Read,
{
    fn from(r: Coords2dTrueColorReader<R>) -> Self {
        Self::Coords2dTrueColor(r)
    }
}

/// A `SectionReader` that reads from a buffered file.
pub type BufFileSectionReader = SectionReader<io::BufReader<std::fs::File>>;

/// Open the file at the given path as a `SectionReader`.
///
/// Returns a `SectionReader` that performs buffered reads on the file at the given path.
pub fn open<P>(path: P) -> io::Result<BufFileSectionReader>
where
    P: AsRef<Path>,
{
    open_path(path.as_ref())
}

fn open_path(path: &Path) -> io::Result<BufFileSectionReader> {
    let file = std::fs::File::open(path).unwrap();
    let buf_reader = std::io::BufReader::new(file);
    Ok(SectionReader::new(buf_reader))
}

/// As recommended in the specification appendix.
pub const DEFAULT_PALETTE: [layout::Color; 64] = [
    layout::Color {
        red: 255,
        green: 0,
        blue: 0,
    },
    layout::Color {
        red: 255,
        green: 16,
        blue: 0,
    },
    layout::Color {
        red: 255,
        green: 32,
        blue: 0,
    },
    layout::Color {
        red: 255,
        green: 48,
        blue: 0,
    },
    layout::Color {
        red: 255,
        green: 64,
        blue: 0,
    },
    layout::Color {
        red: 255,
        green: 80,
        blue: 0,
    },
    layout::Color {
        red: 255,
        green: 96,
        blue: 0,
    },
    layout::Color {
        red: 255,
        green: 112,
        blue: 0,
    },
    layout::Color {
        red: 255,
        green: 128,
        blue: 0,
    },
    layout::Color {
        red: 255,
        green: 144,
        blue: 0,
    },
    layout::Color {
        red: 255,
        green: 160,
        blue: 0,
    },
    layout::Color {
        red: 255,
        green: 176,
        blue: 0,
    },
    layout::Color {
        red: 255,
        green: 192,
        blue: 0,
    },
    layout::Color {
        red: 255,
        green: 208,
        blue: 0,
    },
    layout::Color {
        red: 255,
        green: 224,
        blue: 0,
    },
    layout::Color {
        red: 255,
        green: 240,
        blue: 0,
    },
    layout::Color {
        red: 255,
        green: 255,
        blue: 0,
    },
    layout::Color {
        red: 224,
        green: 255,
        blue: 0,
    },
    layout::Color {
        red: 192,
        green: 255,
        blue: 0,
    },
    layout::Color {
        red: 160,
        green: 255,
        blue: 0,
    },
    layout::Color {
        red: 128,
        green: 255,
        blue: 0,
    },
    layout::Color {
        red: 96,
        green: 255,
        blue: 0,
    },
    layout::Color {
        red: 64,
        green: 255,
        blue: 0,
    },
    layout::Color {
        red: 32,
        green: 255,
        blue: 0,
    },
    layout::Color {
        red: 0,
        green: 255,
        blue: 0,
    },
    layout::Color {
        red: 0,
        green: 255,
        blue: 36,
    },
    layout::Color {
        red: 0,
        green: 255,
        blue: 73,
    },
    layout::Color {
        red: 0,
        green: 255,
        blue: 109,
    },
    layout::Color {
        red: 0,
        green: 255,
        blue: 146,
    },
    layout::Color {
        red: 0,
        green: 255,
        blue: 182,
    },
    layout::Color {
        red: 0,
        green: 255,
        blue: 219,
    },
    layout::Color {
        red: 0,
        green: 255,
        blue: 255,
    },
    layout::Color {
        red: 0,
        green: 227,
        blue: 255,
    },
    layout::Color {
        red: 0,
        green: 198,
        blue: 255,
    },
    layout::Color {
        red: 0,
        green: 170,
        blue: 255,
    },
    layout::Color {
        red: 0,
        green: 142,
        blue: 255,
    },
    layout::Color {
        red: 0,
        green: 113,
        blue: 255,
    },
    layout::Color {
        red: 0,
        green: 85,
        blue: 255,
    },
    layout::Color {
        red: 0,
        green: 56,
        blue: 255,
    },
    layout::Color {
        red: 0,
        green: 28,
        blue: 255,
    },
    layout::Color {
        red: 0,
        green: 0,
        blue: 255,
    },
    layout::Color {
        red: 32,
        green: 0,
        blue: 255,
    },
    layout::Color {
        red: 64,
        green: 0,
        blue: 255,
    },
    layout::Color {
        red: 96,
        green: 0,
        blue: 255,
    },
    layout::Color {
        red: 128,
        green: 0,
        blue: 255,
    },
    layout::Color {
        red: 160,
        green: 0,
        blue: 255,
    },
    layout::Color {
        red: 192,
        green: 0,
        blue: 255,
    },
    layout::Color {
        red: 224,
        green: 0,
        blue: 255,
    },
    layout::Color {
        red: 255,
        green: 0,
        blue: 255,
    },
    layout::Color {
        red: 255,
        green: 32,
        blue: 255,
    },
    layout::Color {
        red: 255,
        green: 64,
        blue: 255,
    },
    layout::Color {
        red: 255,
        green: 96,
        blue: 255,
    },
    layout::Color {
        red: 255,
        green: 128,
        blue: 255,
    },
    layout::Color {
        red: 255,
        green: 160,
        blue: 255,
    },
    layout::Color {
        red: 255,
        green: 192,
        blue: 255,
    },
    layout::Color {
        red: 255,
        green: 224,
        blue: 255,
    },
    layout::Color {
        red: 255,
        green: 255,
        blue: 255,
    },
    layout::Color {
        red: 255,
        green: 224,
        blue: 224,
    },
    layout::Color {
        red: 255,
        green: 192,
        blue: 192,
    },
    layout::Color {
        red: 255,
        green: 160,
        blue: 160,
    },
    layout::Color {
        red: 255,
        green: 128,
        blue: 128,
    },
    layout::Color {
        red: 255,
        green: 96,
        blue: 96,
    },
    layout::Color {
        red: 255,
        green: 64,
        blue: 64,
    },
    layout::Color {
        red: 255,
        green: 32,
        blue: 32,
    },
];
