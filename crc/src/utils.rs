use feature_macros::numbers::*;

#[allow(dead_code)]
use feature_macros::unstd::ops::*;

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

    // CRC Params
    const INITIAL: Prim;
    const REFOUT: bool;
    const XOROUT: Prim;

    const TABLE: TableKind;
    const NORMAL_ACC_SHR: Prim;
    const NORMAL_ACC_SHL: Prim;

    fn process(bytes: &[u8]) -> Prim {

        // setup the initial accumulator value
	let init: Prim = if Self::REFOUT {
		Self::INITIAL.not()
	} else {
		Self::INITIAL
	};

        // run the algorithm
        let output: Prim = bytes.iter().fold(init, |accum: Prim, item: &u8| -> Prim {
            accum.shl(Self::NORMAL_ACC_SHL).bitxor(Self::index_table(
                Self::deref_to_prim(item).bitxor(accum.shr(Self::NORMAL_ACC_SHR)),
            ))
        });

	// we may need to reflect the value _again_
        let pre_xor: Prim = if Self::REFOUT {
		output.not()
	} else {
		output
	};

	// final xor
	pre_xor.xor(Self::XOROUT)
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
