// Copyright (c) 2021 Intel Corporation
//
// SPDX-License-Identifier: BSD-2-Clause-Patent

/// A macro which defines an enum type.

#[macro_export]
macro_rules! enum_builder {
    (
    $(#[$comment:meta])*
    @U8
        EnumName: $enum_name: ident;
        EnumVal { $( $enum_var: ident => $enum_val: expr ),* }
    ) => {
        $(#[$comment])*
        #[allow(non_camel_case_types)]
        #[derive(Debug, PartialEq, Eq, Clone, Copy)]
        pub enum $enum_name {
            $( $enum_var),*
            ,Unknown(u8)
        }
        impl $enum_name {
            pub fn get_u8(&self) -> u8 {
                let x = self.clone();
                match x {
                    $( $enum_name::$enum_var => $enum_val),*
                    ,$enum_name::Unknown(x) => x
                }
            }
        }
        impl Default for $enum_name {
            fn default() -> $enum_name {
                $enum_name::Unknown(0u8)
            }
        }

        impl From<u8> for $enum_name {
            fn from(value: u8) -> Self {
                match value {
                    $($enum_val => $enum_name::$enum_var,)*
                    x => $enum_name::Unknown(x)
                }
            }
        }

        impl<'a> ctx::TryFromCtx<'a, Endian> for $enum_name {
            type Error = scroll::Error;
            fn try_from_ctx (src: &'a [u8], endian: Endian)
            -> Result<(Self, usize), Self::Error> {
                let offset = &mut 0;
                let value = src.gread_with::<u8>(offset, endian)?;
                match value {
                    $($enum_val => Ok(($enum_name::$enum_var, *offset)),)*
                    x => Ok(($enum_name::Unknown(x), *offset))
                }
            }
        }

        impl ctx::TryIntoCtx<Endian> for $enum_name {
            type Error = scroll::Error;
            fn try_into_ctx(self, this: &mut [u8], le: Endian) -> Result<usize, Self::Error> {
                const TPYE_SIZE: usize = core::mem::size_of::<u8>();
                if this.len() < TPYE_SIZE { return Err(scroll::Error::BadOffset(TPYE_SIZE)); }
                let value = self.get_u8();
                this.pwrite_with(value, 0, le)?;
                Ok(TPYE_SIZE)
            }
        }

        impl ctx::TryIntoCtx<Endian> for &$enum_name {
            type Error = scroll::Error;
            fn try_into_ctx(self, this: &mut [u8], le: Endian) -> Result<usize, Self::Error> {
                const TPYE_SIZE: usize = core::mem::size_of::<u8>();
                if this.len() < TPYE_SIZE { return Err(scroll::Error::BadOffset(TPYE_SIZE)); }
                let value = self.get_u8();
                this.pwrite_with(value, 0, le)?;
                Ok(TPYE_SIZE)
            }
        }
    };
    (
    $(#[$comment:meta])*
    @U16
        EnumName: $enum_name: ident;
        EnumVal { $( $enum_var: ident => $enum_val: expr ),* }
    ) => {
        $(#[$comment])*
        #[allow(non_camel_case_types)]
        #[derive(Debug, PartialEq, Eq, Clone, Copy)]
        pub enum $enum_name {
            $( $enum_var),*
            ,Unknown(u16)
        }
        impl $enum_name {
            pub fn get_u16(&self) -> u16 {
                let x = self.clone();
                match x {
                    $( $enum_name::$enum_var => $enum_val),*
                    ,$enum_name::Unknown(x) => x
                }
            }
        }
        impl Default for $enum_name {
            fn default() -> $enum_name {
                $enum_name::Unknown(0u16)
            }
        }

        impl From<u16> for $enum_name {
            fn from(value: u16) -> Self {
                match value {
                    $($enum_val => $enum_name::$enum_var,)*
                    x => $enum_name::Unknown(x)
                }
            }
        }

        impl<'a> ctx::TryFromCtx<'a, Endian> for $enum_name {
            type Error = scroll::Error;
            fn try_from_ctx (src: &'a [u8], endian: Endian)
            -> Result<(Self, usize), Self::Error> {
                let offset = &mut 0;
                let value = src.gread_with::<u16>(offset, endian)?;
                match value {
                    $($enum_val => Ok(($enum_name::$enum_var, *offset)),)*
                    x => Ok(($enum_name::Unknown(x), *offset))
                }
            }
        }

        impl ctx::TryIntoCtx<Endian> for $enum_name {
            type Error = scroll::Error;
            fn try_into_ctx(self, this: &mut [u8], le: Endian) -> Result<usize, Self::Error> {
                const TPYE_SIZE: usize = core::mem::size_of::<u16>();
                if this.len() < TPYE_SIZE { return Err(scroll::Error::BadOffset(TPYE_SIZE)); }
                let value = self.get_u16();
                this.pwrite_with(value, 0, le)?;
                Ok(TPYE_SIZE)
            }
        }

        impl ctx::TryIntoCtx<Endian> for &$enum_name {
            type Error = scroll::Error;
            fn try_into_ctx(self, this: &mut [u8], le: Endian) -> Result<usize, Self::Error> {
                const TPYE_SIZE: usize = core::mem::size_of::<u16>();
                if this.len() < TPYE_SIZE { return Err(scroll::Error::BadOffset(TPYE_SIZE)); }
                let value = self.get_u16();
                this.pwrite_with(value, 0, le)?;
                Ok(TPYE_SIZE)
            }
        }
    };
    (
    $(#[$comment:meta])*
    @U32
        EnumName: $enum_name: ident;
        EnumVal { $( $enum_var: ident => $enum_val: expr ),* }
    ) => {
        $(#[$comment])*
        #[derive(Debug, PartialEq, Eq, Clone, Copy)]
        #[allow(non_camel_case_types)]
        pub enum $enum_name {
            $( $enum_var),*
            ,Unknown(u32)
        }
        impl $enum_name {
            pub fn get_u32(&self) -> u32 {
                let x = self.clone();
                match x {
                    $( $enum_name::$enum_var => $enum_val),*
                    ,$enum_name::Unknown(x) => x
                }
            }
        }
        impl Default for $enum_name {
            fn default() -> $enum_name {
                $enum_name::Unknown(0u32)
            }
        }

        impl From<u32> for $enum_name {
            fn from(value: u32) -> Self {
                match value {
                    $($enum_val => $enum_name::$enum_var,)*
                    x => $enum_name::Unknown(x)
                }
            }
        }

        impl<'a> ctx::TryFromCtx<'a, Endian> for $enum_name {
            type Error = scroll::Error;
            fn try_from_ctx (src: &'a [u8], endian: Endian)
            -> Result<(Self, usize), Self::Error> {
                let offset = &mut 0;
                let value = src.gread_with::<u32>(offset, endian)?;
                match value {
                    $($enum_val => Ok(($enum_name::$enum_var, *offset)),)*
                    x => Ok(($enum_name::Unknown(x), *offset))
                }
            }
        }

        impl ctx::TryIntoCtx<Endian> for $enum_name {
            type Error = scroll::Error;
            fn try_into_ctx(self, this: &mut [u8], le: Endian) -> Result<usize, Self::Error> {
                const TPYE_SIZE: usize = core::mem::size_of::<u32>();
                if this.len() < TPYE_SIZE { return Err(scroll::Error::BadOffset(TPYE_SIZE)); }
                let value = self.get_u32();
                this.pwrite_with(value, 0, le)?;
                Ok(TPYE_SIZE)
            }
        }

        impl ctx::TryIntoCtx<Endian> for &$enum_name {
            type Error = scroll::Error;
            fn try_into_ctx(self, this: &mut [u8], le: Endian) -> Result<usize, Self::Error> {
                const TPYE_SIZE: usize = core::mem::size_of::<u32>();
                if this.len() < TPYE_SIZE { return Err(scroll::Error::BadOffset(TPYE_SIZE)); }
                let value = self.get_u32();
                this.pwrite_with(value, 0, le)?;
                Ok(TPYE_SIZE)
            }
        }
    };
}

#[cfg(test)]
mod test {

    use scroll::Pwrite;
    use scroll::{ctx, Endian, Pread};

    #[test]
    fn test_emun32() {
        enum_builder! {
            @U32
            EnumName: TestEnum;
            EnumVal{
                Value1 => 0x1,
                Value2 => 0x2
            }
        }

        let mut bytes = [0u8; 8];
        bytes.pwrite(TestEnum::Value2, 0).unwrap();
        bytes.pwrite(TestEnum::Value1, 4).unwrap();
        assert_eq!(bytes.pwrite(TestEnum::Value1, 7).is_err(), true);

        assert_eq!(bytes.pread::<TestEnum>(0).unwrap(), TestEnum::Value2);
        assert_eq!(bytes.pread::<TestEnum>(4).unwrap(), TestEnum::Value1);
    }
}
