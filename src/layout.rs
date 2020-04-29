//! An ILDA file consists of sections which either contain a frame or a color palette. Eachsection
//! consists of a fixed length header followed by a variable number of data records, which are
//! either frame points or color palette colors.

use zerocopy::{AsBytes, FromBytes, Unaligned};

/// The endianness of bytes read from and written to the format.
pub type Endianness = byteorder::BigEndian;
type I16 = zerocopy::byteorder::I16<Endianness>;
type U16 = zerocopy::byteorder::U16<Endianness>;

/// The type and data format of the section is defined by the format code.
///
/// There are five different formats currently defined.
///
/// Format 3 was proposed within the ILDA Technical Committee but was never approved. Therefore,
/// format 3 is omitted in this ILDA standard.
///
/// Formats 0, 1, 4 and 5 define point data. Each point includes X and Y coordinates, andcolor
/// information. The 3D formats 0 and 4 also include Z (depth) information.
///
/// The indexed color formats 0 and 1 use a data format where each point has a Color Indexbetween 0
/// and 255 used as an index into a color palette. Format 2 specifies the colorpalette for use with
/// indexed color frames. The true color formats 4 and 5 use a red,green and blue color component
/// of 8 bits for each point. ILDA files may contain a mix offrames with several
/// different format codes.
#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq, AsBytes, FromBytes, Unaligned)]
#[repr(C)]
pub struct Format(pub u8);

#[derive(Copy, Clone, Eq, Hash, PartialEq, AsBytes, FromBytes, Unaligned)]
#[repr(C)]
pub struct Name(pub [u8; 8]);

/// Describes the layout of a section of IDTF.
#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq, AsBytes, FromBytes, Unaligned)]
#[repr(C)]
pub struct Header {
    /// The ASCII letters ILDA, identifying an ILDA format header.
    pub ilda: [u8; 4],
    /// Reserved for future use. Must be zeroed.
    pub reserved: [u8; 3],
    /// One of the format codes defined in the Format Codes section.
    pub format: Format,
    /// Eight ASCII characters with the name of this frame or color palette. If abinary zero is
    /// encountered, than any characters following the zero SHALL be ignored.
    pub data_name: Name,
    /// Eight ASCII characters with the name of the company who created theframe. If a binary zero
    /// is encountered, than any characters following the zero SHALL beignored.
    pub company_name: Name,
    /// Total number of data records (points or colors) that will follow this headerexpressed as an
    /// unsigned integer (0 – 65535). If the number of records is 0, then this is to be taken as
    /// the end of file header and nomore data will follow this header. For color palettes, the
    /// number of records SHALL be between 2 and 256.
    pub num_records: U16,
    /// Frame or color palette number. If the frame is part of a group such as an animation
    /// sequence, thisrepresents the frame number. Counting begins with frame 0. Range is 0 –
    /// 65534.
    pub data_number: U16,
    /// Total frames in this group or sequence. Range is 1 – 65535.
    ///
    /// For colorpalettes this SHALL be 0.
    pub color_or_total_frames: U16,
    /// The projector number that this frame is to be displayed on. Range is 0 – 255. For single
    /// projector files this SHOULD be set 0.
    pub projector_number: u8,
    /// Reserved for future use. Must be zeroed.
    pub reserved2: u8,
}

#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq, AsBytes, FromBytes, Unaligned)]
#[repr(C)]
pub struct Coords3d {
    /// left negative, right positive.
    pub x: I16,
    /// down negative, up positive.
    pub y: I16,
    /// far negative, near positive.
    pub z: I16,
}

#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq, AsBytes, FromBytes, Unaligned)]
#[repr(C)]
pub struct Coords2d {
    /// left negative, right positive.
    pub x: I16,
    /// down negative, up positive.
    pub y: I16,
}

bitflags! {
    #[derive(AsBytes, FromBytes, Unaligned)]
    #[repr(C)]
    pub struct Status: u8 {
        /// This bit SHALL be set to 0 for all points except the lastpoint of the image.
        const LAST_POINT = 0b10000000;
        /// If this is a 1, then the laser is off (blank). If this is a 0, then thelaser is on
        /// (draw). Note that all systems SHALL write this bit, even if a particular systemuses the
        /// color index for blanking/color information.
        ///
        /// When reading files, the blanking bit takes precedence over the color from the
        /// colorpalette or the points RGB values. If the blanking bit is set, all RGB values
        /// SHOULD betreated as zero.
        const BLANKING = 0b01000000;
    }
}

#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq, AsBytes, FromBytes, Unaligned)]
#[repr(C)]
pub struct Color {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq, AsBytes, FromBytes, Unaligned)]
#[repr(C)]
pub struct Coords3dIndexedColor {
    pub coords: Coords3d,
    pub status: Status,
    pub color_index: u8,
}

#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq, AsBytes, FromBytes, Unaligned)]
#[repr(C)]
pub struct Coords2dIndexedColor {
    pub coords: Coords2d,
    pub status: Status,
    pub color_index: u8,
}

#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq, AsBytes, FromBytes, Unaligned)]
#[repr(C)]
pub struct ColorPalette {
    pub color: Color,
}

#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq, AsBytes, FromBytes, Unaligned)]
#[repr(C)]
pub struct Coords3dTrueColor {
    pub coords: Coords3d,
    pub status: Status,
    pub color: Color,
}

#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq, AsBytes, FromBytes, Unaligned)]
#[repr(C)]
pub struct Coords2dTrueColor {
    pub coords: Coords2d,
    pub status: Status,
    pub color: Color,
}

impl Format {
    pub const COORDS_3D_INDEXED_COLOR: Self = Self(0);
    pub const COORDS_2D_INDEXED_COLOR: Self = Self(1);
    pub const COLOR_PALETTE: Self = Self(2);
    pub const COORDS_3D_TRUE_COLOR: Self = Self(4);
    pub const COORDS_2D_TRUE_COLOR: Self = Self(5);
}

impl Name {
    /// Read the ascii bytes as a UTF8 str.
    pub fn as_str(&self) -> Result<&str, std::str::Utf8Error> {
        let len = self.0.iter().position(|&b| b == 0).unwrap_or(self.0.len());
        std::str::from_utf8(&self.0[..len])
    }
}

impl Header {
    pub const ILDA: [u8; 4] = [0x49, 0x4c, 0x44, 0x41];
}

impl Status {
    pub fn is_blanking(&self) -> bool {
        self.contains(Self::BLANKING)
    }

    pub fn is_last_point(&self) -> bool {
        self.contains(Self::LAST_POINT)
    }
}

impl std::fmt::Debug for Name {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self.as_str() {
            Ok(s) => std::fmt::Debug::fmt(s, f),
            _ => std::fmt::Debug::fmt(&self.0, f),
        }
    }
}

impl std::fmt::Display for Name {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self.as_str() {
            Ok(s) => std::fmt::Display::fmt(s, f),
            _ => std::fmt::Display::fmt("invalid", f),
        }
    }
}
