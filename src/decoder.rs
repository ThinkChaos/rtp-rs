use byteorder::{BigEndian, ReadBytesExt};
use std::convert::{Infallible, TryFrom};
use std::error::Error;
use std::fmt::{self, Display, Formatter};

use crate::header::{Header, CSRC, SSRC};
use crate::version::{Version, VersionError};

pub fn decode<TBuffer>(buffer: TBuffer) -> Result<Header, DecodeError>
where
    TBuffer: AsRef<[u8]>,
{
    let mut buffer = buffer.as_ref();

    let byte = buffer.read_u8().map_err(|_| DecodeError::UnexpectedEOF)?;
    let version = match Version::try_from(byte & 0x3)? {
        Version::RTP2 => Version::RTP2,
        _ => return Err(DecodeError::UnsupportedVersion),
    };
    let has_padding = byte & 0x4 > 0;
    let has_extension = byte & 0x8 > 0;
    let csrc_count = byte >> 4;
    let byte = buffer.read_u8().map_err(|_| DecodeError::UnexpectedEOF)?;
    let has_marker = byte & 0x1 > 0;
    let payload_type = byte >> 1;
    let sequence_number = buffer
        .read_u16::<BigEndian>()
        .map_err(|_| DecodeError::UnexpectedEOF)?;
    let timestamp = buffer
        .read_u32::<BigEndian>()
        .map_err(|_| DecodeError::UnexpectedEOF)?;
    let ssrc = SSRC(
        buffer
            .read_u32::<BigEndian>()
            .map_err(|_| DecodeError::UnexpectedEOF)?,
    );
    let mut csrcs = Vec::with_capacity(csrc_count as usize);

    for _ in 0..csrc_count {
        let csrc = buffer
            .read_u32::<BigEndian>()
            .map_err(|_| DecodeError::UnexpectedEOF)?;
        csrcs.push(CSRC(csrc));
    }

    Ok(Header {
        csrcs,
        has_extension,
        has_marker,
        has_padding,
        payload_type,
        sequence_number,
        ssrc,
        timestamp,
        version,
    })
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DecodeError {
    Version(VersionError),
    UnexpectedEOF,
    UnsupportedVersion,
}

impl Display for DecodeError {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        use self::DecodeError::*;

        match self {
            Version(error) => error.fmt(formatter),
            UnexpectedEOF => write!(formatter, "unexpected EOF"),
            UnsupportedVersion => write!(formatter, "unsupported version"),
        }
    }
}

impl Error for DecodeError {}

impl From<Infallible> for DecodeError {
    fn from(_: Infallible) -> Self {
        DecodeError::UnsupportedVersion
    }
}

impl From<VersionError> for DecodeError {
    fn from(value: VersionError) -> Self {
        DecodeError::Version(value)
    }
}
