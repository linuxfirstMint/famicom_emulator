fn main() {
    println!("Hello, world!");
}

pub struct ProcessorStauts {
    pub carry_flag: bool,
    pub zero_flag: bool,
    pub interrupt_disable: bool,
    pub decimal_mode_flag: bool, //Does not support
    pub break_command: bool,
    pub overflow_flag: bool,
    pub negative_flag: bool,
}

pub struct CPU {
    pub accumulator: u8,
    pub status: ProcessorStauts,
    pub program_counter: u16,
}

impl CPU {
    pub fn new() -> Self {
        CPU {
            accumulator: 0,
            status: ProcessorStauts {
                carry_flag: false,
                zero_flag: false,
                interrupt_disable: false,
                decimal_mode_flag: false,
                break_command: false,
                overflow_flag: false,
                negative_flag: false,
            },
            program_counter: 0,
        }
    }
}
