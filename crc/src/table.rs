#![allow(unused_macro)]
#[allow(unused_imports)]
use ::feature_macros::numbers::*;
use ::feature_macros::unstd::ops::*;

macro_rules! poly_sum_16 {
    ($poly: expr, $value: expr) => {
        {
            let value: u16 = $value as u16;
            (value << 1)^((value >> 15) * $poly)
        }
    }
}
macro_rules! poly_sum_32 {
    ($poly: expr, $value: expr) => {
        {
            let value: u32 = $value as u64;
            (value << 1)^((value >> 31) * $poly)
        }
    }
}
macro_rules! poly_sum_64 {
    ($poly: expr, $value: expr) => {
        {
            let value: u64 = $value as u64;
            (value << 1)^((value >> 61) * $poly)
        }
    }
}

macro_rules! reflect_8 {
    ($arg: expr) => {
        {
            let arg = $arg as u8;
            let a:u8 = (arg & 0xF0u8) >> 4 | (arg & 0x0Fu8) << 4;
            let b:u8 = (a & 0xCCu8) >> 2 | (a & 0x33u8) << 2;
            (b & 0xAAu8) >> 1 | (b & 0x55u8) << 1
        }
    };
}

macro_rules! reflect_16 {
    ($arg: expr) => {
        {
            let arg: u16 = $arg as u16;
            let a: u16 = (arg & 0xFF00u16) >> 8 | (arg & 0x00FFu16) << 8;
            let b: u16 = (a & 0xF0F0u16) >> 4 | (a & 0x0F0Fu16) << 4;
            let c: u16 = (b & 0xCCCCu16) >> 2 | (b & 0x3333u16) << 2;
            (c & 0xAAAAu16) >> 1 | (c & 0x5555u16) << 1
        }
    };
}

macro_rules! reflect_32 {
    ($arg: expr) => {
        {
            let arg: u32 = $arg as u32;
            let a: u32 = (arg & 0xFFFF0000u32) >> 16 | (arg & 0x0000FFFFu32) << 16;
            let b: u32 = (a & 0xFF00FF00u32) >> 8  | (a & 0x00FF00FFu32) << 8;
            let c: u32 = (b & 0xF0F0F0F0u32) >> 4  | (b & 0x0F0F0F0Fu32) << 4;
            let d: u32 = (c & 0xCCCCCCCCu32) >> 2  | (c & 0x33333333u32) << 2;
            (d & 0xAAAAAAAAu32) >> 1  | (d & 0x55555555u32) << 1
        }
    };
}

macro_rules! reflect_64 {
    ($arg: expr) => {
        {
            let arg: u64 = arg as u64;
            let a: u64 = (arg & 0xFFFFFFFF00000000u64) >> 32 | (arg & 0x00000000FFFFFFFFu64) << 32;
            let b: u64 = (a & 0xFFFF0000FFFF0000u64) >> 16 | (a & 0x0000FFFF0000FFFFu64) << 16;
            let c: u64 = (b & 0xFF00FF00FF00FF00u64) >> 8  | (b & 0x00FF00FF00FF00FFu64) << 8;
            let d: u64 = (c & 0xF0F0F0F0F0F0F0F0u64) >> 4  | (c & 0x0F0F0F0F0F0F0F0Fu64) << 4;
            let e: u64 = (d & 0xCCCCCCCCCCCCCCCCu64) >> 2  | (d & 0x3333333333333333u64) << 2;
            (e & 0xAAAAAAAAAAAAAAAAu64) >> 1  | (e & 0x5555555555555555u64) << 1
        }
    };
}

macro_rules! build_table_16_index {

    ($poly: expr, $reflect: expr, $count:expr) => {
        {
            // handle reflections
            let pick: [u8;2] = [$count as u8, reflect_8!($count as u8)];
            let byte: u16 = pick[$reflect as usize] as u16;
            let value: u16 = byte << 8;


            let value = poly_sum_16!($poly, poly_sum_16!($poly, poly_sum_16!($poly, poly_sum_16!($poly, poly_sum_16!($poly, poly_sum_16!($poly, poly_sum_16!($poly, poly_sum_16!($poly, value))))))));

            // handle reflections again
            let pick_again: [u16;2] = [value, reflect_16!(value)];
            let value: u16 = pick_again[$reflect as usize];

            // shove the value into the table
		value
        }
    };
}

#[derive(Clone,Copy)]
pub struct CRCTableConcrete<T: BasicNumber + 'static> {
	data: &'static [T;256],
}
impl<T: BasicNumber> Index<u8> for CRCTableConcrete<T> {
	type Output = T;
	#[inline(always)]
	fn index<'a>(&'a self, index: u8) -> &'a T {
		self.data.index(index as usize)
	}
}
impl<T: BasicNumber> super::utils::CRCTable<T> for CRCTableConcrete<T> { }

macro_rules! build_table{
	(@16
		TypeName: $TABLE_NAME: ident;
		ExportName: $EXPORT: ident;
		InternalTableName: $CONST_TABLE_NAME: ident;
		Poly: $poly:expr;
		Initial: $INITIAL: 
		Refin: $reflect: expr;
	) => {
		const $CONST_TABLE_NAME: &'static [u16;256] = &[
			build_table_16_index!($poly, $reflect, 0),
			build_table_16_index!($poly, $reflect, 1),
			build_table_16_index!($poly, $reflect, 2),
			build_table_16_index!($poly, $reflect, 3),
			build_table_16_index!($poly, $reflect, 4),
			build_table_16_index!($poly, $reflect, 5),
			build_table_16_index!($poly, $reflect, 6),
			build_table_16_index!($poly, $reflect, 7),
			build_table_16_index!($poly, $reflect, 8),
			build_table_16_index!($poly, $reflect, 9),
			build_table_16_index!($poly, $reflect, 10),
			build_table_16_index!($poly, $reflect, 11),
			build_table_16_index!($poly, $reflect, 12),
			build_table_16_index!($poly, $reflect, 13),
			build_table_16_index!($poly, $reflect, 14),
			build_table_16_index!($poly, $reflect, 15),
			build_table_16_index!($poly, $reflect, 16),
			build_table_16_index!($poly, $reflect, 17),
			build_table_16_index!($poly, $reflect, 18),
			build_table_16_index!($poly, $reflect, 19),
			build_table_16_index!($poly, $reflect, 20),
			build_table_16_index!($poly, $reflect, 21),
			build_table_16_index!($poly, $reflect, 22),
			build_table_16_index!($poly, $reflect, 23),
			build_table_16_index!($poly, $reflect, 24),
			build_table_16_index!($poly, $reflect, 25),
			build_table_16_index!($poly, $reflect, 26),
			build_table_16_index!($poly, $reflect, 27),
			build_table_16_index!($poly, $reflect, 28),
			build_table_16_index!($poly, $reflect, 29),
			build_table_16_index!($poly, $reflect, 30),
			build_table_16_index!($poly, $reflect, 31),
			build_table_16_index!($poly, $reflect, 32),
			build_table_16_index!($poly, $reflect, 33),
			build_table_16_index!($poly, $reflect, 34),
			build_table_16_index!($poly, $reflect, 35),
			build_table_16_index!($poly, $reflect, 36),
			build_table_16_index!($poly, $reflect, 37),
			build_table_16_index!($poly, $reflect, 38),
			build_table_16_index!($poly, $reflect, 39),
			build_table_16_index!($poly, $reflect, 40),
			build_table_16_index!($poly, $reflect, 41),
			build_table_16_index!($poly, $reflect, 42),
			build_table_16_index!($poly, $reflect, 43),
			build_table_16_index!($poly, $reflect, 44),
			build_table_16_index!($poly, $reflect, 45),
			build_table_16_index!($poly, $reflect, 46),
			build_table_16_index!($poly, $reflect, 47),
			build_table_16_index!($poly, $reflect, 48),
			build_table_16_index!($poly, $reflect, 49),
			build_table_16_index!($poly, $reflect, 50),
			build_table_16_index!($poly, $reflect, 51),
			build_table_16_index!($poly, $reflect, 52),
			build_table_16_index!($poly, $reflect, 53),
			build_table_16_index!($poly, $reflect, 54),
			build_table_16_index!($poly, $reflect, 55),
			build_table_16_index!($poly, $reflect, 56),
			build_table_16_index!($poly, $reflect, 57),
			build_table_16_index!($poly, $reflect, 58),
			build_table_16_index!($poly, $reflect, 59),
			build_table_16_index!($poly, $reflect, 60),
			build_table_16_index!($poly, $reflect, 61),
			build_table_16_index!($poly, $reflect, 62),
			build_table_16_index!($poly, $reflect, 63),
			build_table_16_index!($poly, $reflect, 64),
			build_table_16_index!($poly, $reflect, 65),
			build_table_16_index!($poly, $reflect, 66),
			build_table_16_index!($poly, $reflect, 67),
			build_table_16_index!($poly, $reflect, 68),
			build_table_16_index!($poly, $reflect, 69),
			build_table_16_index!($poly, $reflect, 70),
			build_table_16_index!($poly, $reflect, 71),
			build_table_16_index!($poly, $reflect, 72),
			build_table_16_index!($poly, $reflect, 73),
			build_table_16_index!($poly, $reflect, 74),
			build_table_16_index!($poly, $reflect, 75),
			build_table_16_index!($poly, $reflect, 76),
			build_table_16_index!($poly, $reflect, 77),
			build_table_16_index!($poly, $reflect, 78),
			build_table_16_index!($poly, $reflect, 79),
			build_table_16_index!($poly, $reflect, 80),
			build_table_16_index!($poly, $reflect, 81),
			build_table_16_index!($poly, $reflect, 82),
			build_table_16_index!($poly, $reflect, 83),
			build_table_16_index!($poly, $reflect, 84),
			build_table_16_index!($poly, $reflect, 85),
			build_table_16_index!($poly, $reflect, 86),
			build_table_16_index!($poly, $reflect, 87),
			build_table_16_index!($poly, $reflect, 88),
			build_table_16_index!($poly, $reflect, 89),
			build_table_16_index!($poly, $reflect, 90),
			build_table_16_index!($poly, $reflect, 91),
			build_table_16_index!($poly, $reflect, 92),
			build_table_16_index!($poly, $reflect, 93),
			build_table_16_index!($poly, $reflect, 94),
			build_table_16_index!($poly, $reflect, 95),
			build_table_16_index!($poly, $reflect, 96),
			build_table_16_index!($poly, $reflect, 97),
			build_table_16_index!($poly, $reflect, 98),
			build_table_16_index!($poly, $reflect, 99),
			build_table_16_index!($poly, $reflect, 100),
			build_table_16_index!($poly, $reflect, 101),
			build_table_16_index!($poly, $reflect, 102),
			build_table_16_index!($poly, $reflect, 103),
			build_table_16_index!($poly, $reflect, 104),
			build_table_16_index!($poly, $reflect, 105),
			build_table_16_index!($poly, $reflect, 106),
			build_table_16_index!($poly, $reflect, 107),
			build_table_16_index!($poly, $reflect, 108),
			build_table_16_index!($poly, $reflect, 109),
			build_table_16_index!($poly, $reflect, 110),
			build_table_16_index!($poly, $reflect, 111),
			build_table_16_index!($poly, $reflect, 112),
			build_table_16_index!($poly, $reflect, 113),
			build_table_16_index!($poly, $reflect, 114),
			build_table_16_index!($poly, $reflect, 115),
			build_table_16_index!($poly, $reflect, 116),
			build_table_16_index!($poly, $reflect, 117),
			build_table_16_index!($poly, $reflect, 118),
			build_table_16_index!($poly, $reflect, 119),
			build_table_16_index!($poly, $reflect, 120),
			build_table_16_index!($poly, $reflect, 121),
			build_table_16_index!($poly, $reflect, 122),
			build_table_16_index!($poly, $reflect, 123),
			build_table_16_index!($poly, $reflect, 124),
			build_table_16_index!($poly, $reflect, 125),
			build_table_16_index!($poly, $reflect, 126),
			build_table_16_index!($poly, $reflect, 127),
			build_table_16_index!($poly, $reflect, 128),
			build_table_16_index!($poly, $reflect, 129),
			build_table_16_index!($poly, $reflect, 130),
			build_table_16_index!($poly, $reflect, 131),
			build_table_16_index!($poly, $reflect, 132),
			build_table_16_index!($poly, $reflect, 133),
			build_table_16_index!($poly, $reflect, 134),
			build_table_16_index!($poly, $reflect, 135),
			build_table_16_index!($poly, $reflect, 136),
			build_table_16_index!($poly, $reflect, 137),
			build_table_16_index!($poly, $reflect, 138),
			build_table_16_index!($poly, $reflect, 139),
			build_table_16_index!($poly, $reflect, 140),
			build_table_16_index!($poly, $reflect, 141),
			build_table_16_index!($poly, $reflect, 142),
			build_table_16_index!($poly, $reflect, 143),
			build_table_16_index!($poly, $reflect, 144),
			build_table_16_index!($poly, $reflect, 145),
			build_table_16_index!($poly, $reflect, 146),
			build_table_16_index!($poly, $reflect, 147),
			build_table_16_index!($poly, $reflect, 148),
			build_table_16_index!($poly, $reflect, 149),
			build_table_16_index!($poly, $reflect, 150),
			build_table_16_index!($poly, $reflect, 151),
			build_table_16_index!($poly, $reflect, 152),
			build_table_16_index!($poly, $reflect, 153),
			build_table_16_index!($poly, $reflect, 154),
			build_table_16_index!($poly, $reflect, 155),
			build_table_16_index!($poly, $reflect, 156),
			build_table_16_index!($poly, $reflect, 157),
			build_table_16_index!($poly, $reflect, 158),
			build_table_16_index!($poly, $reflect, 159),
			build_table_16_index!($poly, $reflect, 160),
			build_table_16_index!($poly, $reflect, 161),
			build_table_16_index!($poly, $reflect, 162),
			build_table_16_index!($poly, $reflect, 163),
			build_table_16_index!($poly, $reflect, 164),
			build_table_16_index!($poly, $reflect, 165),
			build_table_16_index!($poly, $reflect, 166),
			build_table_16_index!($poly, $reflect, 167),
			build_table_16_index!($poly, $reflect, 168),
			build_table_16_index!($poly, $reflect, 169),
			build_table_16_index!($poly, $reflect, 170),
			build_table_16_index!($poly, $reflect, 171),
			build_table_16_index!($poly, $reflect, 172),
			build_table_16_index!($poly, $reflect, 173),
			build_table_16_index!($poly, $reflect, 174),
			build_table_16_index!($poly, $reflect, 175),
			build_table_16_index!($poly, $reflect, 176),
			build_table_16_index!($poly, $reflect, 177),
			build_table_16_index!($poly, $reflect, 178),
			build_table_16_index!($poly, $reflect, 179),
			build_table_16_index!($poly, $reflect, 180),
			build_table_16_index!($poly, $reflect, 181),
			build_table_16_index!($poly, $reflect, 182),
			build_table_16_index!($poly, $reflect, 183),
			build_table_16_index!($poly, $reflect, 184),
			build_table_16_index!($poly, $reflect, 185),
			build_table_16_index!($poly, $reflect, 186),
			build_table_16_index!($poly, $reflect, 187),
			build_table_16_index!($poly, $reflect, 188),
			build_table_16_index!($poly, $reflect, 189),
			build_table_16_index!($poly, $reflect, 190),
			build_table_16_index!($poly, $reflect, 191),
			build_table_16_index!($poly, $reflect, 192),
			build_table_16_index!($poly, $reflect, 193),
			build_table_16_index!($poly, $reflect, 194),
			build_table_16_index!($poly, $reflect, 195),
			build_table_16_index!($poly, $reflect, 196),
			build_table_16_index!($poly, $reflect, 197),
			build_table_16_index!($poly, $reflect, 198),
			build_table_16_index!($poly, $reflect, 199),
			build_table_16_index!($poly, $reflect, 200),
			build_table_16_index!($poly, $reflect, 201),
			build_table_16_index!($poly, $reflect, 202),
			build_table_16_index!($poly, $reflect, 203),
			build_table_16_index!($poly, $reflect, 204),
			build_table_16_index!($poly, $reflect, 205),
			build_table_16_index!($poly, $reflect, 206),
			build_table_16_index!($poly, $reflect, 207),
			build_table_16_index!($poly, $reflect, 208),
			build_table_16_index!($poly, $reflect, 209),
			build_table_16_index!($poly, $reflect, 210),
			build_table_16_index!($poly, $reflect, 211),
			build_table_16_index!($poly, $reflect, 212),
			build_table_16_index!($poly, $reflect, 213),
			build_table_16_index!($poly, $reflect, 214),
			build_table_16_index!($poly, $reflect, 215),
			build_table_16_index!($poly, $reflect, 216),
			build_table_16_index!($poly, $reflect, 217),
			build_table_16_index!($poly, $reflect, 218),
			build_table_16_index!($poly, $reflect, 219),
			build_table_16_index!($poly, $reflect, 220),
			build_table_16_index!($poly, $reflect, 221),
			build_table_16_index!($poly, $reflect, 222),
			build_table_16_index!($poly, $reflect, 223),
			build_table_16_index!($poly, $reflect, 224),
			build_table_16_index!($poly, $reflect, 225),
			build_table_16_index!($poly, $reflect, 226),
			build_table_16_index!($poly, $reflect, 227),
			build_table_16_index!($poly, $reflect, 228),
			build_table_16_index!($poly, $reflect, 229),
			build_table_16_index!($poly, $reflect, 230),
			build_table_16_index!($poly, $reflect, 231),
			build_table_16_index!($poly, $reflect, 232),
			build_table_16_index!($poly, $reflect, 233),
			build_table_16_index!($poly, $reflect, 234),
			build_table_16_index!($poly, $reflect, 235),
			build_table_16_index!($poly, $reflect, 236),
			build_table_16_index!($poly, $reflect, 237),
			build_table_16_index!($poly, $reflect, 238),
			build_table_16_index!($poly, $reflect, 239),
			build_table_16_index!($poly, $reflect, 240),
			build_table_16_index!($poly, $reflect, 241),
			build_table_16_index!($poly, $reflect, 242),
			build_table_16_index!($poly, $reflect, 243),
			build_table_16_index!($poly, $reflect, 244),
			build_table_16_index!($poly, $reflect, 245),
			build_table_16_index!($poly, $reflect, 246),
			build_table_16_index!($poly, $reflect, 247),
			build_table_16_index!($poly, $reflect, 248),
			build_table_16_index!($poly, $reflect, 249),
			build_table_16_index!($poly, $reflect, 250),
			build_table_16_index!($poly, $reflect, 251),
			build_table_16_index!($poly, $reflect, 252),
			build_table_16_index!($poly, $reflect, 253),
			build_table_16_index!($poly, $reflect, 254),
			build_table_16_index!($poly, $reflect, 255),
		];
		pub const $TABLE_CONST: CRCTableConcrete<u16> = CRCTableConcrete {
			data: $CONST_TABLE_NAME,
		};

		#[derive(Clone,Copy,Default)]
                pub struct $TYPE_NAME {
			#[allow(dead_code)] _field: u8,
                };
		impl ::super::utils::CRC<u16, CRCTableConcrete<u16>> for $TYPE_NAME {
			const INITIAL: u16 = $INITIAL;
			const REFOUT: bool = $REFLECT_OUT;
			const XOROUT: bool = $XOR_OUT;

			const TABLE: CRCTableConcrete = $TABLE_CONST;
			const NORMAL_ACC_SHR: u16 = 24;
			const NORMAL_ACC_SHL: u16 = 24;
		}
	}
}

build_table! {@16
        TypeName: CRC_16_ARC;
	TableName: CRC_16_TABLE_ARC;
	InternalTableName: _CRC_16_ARC_TABLE;
	Poly: 0x8005u16;
	Initial: 0x0000u16;
	Reflect_In: true;
	Reflect_Out: true;
	Xor_Out: 0x000u16;
}
/*
build_table! {@16
	ExportName: CRC_16_CDMA2000;
	InternalTableName: _CRC_16_CDMA2000;
	Poly: 0xc867u16;
	Refin: false;
}
build_table! {@16
	ExportName: CRC_16_CMS;
	InternalTableName: _CRC_16_CMS;
	Poly: 0x8005;
	Refin: false;
}
build_table! {@16
	ExportName: CRC_16_DDS_110;
	InternalTableName: _CRC_16_DDS_110;
	Poly: 0x8005;
	Refin: false;
}
*/



