use core::convert::{From, Into};
use core::fmt;
use r_efi::efi;
use scroll::{ctx, Endian, Pread, Pwrite};

///
/// Wrapper r_efi::efi::Guid
///
/// 128 bit buffer containing a unique identifier value.
/// Unless otherwise specified, aligned on a 64 bit boundary.
///
#[repr(C)]
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Guid(efi::Guid);

impl Guid {
    /// Initialize a Guid from its individual fields
    pub const fn from_fields(
        time_low: u32,
        time_mid: u16,
        time_hi_and_version: u16,
        clk_seq_hi_res: u8,
        clk_seq_low: u8,
        node: &[u8; 6],
    ) -> Guid {
        Guid(efi::Guid::from_fields(
            time_low,
            time_mid,
            time_hi_and_version,
            clk_seq_hi_res,
            clk_seq_low,
            node,
        ))
    }

    /// Access a Guid as individual fields
    pub const fn as_fields(&self) -> (u32, u16, u16, u8, u8, &[u8; 6]) {
        self.0.as_fields()
    }

    /// Access a Guid as raw byte array
    pub fn as_bytes(&self) -> &[u8; 16] {
        self.0.as_bytes()
    }
}

impl ctx::TryIntoCtx<Endian> for Guid {
    type Error = scroll::Error;
    fn try_into_ctx(self, this: &mut [u8], le: Endian) -> Result<usize, Self::Error> {
        const TPYE_SIZE: usize = core::mem::size_of::<efi::Guid>();
        if this.len() < TPYE_SIZE {
            return Err(scroll::Error::BadOffset(TPYE_SIZE));
        }
        let value = self.0.as_bytes();
        let offset = &mut 0;
        for v in value {
            this.gwrite_with::<u8>(*v, offset, le)?;
        }
        Ok(TPYE_SIZE)
    }
}

impl ctx::TryIntoCtx<Endian> for &Guid {
    type Error = scroll::Error;
    fn try_into_ctx(self, this: &mut [u8], le: Endian) -> Result<usize, Self::Error> {
        const TPYE_SIZE: usize = core::mem::size_of::<efi::Guid>();
        if this.len() < TPYE_SIZE {
            return Err(scroll::Error::BadOffset(TPYE_SIZE));
        }
        let value = self.as_bytes();
        let offset = &mut 0;
        for v in value {
            this.gwrite_with::<u8>(*v, offset, le)?;
        }
        Ok(TPYE_SIZE)
    }
}

impl<'a> ctx::TryFromCtx<'a, Endian> for Guid {
    type Error = scroll::Error;
    fn try_from_ctx(src: &'a [u8], endian: Endian) -> Result<(Self, usize), Self::Error> {
        let offset = &mut 0;
        let time_low = src.gread_with::<u32>(offset, endian)?;
        let time_mid = src.gread_with::<u16>(offset, endian)?;
        let time_hi_and_version = src.gread_with::<u16>(offset, endian)?;
        let clk_seq_hi_res = src.gread_with::<u8>(offset, endian)?;
        let clk_seq_low = src.gread_with::<u8>(offset, endian)?;
        let mut node = [0u8; 6];
        src.gread_inout_with(offset, &mut node, endian)?;
        let efi_guid = Guid::from_fields(
            time_low,
            time_mid,
            time_hi_and_version,
            clk_seq_hi_res,
            clk_seq_low,
            &node,
        );
        Ok((efi_guid, *offset))
    }
}

impl fmt::Debug for Guid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let fields = self.as_fields();
        f.write_fmt(format_args!(
            "{:08x}-{:04x}-{:04x}-{:02x}{:02x}-",
            fields.0, fields.1, fields.2, fields.3, fields.4
        ))?;
        for v in fields.5 {
            write!(f, "{:02x}", *v)?;
        }
        Ok(())
    }
}

impl From<efi::Guid> for Guid {
    fn from(guid: efi::Guid) -> Self {
        Guid(guid)
    }
}

impl Into<efi::Guid> for Guid {
    fn into(self) -> efi::Guid {
        self.0
    }
}

impl PartialEq<efi::Guid> for Guid {
    fn eq(&self, other: &efi::Guid) -> bool {
        &self.0 == other
    }
}

impl PartialEq<Guid> for efi::Guid {
    fn eq(&self, other: &Guid) -> bool {
        self == &other.0
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn test_guid() {
        use scroll::{Pread, Pwrite};
        let guid = super::Guid::from_fields(
            0x7739F24C,
            0x93D7,
            0x11D4,
            0x9A,
            0x3A,
            &[0x00, 0x90, 0x27, 0x3F, 0xC1, 0x4D],
        );
        println!("Guid: {:?}", guid);
        let mut guid_bytes = [0u8; 16];
        guid_bytes.pwrite(guid, 0).unwrap();
        assert_eq!(&guid_bytes, guid.as_bytes());

        let guid2 = guid_bytes.pread::<super::Guid>(0).unwrap();
        assert_eq!(guid, guid2);
    }

    #[test]
    fn test_guid_from_to() {
        use super::Guid;
        use r_efi::efi;
        let r_guid = efi::Guid::from_fields(
            0x7739F24C,
            0x93D7,
            0x11D4,
            0x9A,
            0x3A,
            &[0x00, 0x90, 0x27, 0x3F, 0xC1, 0x4D],
        );
        let guid = Guid::from(r_guid);
        let r_guid2: efi::Guid = guid.into();

        assert_eq!(guid, r_guid2);
        assert_eq!(r_guid2, guid);
        assert!(r_guid2 == guid);
        assert!(!(guid != r_guid2));
    }
}
