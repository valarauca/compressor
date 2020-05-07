use feature_macros::numbers::*;

#[allow(dead_code)]
use feature_macros::unstd::ops::*;

/// What type of calculation are we performing
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Calc {
    Normal,
    Reverse,
    Compat,
}
impl Default for Calc {
    #[inline(always)]
    fn default() -> Self {
        Self::Normal
    }
}

/// Trivial type which acts like a table
pub trait CRCTable<Prim: BasicNumber>: Index<u8, Output = Prim> {}

/// Fundamental operation of CRC.
///
/// This trait is extremely ugly, but it means we can implement
/// every kind of CRC16, 32, and 64 by just specifying constants.
pub trait CRC<Prim: BasicNumber, TableKind: CRCTable<Prim>>
where
    Num<Prim>: From<u8>,
    Num<Prim>: PrimativeNumber<Primative = Prim>,
    Num<u8>: From<Prim>,
    Prim: Shl<Prim, Output = Prim>,
    Prim: Shr<Prim, Output = Prim>,
    Prim: BitXor<Prim, Output = Prim>,
{
    const TABLE: TableKind;
    const NORMAL_ACC_SHR: Prim;
    const NORMAL_ACC_SHL: Prim;
    const REVERSE_ACC_SHR: Prim;
    const COMPAT_ACC_SHR: Prim;

    fn update(value: Prim, bytes: &[u8], calc: &Calc) -> Prim {
        match calc {
            &Calc::Normal => bytes.iter().fold(value, |accum: Prim, item: &u8| -> Prim {
                accum.shl(Self::NORMAL_ACC_SHL).bitxor(Self::index_table(
                    Self::deref_to_prim(item).bitxor(accum.shr(Self::NORMAL_ACC_SHR)),
                ))
            }),
            &Calc::Reverse => bytes.iter().fold(value, |accum: Prim, item: &u8| -> Prim {
                accum
                    .shr(Self::REVERSE_ACC_SHR)
                    .bitxor(Self::index_table(accum.bitxor(Self::deref_to_prim(item))))
            }),
            &Calc::Compat => bytes
                .iter()
                .fold(value.not(), |accum: Prim, item: &u8| -> Prim {
                    accum
                        .shr(Self::COMPAT_ACC_SHR)
                        .bitxor(Self::index_table(accum.bitxor(Self::deref_to_prim(item))))
                })
                .not(),
        }
    }

    #[inline(always)]
    fn index_table(index: Prim) -> Prim {
        Self::TABLE.index(Num::<u8>::from(index).inner()).clone()
    }

    #[inline(always)]
    fn deref_to_prim(arg: &u8) -> Prim {
        // this an `as` operation
        // but hidding behind generics
        Num::<Prim>::from(*arg).inner()
    }
}
