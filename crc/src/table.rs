
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
    ($poly: expr, $reflect: expr, $table: expr, $count:expr) => {
        {

            // handle reflections
            let pick: [u8;2] = [$count as u8, reflect_8!($count as u8)];
            let byte: u16 = pick[$reflect as usize] as u16;
            let value: u16 = byte << 8;


            let value = poly_sum_16!($poly, poly_sum_16!($poly, poly_sum_16!($poly, poly_sum_16!($poly, poly_sum_16!($poly, poly_sum_16!($poly, poly_sum_16!($poly, poly_sum_16!($poly, value))))))));

            // handle reflections again
            let pick_again: [u8;2] = [value as u8, reflect_8!(value)];
            let value: u8 = pick_again[$reflect as usize];

            // shove the value into the table
            $table[$count] = value;
        }
    }
}
