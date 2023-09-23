use famicom_emulator::cpu::{AddressingMode, ProcessorStatus, CPU};

#[cfg(test)]
mod tests {
    use super::*;

    pub fn run<F>(program: Vec<u8>, f: F) -> CPU
    where
        F: FnOnce(&mut CPU),
    {
        let mut cpu = CPU::new();
        cpu.load(program);
        cpu.reset();
        f(&mut cpu);
        cpu.run();
        cpu
    }

    mod opcode_tests {
        use super::*;

        mod lda {
            use super::*;

            #[test]
            fn test_lda_effects() {
                let mut cpu = run(vec![0xa9, 0x05, 0x00], |_| {});
                assert_eq!(
                    cpu.status
                        .contains(ProcessorStatus::NEGATIVE | ProcessorStatus::ZERO),
                    false
                );

                cpu = run(vec![0xa9, 0x00, 0x00], |_| {});
                assert!(cpu.status.contains(ProcessorStatus::ZERO));

                cpu = run(vec![0xa9, 0x80, 0x00], |_| {});
                assert!(cpu.status.contains(ProcessorStatus::NEGATIVE));
            }

            #[test]
            fn test_lda_immediate() {
                let cpu = run(vec![0xA9, 0x10, 0x00], |_| {});
                assert_eq!(cpu.accumulator, 0x10);
            }

            #[test]
            fn test_lda_zero_page() {
                let cpu = run(vec![0xA5, 0x10, 0x00], |cpu| {
                    cpu.mem_write(0x10, 0x78);
                });

                assert_eq!(cpu.accumulator, 0x78);
            }

            #[test]
            fn test_lda_zero_page_x() {
                let cpu = run(vec![0xB5, 0x08, 0x00], |cpu| {
                    cpu.mem_write(0x28, 0x07);
                    cpu.index_register_x = 0x20;
                });

                assert_eq!(cpu.accumulator, 0x07);
            }

            #[test]
            fn test_lda_absolute() {
                let cpu = run(vec![0xAD, 0x28, 0x52, 0x00], |cpu| {
                    cpu.mem_write(0x5228, 0xF0);
                });
                assert_eq!(cpu.accumulator, 0xF0);
            }

            #[test]
            fn test_lda_absolute_x() {
                let cpu = run(vec![0xBD, 0xA8, 0xF0, 0x00], |cpu| {
                    cpu.mem_write(0xF0B9, 0x98);
                    cpu.index_register_x = 0x11;
                });
                assert_eq!(cpu.accumulator, 0x98);
            }

            #[test]
            fn test_lda_absolute_y() {
                let cpu = run(vec![0xB9, 0xB0, 0x59, 0x00], |cpu| {
                    cpu.mem_write(0x5A00, 0xEA);
                    cpu.index_register_y = 0x50;
                });
                assert_eq!(cpu.accumulator, 0xEA);
            }

            #[test]
            fn test_lda_indirect_x() {
                let cpu = run(vec![0xA1, 0x80, 0x00], |cpu| {
                    cpu.mem_write_u16(0x85, 0x2030);
                    cpu.mem_write(0x2030, 0xE1);
                    cpu.index_register_x = 0x05;
                });
                assert_eq!(cpu.accumulator, 0xE1);
            }

            #[test]
            fn test_lda_indirect_y() {
                let cpu = run(vec![0xB1, 0x80, 0x00], |cpu| {
                    cpu.mem_write_u16(0x80, 0x2030);
                    cpu.mem_write(0x2035, 0xE6);
                    cpu.index_register_y = 0x05;
                });
                assert_eq!(cpu.accumulator, 0xE6);
            }
        }
        mod tax {

            use super::*;

            #[test]
            fn test_tax_effects() {
                let mut cpu = CPU::new();
                cpu.load_and_run(vec![0xa9, 0x10, 0xaa, 0x00]);

                assert_eq!(cpu.index_register_x, 16);
                assert_eq!(
                    cpu.status
                        .contains(ProcessorStatus::ZERO | ProcessorStatus::NEGATIVE),
                    false
                );

                cpu.load_and_run(vec![0xa9, 0x00, 0xaa, 0x00]);
                assert_eq!(cpu.status.contains(ProcessorStatus::ZERO), true);

                cpu.load_and_run(vec![0xa9, 0x80, 0xaa, 0x00]);
                assert_eq!(cpu.status.contains(ProcessorStatus::NEGATIVE), true);
            }
        }
        mod imx {

            use super::*;

            #[test]
            fn test_inx_effects() {
                let mut cpu = CPU::new();
                cpu.load_and_run(vec![0xe8, 0x00]);

                assert_eq!(cpu.index_register_x, 1);
                assert_eq!(
                    cpu.status
                        .contains(ProcessorStatus::ZERO | ProcessorStatus::NEGATIVE),
                    false
                );

                cpu.load_and_run(vec![0xa9, 0xff, 0xaa, 0xe8, 0x00]);
                assert_eq!(cpu.index_register_x, 0);
                assert_eq!(cpu.status.contains(ProcessorStatus::ZERO), true);

                cpu.load_and_run(vec![0xa9, 0x80, 0xaa, 0xe8, 0x00]);
                assert_eq!(cpu.status.contains(ProcessorStatus::NEGATIVE), true);

                cpu.load_and_run(vec![0xa9, 0xff, 0xaa, 0xe8, 0xe8, 0x00]);
                assert_eq!(cpu.index_register_x, 1)
            }
        }
        mod adc {
            use super::*;

            mod effects {
                use super::*;

                #[test]
                fn test_adc_no_carry() {
                    //キャリーなし
                    let mut cpu = CPU::new();

                    cpu.load(vec![0x69, 0x10, 0x00]);
                    cpu.reset();
                    cpu.accumulator = 0x02;
                    cpu.run();

                    println!();

                    assert_eq!(cpu.accumulator, 0x12);
                    assert_eq!(
                        cpu.status.contains(
                            ProcessorStatus::CARRY
                                | ProcessorStatus::ZERO
                                | ProcessorStatus::OVERFLOW
                                | ProcessorStatus::NEGATIVE
                        ),
                        false
                    );
                }

                #[test]
                fn test_adc_has_carry() {
                    // 計算前にキャリーあり

                    let mut cpu = CPU::new();

                    cpu.load(vec![0x69, 0x10, 0x00]);
                    cpu.reset();
                    cpu.accumulator = 0x01;
                    cpu.status.insert(ProcessorStatus::CARRY);
                    cpu.run();

                    assert_eq!(cpu.accumulator, 0x12);
                    assert_eq!(
                        cpu.status.contains(
                            ProcessorStatus::CARRY
                                | ProcessorStatus::ZERO
                                | ProcessorStatus::OVERFLOW
                                | ProcessorStatus::NEGATIVE
                        ),
                        false
                    );
                }
                #[test]
                fn test_adc_occur_carry() {
                    // 計算中にキャリー発生

                    let mut cpu = CPU::new();

                    cpu.load(vec![0x69, 0x01, 0x00]);
                    cpu.reset();
                    cpu.accumulator = 0xFF;
                    cpu.run();

                    assert_eq!(cpu.accumulator, 0x0);
                    assert_eq!(
                        cpu.status
                            .contains(ProcessorStatus::CARRY | ProcessorStatus::ZERO),
                        true
                    );
                    assert_eq!(
                        cpu.status
                            .contains(ProcessorStatus::OVERFLOW | ProcessorStatus::NEGATIVE),
                        false
                    );
                }

                #[test]
                fn test_adc_occur_overflow_plus() {
                    //キャリーとオーバーフローが発生し計算結果がプラスの値になる場合
                    let mut cpu = CPU::new();

                    cpu.load(vec![0x69, 0x01, 0x00]);
                    cpu.reset();
                    cpu.accumulator = 0x7F;
                    cpu.run();

                    assert_eq!(cpu.accumulator, 0x80);
                    assert_eq!(
                        cpu.status
                            .contains(ProcessorStatus::NEGATIVE | ProcessorStatus::OVERFLOW),
                        true
                    )
                }

                #[test]
                fn test_adc_occur_overflow_minus() {
                    //キャリーとオーバーフローが発生し計算結果がマイナスの値になる場合
                    let mut cpu = CPU::new();

                    cpu.load(vec![0x69, 0x81, 0x00]);
                    cpu.reset();
                    cpu.accumulator = 0x81;
                    cpu.run();

                    assert_eq!(cpu.accumulator, 0x2);
                    assert_eq!(
                        cpu.status
                            .contains(ProcessorStatus::CARRY | ProcessorStatus::OVERFLOW),
                        true
                    )
                }

                #[test]
                fn test_adc_occur_overflow_minus_has_carry() {
                    //計算前にキャリーがあり計算中にオーバーフローが発生して計算結果がプラスの値になる場合
                    let mut cpu = CPU::new();

                    cpu.load(vec![0x69, 0x6F, 0x00]);
                    cpu.reset();
                    cpu.accumulator = 0x10;
                    cpu.status.insert(ProcessorStatus::CARRY);
                    cpu.run();

                    assert_eq!(cpu.accumulator, 0x80);
                    assert_eq!(
                        cpu.status
                            .contains(ProcessorStatus::NEGATIVE | ProcessorStatus::OVERFLOW),
                        true
                    )
                }

                #[test]
                fn test_adc_occur_overflow_plus_has_carry() {
                    //計算前にキャリーがあり計算中にオーバーフローが発生して計算結果がマイナスの値になる場合
                    let mut cpu = CPU::new();

                    cpu.load(vec![0x69, 0x81, 0x00]);
                    cpu.reset();
                    cpu.accumulator = 0x81;
                    cpu.status.insert(ProcessorStatus::CARRY);
                    cpu.run();

                    assert_eq!(cpu.accumulator, 0x03);
                    assert_eq!(
                        cpu.status
                            .contains(ProcessorStatus::CARRY | ProcessorStatus::OVERFLOW),
                        true
                    )
                }

                #[test]
                fn test_adc_no_overflow() {
                    //オーバーフローなし
                    let mut cpu = CPU::new();

                    cpu.load(vec![0x69, 0x7F, 0x00]);
                    cpu.reset();
                    cpu.accumulator = 0x82;
                    cpu.run();

                    println!();

                    assert_eq!(cpu.accumulator, 0x01);
                    assert_eq!(
                        cpu.status.contains(
                            ProcessorStatus::CARRY
                                | ProcessorStatus::OVERFLOW
                                | ProcessorStatus::NEGATIVE
                        ),
                        false
                    );
                }
            }
        }
        mod sbc {
            use super::*;
            mod effects {
                use super::*;

                #[test]
                fn test_sbc_no_carry() {
                    //キャリーなし
                    let mut cpu = CPU::new();

                    cpu.load(vec![0xE9, 0x10, 0x00]);
                    cpu.reset();
                    cpu.accumulator = 0x20;
                    cpu.run();

                    assert_eq!(cpu.accumulator, 0x0F);
                    assert_eq!(cpu.status.contains(ProcessorStatus::CARRY), true);
                    assert_eq!(
                        cpu.status.contains(
                            ProcessorStatus::ZERO
                                | ProcessorStatus::NEGATIVE
                                | ProcessorStatus::OVERFLOW
                        ),
                        false
                    );
                }

                #[test]
                fn test_sbc_has_carry() {
                    // 計算前にキャリーあり

                    let mut cpu = CPU::new();

                    cpu.load(vec![0xE9, 0x10, 0x00]);
                    cpu.reset();
                    cpu.accumulator = 0x20;
                    cpu.status.insert(ProcessorStatus::CARRY);
                    cpu.run();

                    assert_eq!(cpu.accumulator, 0x10);
                    assert_eq!(cpu.status.contains(ProcessorStatus::CARRY), true);
                    assert_eq!(
                        cpu.status.contains(
                            ProcessorStatus::ZERO
                                | ProcessorStatus::NEGATIVE
                                | ProcessorStatus::OVERFLOW
                        ),
                        false
                    );
                }
                #[test]
                fn test_sbc_occur_carry() {
                    // 計算中にキャリー発生

                    let mut cpu = CPU::new();

                    cpu.load(vec![0xE9, 0x02, 0x00]);
                    cpu.reset();
                    cpu.accumulator = 0x01;
                    cpu.run();

                    assert_eq!(cpu.accumulator, 0xFE);
                    assert_eq!(cpu.status.contains(ProcessorStatus::NEGATIVE), true);
                    assert_eq!(
                        cpu.status.contains(
                            ProcessorStatus::CARRY
                                | ProcessorStatus::ZERO
                                | ProcessorStatus::OVERFLOW
                        ),
                        false
                    );
                }

                #[test]
                fn test_sbc_occur_overflow() {
                    //オーバーフローが発生
                    let mut cpu = CPU::new();

                    cpu.load(vec![0xE9, 0x81, 0x00]);
                    cpu.reset();
                    cpu.accumulator = 0x7F;
                    cpu.run();

                    assert_eq!(cpu.accumulator, 0xFD);
                    assert_eq!(
                        cpu.status
                            .contains(ProcessorStatus::OVERFLOW | ProcessorStatus::NEGATIVE),
                        true
                    );
                    assert_eq!(
                        cpu.status
                            .contains(ProcessorStatus::CARRY | ProcessorStatus::ZERO),
                        false
                    );
                }

                #[test]
                fn test_sbc_occur_overflow_has_carry() {
                    //計算前にキャリーがあり計算中にオーバーフローが発生
                    let mut cpu = CPU::new();

                    cpu.load(vec![0xE9, 0x7F, 0x00]);
                    cpu.reset();
                    cpu.accumulator = 0x7F;
                    cpu.status.insert(ProcessorStatus::CARRY);
                    cpu.run();

                    assert_eq!(cpu.accumulator, 0x0);
                    assert_eq!(
                        cpu.status
                            .contains(ProcessorStatus::CARRY | ProcessorStatus::ZERO),
                        true
                    );
                    assert_eq!(
                        cpu.status
                            .contains(ProcessorStatus::NEGATIVE | ProcessorStatus::OVERFLOW),
                        false
                    )
                }

                #[test]
                fn test_sbc_no_overflow() {
                    //オーバーフローなし
                    let mut cpu = CPU::new();

                    cpu.load(vec![0xE9, 0x7F, 0x00]);
                    cpu.reset();
                    cpu.accumulator = 0x7E;
                    cpu.run();

                    println!();

                    assert_eq!(cpu.accumulator, 0xFE);
                    assert_eq!(cpu.status.contains(ProcessorStatus::NEGATIVE), true);
                    assert_eq!(
                        cpu.status.contains(
                            ProcessorStatus::CARRY
                                | ProcessorStatus::ZERO
                                | ProcessorStatus::OVERFLOW
                        ),
                        false
                    );
                }
            }
        }
        mod and {
            use super::*;

            #[test]
            fn test_and() {
                let mut cpu = CPU::new();

                cpu.load(vec![0x29, 0xF0, 0x00]);
                cpu.reset();
                cpu.accumulator = 0x6E;
                cpu.run();

                assert_eq!(cpu.accumulator, 0x60);
                assert_eq!(cpu.status.is_empty(), true);
            }
        }
        mod eor {
            use super::*;

            #[test]
            fn test_eor() {
                let mut cpu = CPU::new();

                cpu.load(vec![0x49, 0xF0, 0x00]);
                cpu.reset();
                cpu.accumulator = 0x6E;
                cpu.run();

                assert_eq!(cpu.accumulator, 0x9E);
                assert_eq!(cpu.status.contains(ProcessorStatus::NEGATIVE), true);
            }
        }
        mod ora {
            use super::*;

            #[test]
            fn test_ora() {
                let mut cpu = CPU::new();

                cpu.load(vec![0x09, 0xF0, 0x00]);
                cpu.reset();
                cpu.accumulator = 0x6E;
                cpu.run();

                assert_eq!(cpu.accumulator, 0xFE);
                assert_eq!(cpu.status.contains(ProcessorStatus::NEGATIVE), true);
            }
        }
        mod asl {
            use super::*;

            #[test]
            fn test_asl_load_acc() {
                let mut cpu = CPU::new();

                cpu.load(vec![0x0A, 0x00]);
                cpu.reset();
                cpu.accumulator = 0b11101010;
                cpu.run();

                assert_eq!(cpu.accumulator, 0b11010100);
                assert_eq!(cpu.status.contains(ProcessorStatus::CARRY), true);
                assert_eq!(cpu.status.contains(ProcessorStatus::NEGATIVE), true);
            }

            #[test]
            fn test_asl_load_mem() {
                let mut cpu = CPU::new();

                cpu.mem_write(0x10, 0b01101010);
                cpu.load(vec![0x06, 0x10, 0x00]);
                cpu.reset();
                cpu.run();

                assert_eq!(cpu.mem_read(0x10), 0b11010100);
                assert_eq!(cpu.status.contains(ProcessorStatus::CARRY), false);
                assert_eq!(cpu.status.contains(ProcessorStatus::NEGATIVE), false);
            }
        }
        mod lsr {
            use super::*;

            #[test]
            fn test_lsr_load_acc() {
                let mut cpu = CPU::new();

                cpu.load(vec![0x4A, 0x00]);
                cpu.reset();
                cpu.accumulator = 0b11101010;
                cpu.run();

                assert_eq!(cpu.accumulator, 0b01110101);
                assert_eq!(cpu.status.contains(ProcessorStatus::CARRY), true);
                assert_eq!(cpu.status.contains(ProcessorStatus::NEGATIVE), true);
            }

            #[test]
            fn test_lsr_load_mem() {
                let mut cpu = CPU::new();

                cpu.mem_write(0x10, 0b01101010);
                cpu.load(vec![0x46, 0x10, 0x00]);
                cpu.reset();
                cpu.run();

                assert_eq!(cpu.mem_read(0x10), 0b00110101);
                assert_eq!(cpu.status.contains(ProcessorStatus::CARRY), false);
                assert_eq!(cpu.status.contains(ProcessorStatus::NEGATIVE), false);
            }
        }
        mod rol {
            use super::*;

            #[test]
            fn test_rol_load_acc() {
                let mut cpu = CPU::new();

                cpu.load(vec![0x2A, 0x00]);
                cpu.reset();
                cpu.accumulator = 0b10101011;
                cpu.run();

                assert_eq!(cpu.accumulator, 0b01010111);
                assert_eq!(cpu.status.contains(ProcessorStatus::CARRY), true);
                assert_eq!(cpu.status.contains(ProcessorStatus::NEGATIVE), true);
            }

            #[test]
            fn test_rol_load_mem() {
                let mut cpu = CPU::new();

                cpu.mem_write(0x10, 0b01100101);
                cpu.load(vec![0x26, 0x10, 0x00]);
                cpu.reset();
                cpu.run();

                assert_eq!(cpu.mem_read(0x10), 0b11001010);
                assert_eq!(cpu.status.contains(ProcessorStatus::CARRY), false);
                assert_eq!(cpu.status.contains(ProcessorStatus::NEGATIVE), false);
            }
        }
        mod ror {
            use super::*;

            #[test]
            fn test_ror_load_acc() {
                let mut cpu = CPU::new();

                cpu.load(vec![0x6A, 0x00]);
                cpu.reset();
                cpu.accumulator = 0b10101011;
                cpu.run();

                assert_eq!(cpu.accumulator, 0b11010101);
                assert_eq!(cpu.status.contains(ProcessorStatus::CARRY), true);
                assert_eq!(cpu.status.contains(ProcessorStatus::NEGATIVE), true);
            }

            #[test]
            fn test_ror_load_mem() {
                let mut cpu = CPU::new();

                cpu.mem_write(0x10, 0b01100101);
                cpu.load(vec![0x66, 0x10, 0x00]);
                cpu.reset();
                cpu.run();

                assert_eq!(cpu.mem_read(0x10), 0b10110010);
                assert_eq!(cpu.status.contains(ProcessorStatus::CARRY), false);
                assert_eq!(cpu.status.contains(ProcessorStatus::NEGATIVE), false);
            }
        }
        mod branch {
            use super::*;
            mod bcc {
                use super::*;
                #[test]
                fn test_bcc() {
                    let program = vec![0x90, 0x02, 0x00, 0x00, 0x00];
                    let cpu = run(program, |_| {});
                    assert_eq!(cpu.program_counter, 0x8005);
                }
            }
            mod bcs {
                use super::*;

                #[test]
                fn test_bcs() {
                    let program = vec![0xB0, 0x02, 0x00, 0x00, 0x00];
                    let cpu = run(program, |cpu| cpu.status.insert(ProcessorStatus::CARRY));

                    assert_eq!(cpu.program_counter, 0x8005);
                }
            }
            mod bne {
                use super::*;

                #[test]
                fn test_bne() {
                    let program = vec![0xD0, 0x03, 0x00, 0x00, 0x00, 0xE8, 0x00];
                    let cpu = run(program, |_| {});

                    assert_eq!(cpu.program_counter, 0x8007);
                    assert_eq!(cpu.index_register_x, 0x1);
                }
            }
            mod beq {
                use super::*;

                #[test]
                fn test_beq() {
                    let program = vec![0xD0, 0x03, 0x00, 0x00, 0x00, 0xE8, 0x00];
                    let cpu = run(program, |cpu| cpu.status.insert(ProcessorStatus::ZERO));

                    assert_eq!(cpu.program_counter, 0x8007);
                    assert_eq!(cpu.index_register_x, 0x1);
                }
            }
            mod bvc {
                use super::*;

                #[test]
                fn test_bvc() {
                    let program = vec![0xD0, 0x04, 0x00, 0x00, 0x00, 0x00, 0xE8, 0x00];
                    let cpu = run(program, |_| {});

                    assert_eq!(cpu.program_counter, 0x8008);
                    assert_eq!(cpu.index_register_x, 0x1);
                }
            }
            mod bvs {
                use super::*;

                #[test]
                fn test_bvs() {
                    let program = vec![0xD0, 0x04, 0x00, 0x00, 0x00, 0x00, 0xE8, 0x00];
                    let cpu = run(program, |cpu| cpu.status.insert(ProcessorStatus::OVERFLOW));

                    assert_eq!(cpu.program_counter, 0x8008);
                    assert_eq!(cpu.index_register_x, 0x1);
                }
            }
            mod bpl {
                use super::*;

                #[test]
                fn test_bvs() {
                    let program = vec![0x10, 0x03, 0x00, 0x00, 0x00, 0xE8, 0x00];
                    let cpu = run(program, |_| {});
                    assert_eq!(cpu.program_counter, 0x8007);
                    assert_eq!(cpu.index_register_x, 0x01);
                }
            }
            mod bmi {
                use super::*;

                #[test]
                fn test_bmi() {
                    let program = vec![0x10, 0x03, 0x00, 0x00, 0x00, 0xE8, 0x00];
                    let cpu = run(program, |cpu| cpu.status.insert(ProcessorStatus::NEGATIVE));

                    assert_eq!(cpu.program_counter, 0x8007);
                    assert_eq!(cpu.index_register_x, 0x01);
                }
            }
        }
        mod bit {
            use super::*;

            #[test]
            fn test_bit() {
                let mut cpu = run(vec![0x24, 0x81, 0x00], |cpu| {
                    cpu.mem_write(0x81, 0x60);
                    cpu.accumulator = 0x70;
                });

                assert_eq!(cpu.accumulator, 0x70);
                assert_eq!(cpu.mem_read(0x81), 0x60);
                assert_eq!(
                    cpu.status.contains(
                        ProcessorStatus::ZERO
                            | ProcessorStatus::OVERFLOW
                            | ProcessorStatus::NEGATIVE
                    ),
                    true
                );

                cpu = run(vec![0x24, 0x81, 0x00], |cpu| {
                    cpu.mem_write(0x81, 0x60);
                    cpu.accumulator = 0x90;
                });

                assert_eq!(cpu.status.contains(ProcessorStatus::ZERO), false);
                assert_eq!(
                    cpu.status
                        .contains(ProcessorStatus::OVERFLOW | ProcessorStatus::NEGATIVE),
                    true
                );
            }
        }
        mod flag {
            use super::*;

            mod carry {
                use super::*;

                #[test]
                fn test_carry() {
                    let mut cpu = run(vec![0x18, 0x00], |cpu| {
                        cpu.status.set(ProcessorStatus::CARRY, true)
                    });

                    assert_eq!(cpu.status.contains(ProcessorStatus::CARRY), false);

                    cpu = run(vec![0x38, 0x00], |cpu| {
                        cpu.status.set(ProcessorStatus::CARRY, false)
                    });

                    assert_eq!(cpu.status.contains(ProcessorStatus::CARRY), true);
                }
            }
            mod interrupt_disable {
                use super::*;

                #[test]
                fn test_interrupt_disable() {
                    let mut cpu = run(vec![0x58, 0x00], |cpu| {
                        cpu.status.set(ProcessorStatus::INTERRUPT_DISABLE, true)
                    });

                    assert_eq!(
                        cpu.status.contains(ProcessorStatus::INTERRUPT_DISABLE),
                        false
                    );

                    cpu = run(vec![0x78, 0x00], |cpu| {
                        cpu.status.set(ProcessorStatus::INTERRUPT_DISABLE, false)
                    });

                    assert_eq!(
                        cpu.status.contains(ProcessorStatus::INTERRUPT_DISABLE),
                        true
                    );
                }
            }
            mod decimal_mode {
                use super::*;

                #[test]
                fn test_decimal_mode() {
                    let mut cpu = run(vec![0xD8, 0x00], |cpu| {
                        cpu.status.set(ProcessorStatus::DECIMAL, true)
                    });

                    assert_eq!(cpu.status.contains(ProcessorStatus::DECIMAL), false);

                    cpu = run(vec![0xF8, 0x00], |cpu| {
                        cpu.status.set(ProcessorStatus::DECIMAL, false)
                    });

                    assert_eq!(cpu.status.contains(ProcessorStatus::DECIMAL), true);
                }
            }
            mod overflow {
                use super::*;

                #[test]
                fn test_overflow() {
                    let cpu = run(vec![0xB8, 0x00], |cpu| {
                        cpu.status.set(ProcessorStatus::OVERFLOW, true)
                    });

                    assert_eq!(cpu.status.contains(ProcessorStatus::OVERFLOW), false);
                }
            }
        }
        mod cmp {
            use super::*;
            mod cmp {
                use super::*;

                #[test]
                fn test_cmp_greater_than_or_eq() {
                    let cpu = run(vec![0xC9, 0x10, 0x00], |cpu| {
                        cpu.accumulator = 0x50;
                    });

                    // accumulator > memory
                    assert_eq!(cpu.status.contains(ProcessorStatus::CARRY), true);
                    assert_eq!(
                        cpu.status
                            .contains(ProcessorStatus::ZERO | ProcessorStatus::NEGATIVE),
                        false
                    );
                }
                #[test]
                fn test_cmp_eq() {
                    let cpu = run(vec![0xC9, 0x50, 0x00], |cpu| {
                        cpu.accumulator = 0x50;
                    });

                    // accumulator = memory
                    assert_eq!(
                        cpu.status
                            .contains(ProcessorStatus::CARRY | ProcessorStatus::ZERO),
                        true
                    );
                    assert_eq!(cpu.status.contains(ProcessorStatus::NEGATIVE), false);
                }
                #[test]
                fn test_cmp_less_than() {
                    // accumulator < memory
                    let cpu = run(vec![0xC9, 0xB8, 0x00], |cpu| {
                        cpu.accumulator = 0x10;
                        cpu.mem_write(0xB8, 0x50);
                    });

                    assert_eq!(
                        cpu.status.contains(
                            ProcessorStatus::CARRY
                                | ProcessorStatus::ZERO
                                | ProcessorStatus::NEGATIVE
                        ),
                        false
                    );
                    assert_eq!(cpu.status.contains(ProcessorStatus::NEGATIVE), false);
                }

                mod cpx {
                    use super::*;

                    #[test]
                    fn test_cpx_greater_than_or_eq() {
                        let cpu = run(vec![0xE0, 0x10, 0x00], |cpu| {
                            cpu.index_register_x = 0x50;
                        });

                        // index X > memory
                        assert_eq!(cpu.status.contains(ProcessorStatus::CARRY), true);
                        assert_eq!(
                            cpu.status
                                .contains(ProcessorStatus::ZERO | ProcessorStatus::NEGATIVE),
                            false
                        );
                    }
                    #[test]
                    fn test_cpx_eq() {
                        let cpu = run(vec![0xE0, 0x50, 0x00], |cpu| {
                            cpu.index_register_x = 0x50;
                        });

                        // index X = memory
                        assert_eq!(
                            cpu.status
                                .contains(ProcessorStatus::CARRY | ProcessorStatus::ZERO),
                            true
                        );
                        assert_eq!(cpu.status.contains(ProcessorStatus::NEGATIVE), false);
                    }
                    #[test]
                    fn test_cpx_less_than() {
                        // index X < memory
                        let cpu = run(vec![0xE0, 0xB8, 0x00], |cpu| {
                            cpu.index_register_x = 0x10;
                            cpu.mem_write(0xB8, 0x50);
                        });

                        assert_eq!(
                            cpu.status.contains(
                                ProcessorStatus::CARRY
                                    | ProcessorStatus::ZERO
                                    | ProcessorStatus::NEGATIVE
                            ),
                            false
                        );
                        assert_eq!(cpu.status.contains(ProcessorStatus::NEGATIVE), false);
                    }
                }
                mod cpy {
                    use super::*;

                    #[test]
                    fn test_cpy_greater_than_or_eq() {
                        let cpu = run(vec![0xC0, 0x10, 0x00], |cpu| {
                            cpu.index_register_y = 0x50;
                        });

                        // index Y > memory
                        assert_eq!(cpu.status.contains(ProcessorStatus::CARRY), true);
                        assert_eq!(
                            cpu.status
                                .contains(ProcessorStatus::ZERO | ProcessorStatus::NEGATIVE),
                            false
                        );
                    }
                    #[test]
                    fn test_cpy_eq() {
                        let cpu = run(vec![0xC0, 0x50, 0x00], |cpu| {
                            cpu.index_register_y = 0x50;
                        });

                        // index Y = memory
                        assert_eq!(
                            cpu.status
                                .contains(ProcessorStatus::CARRY | ProcessorStatus::ZERO),
                            true
                        );
                        assert_eq!(cpu.status.contains(ProcessorStatus::NEGATIVE), false);
                    }
                    #[test]
                    fn test_cpy_less_than() {
                        // index Y < memory
                        let cpu = run(vec![0xC0, 0xB8, 0x00], |cpu| {
                            cpu.index_register_y = 0x10;
                            cpu.mem_write(0xB8, 0x50);
                        });

                        assert_eq!(
                            cpu.status.contains(
                                ProcessorStatus::CARRY
                                    | ProcessorStatus::ZERO
                                    | ProcessorStatus::NEGATIVE
                            ),
                            false
                        );
                        assert_eq!(cpu.status.contains(ProcessorStatus::NEGATIVE), false);
                    }
                }
            }
        }
        mod decrement {
            use super::*;

            #[test]
            fn test_dec_memory() {
                let cpu = run(vec![0xC6, 0x20, 0x00], |cpu| {
                    cpu.mem_write(0x20, 0x70);
                });
                assert_eq!(cpu.mem_read(0x20), 0x6F);
                assert_eq!(cpu.status.contains(ProcessorStatus::NEGATIVE), false);
            }
            #[test]
            fn test_dec_index_x() {
                let cpu = run(vec![0xCA, 0x00], |cpu| {
                    cpu.index_register_x = 0x70;
                });
                assert_eq!(cpu.index_register_x, 0x6F);
                assert_eq!(cpu.status.contains(ProcessorStatus::NEGATIVE), false);
            }
            #[test]
            fn test_dec_index_y() {
                let cpu = run(vec![0x88, 0x00], |cpu| {
                    cpu.index_register_y = 0x70;
                });
                assert_eq!(cpu.index_register_y, 0x6F);
                assert_eq!(cpu.status.contains(ProcessorStatus::NEGATIVE), false);
            }
        }
        mod increment {
            use super::*;

            #[test]
            fn test_inc_memory() {
                let cpu = run(vec![0xE6, 0x20, 0x00], |cpu| {
                    cpu.mem_write(0x20, 0x70);
                });
                assert_eq!(cpu.mem_read(0x20), 0x71);
                assert_eq!(cpu.status.contains(ProcessorStatus::NEGATIVE), false);
            }
            #[test]
            fn test_inc_index_y() {
                let cpu = run(vec![0xC8, 0x00], |cpu| {
                    cpu.index_register_y = 0x70;
                });
                assert_eq!(cpu.index_register_y, 0x71);
                assert_eq!(cpu.status.contains(ProcessorStatus::NEGATIVE), false);
            }
        }
        mod nop {
            use super::*;

            #[test]
            fn test_nop() {
                let cpu = run(vec![0xEA, 0x00], |_| {});
                assert_eq!(cpu.program_counter, 0x8002);
            }
        }
        mod load_mem {
            use super::*;

            #[test]
            fn load_mem_to_registe_x() {
                let cpu = run(vec![0xA2, 0x10, 0x00], |_| {});
                assert_eq!(cpu.index_register_x, 0x10);
            }

            #[test]
            fn load_mem_to_registe_y() {
                let cpu = run(vec![0xA0, 0x10, 0x00], |_| {});
                assert_eq!(cpu.index_register_y, 0x10);
            }
        }
        mod store {
            use super::*;

            #[test]
            fn store_accumulator_for_mem() {
                let cpu = run(vec![0x85, 0xF0, 0x00], |cpu| {
                    cpu.accumulator = 0x90;
                    cpu.mem_write(0xF0, 0x00);
                });

                assert_eq!(cpu.mem_read(0xF0), cpu.accumulator);
            }

            #[test]
            fn store_register_x_for_mem() {
                let cpu = run(vec![0x86, 0xF0, 0x00], |cpu| {
                    cpu.index_register_x = 0x90;
                    cpu.mem_write(0xF0, 0x00);
                });

                assert_eq!(cpu.mem_read(0xF0), cpu.index_register_x);
            }

            #[test]
            fn store_register_y_for_mem() {
                let cpu = run(vec![0x84, 0xF0, 0x00], |cpu| {
                    cpu.index_register_x = 0x90;
                    cpu.mem_write(0xF0, 0x00);
                });

                assert_eq!(cpu.mem_read(0xF0), cpu.index_register_y);
            }
        }
        mod transfer {
            use super::*;

            #[test]
            fn test_accumlator_to_register_y() {
                let cpu = run(vec![0xA8, 0x00], |cpu| {
                    cpu.accumulator = 0x90;
                });
                assert_eq!(cpu.index_register_y, cpu.accumulator);
            }

            #[test]
            fn test_register_y_to_accumlator() {
                let cpu = run(vec![0xA8, 0x00], |cpu| {
                    cpu.index_register_y = 0x90;
                });
                assert_eq!(cpu.accumulator, cpu.index_register_y);
            }

            #[test]
            fn test_register_x_to_accumlator() {
                let cpu = run(vec![0x8A, 0x00], |cpu| {
                    cpu.index_register_x = 0x90;
                });
                assert_eq!(cpu.accumulator, cpu.index_register_x);
            }

            #[test]
            fn test_txs_register_x_to_stack() {
                let cpu = run(vec![0x9A, 0x00], |cpu| {
                    cpu.index_register_x = 0x90;
                });
                assert_eq!(cpu.stack_pointer, cpu.index_register_x);
            }

            #[test]
            fn test_tsx_stack_to_register_x() {
                let cpu = run(vec![0xBA, 0x00], |cpu| {
                    cpu.stack_pointer = 0x90;
                });
                assert_eq!(cpu.index_register_x, cpu.stack_pointer);
            }
        }
        mod stack {
            use super::*;

            mod push {
                use super::*;

                #[test]
                fn test_push_accumlator() {
                    let cpu = run(vec![0x48, 0x00], |cpu| {
                        cpu.accumulator = 0x90;
                    });
                    assert_eq!(cpu.memory[0x1FF], 0x90);
                    assert_eq!(cpu.stack_pointer, 0xFE);
                }
                #[test]
                fn test_push_processor_status() {
                    let cpu = run(vec![0x08, 0x00], |cpu| {
                        cpu.status = ProcessorStatus::CARRY | ProcessorStatus::ZERO;
                    });

                    assert_eq!(cpu.memory[0x1FF], 0x03); // (ProcessorStatus::CARRY | ProcessorStatus::ZERO) to bit flag is 0x03
                    assert_eq!(cpu.stack_pointer, 0xFE);
                }
            }
            mod pull {
                use super::*;

                #[test]
                fn test_pull_accumlator() {
                    let cpu = run(vec![0x48, 0x68, 0x00], |cpu| {
                        cpu.accumulator = 0x90;
                    });
                    assert_eq!(
                        cpu.status
                            .contains(ProcessorStatus::CARRY | ProcessorStatus::ZERO),
                        false
                    );
                    assert_eq!(cpu.accumulator, 0x90);
                    assert_eq!(cpu.stack_pointer, 0xFF);
                }

                #[test]
                fn test_pull_processor_status() {
                    let cpu = run(vec![0x08, 0x28, 0x00], |cpu| {
                        cpu.status = ProcessorStatus::CARRY | ProcessorStatus::ZERO;
                    });

                    assert_eq!(
                        cpu.status
                            .contains(ProcessorStatus::CARRY | ProcessorStatus::ZERO),
                        true
                    );
                    assert_eq!(cpu.stack_pointer, 0xFF);
                }
            }
        }
        mod jmp {
            use super::*;

            #[test]
            fn test_jmp_absolute() {
                let cpu = run(vec![0x4C, 0x34, 0x89, 0x00], |_| {});
                assert_eq!(cpu.program_counter, 0x8935); // JMP命令(0x8934) + BREAKE命令(1)
            }

            #[test]
            fn test_jmp_indirect() {
                let cpu = run(vec![0x6C, 0x80, 0x00], |cpu| {
                    cpu.mem_write_u16(0x80, 0x8900); //JMP命令(0x8900) + BREAKE命令(1)
                });
                assert_eq!(cpu.program_counter, 0x8901);
            }
        }

        mod subroutine {
            use super::*;

            #[test]
            fn test_jsr_rts() {
                let cpu = run(vec![0x20, 0xAF, 0x80, 0x20, 0x00, 0x9A, 0x00], |cpu| {
                    cpu.mem_write(0x80AF, 0xA9); //LDA
                    cpu.mem_write(0x80B0, 0x42); // LDA. 0x42
                    cpu.mem_write(0x80B1, 0x60); //RTS
                    cpu.mem_write(0x9A00, 0xE8); //INX
                    cpu.mem_write(0x9A01, 0x60); //RTS
                });
                assert_eq!(cpu.accumulator, 0x42);
                assert_eq!(cpu.index_register_x, 0x01);
                assert_eq!(cpu.program_counter, 0x8007);
            }
        }
        // TODO 割り込み処理の実装後にテストを書く
        // mod interrupt {
        //     use super::*;

        // #[test]
        // fn test_interrupt_brk_rti() {
        //     let cpu = run(vec![0x00, 0x00, 0x00, 0x00, 0x00], |cpu| {
        //         cpu.mem_write(0xFFFE, 0x00);
        //         cpu.mem_write(0xFFFF, 0x80);
        //         cpu.program_counter = 0x8000;
        //         cpu.mem_write(0x8000, 0x00); //BRK
        //         cpu.mem_write(0x8001, 0x40); //RTI
        //     });
        //     assert_eq!(cpu.program_counter, 0x8002);
        // }
        // mod brk {

        //     use super::*;

        //     #[test]
        //     fn test_brk_effects() {
        //         let cpu = run(vec![0x00], |cpu| {
        //             cpu.status.set(ProcessorStatus::CARRY, true);
        //         });
        //     }
        //     assert_eq!(cpu.program_counter, 0xFFFE);
        //     assert_eq!(cpu.stack_pointer, 0xFD);
        //     // assert_eq!(cpu.mem_read_u16(0x1FF), 0x0002);
        //     // assert_eq!(cpu.mem_read(0x1FD), 0b01000100);
        //     assert_eq!(cpu.pop_u16(), 0x0002);
        //     assert_eq!(
        //         cpu.pop(),
        //         ProcessorStatus::from_name(ProcessorStatus::BREAK | ProcessorStatus::CARRY)
        //             .unrap()
        //     );
        //     assert_eq!(cpu.status.contains(ProcessorStatus::BREAK), true);
        // }
        // }
    }
    mod operand_address_tests {

        use super::*;

        #[test]
        fn test_get_operand_address() {
            let mut cpu = CPU::new();
            cpu.program_counter = 0x90;
            let mut mode = AddressingMode::Immediate;
            let mut effective_address = cpu.get_operand_address(&mode);
            assert_eq!(
                effective_address, cpu.program_counter,
                "オペランドアドレスがプログラムカウンタと一致していません"
            );

            cpu.reset();
            cpu.memory[cpu.program_counter as usize] = 0x44;
            mode = AddressingMode::ZeroPage;
            effective_address = cpu.get_operand_address(&mode);
            assert_eq!(effective_address, 0x44);

            cpu.reset();
            mode = AddressingMode::ZeroPage;
            for address in 0x00..=0xFF {
                cpu.memory[cpu.program_counter as usize] = address;
                effective_address = cpu.get_operand_address(&mode);
                assert_eq!(effective_address, address as u16);
            }

            cpu.reset();
            cpu.memory[cpu.program_counter as usize] = 0x44;
            cpu.index_register_x = 0x10;
            mode = AddressingMode::ZeroPage_X;
            effective_address = cpu.get_operand_address(&mode);
            assert_eq!(effective_address, 0x54);

            cpu.reset();
            cpu.index_register_y = 0x02;
            cpu.memory[cpu.program_counter as usize] = 0x50;
            mode = AddressingMode::ZeroPage_Y;
            effective_address = cpu.get_operand_address(&mode);
            assert_eq!(effective_address, 0x52);

            cpu.reset();
            cpu.memory[cpu.program_counter as usize] = 0x80;
            cpu.memory[cpu.program_counter.wrapping_add(1) as usize] = 0x49;
            mode = AddressingMode::Absolute;
            effective_address = cpu.get_operand_address(&mode);
            assert_eq!(effective_address, 0x4980);

            cpu.reset();
            cpu.index_register_x = 0x20;
            cpu.memory[cpu.program_counter as usize] = 0x30;
            cpu.memory[cpu.program_counter.wrapping_add(1) as usize] = 0x98;
            mode = AddressingMode::Absolute_X;
            effective_address = cpu.get_operand_address(&mode);
            assert_eq!(effective_address, 0x9850);

            cpu.reset();
            cpu.index_register_y = 0x42;
            cpu.memory[cpu.program_counter as usize] = 0x50;
            cpu.memory[cpu.program_counter.wrapping_add(1) as usize] = 0xE0;
            mode = AddressingMode::Absolute_Y;
            effective_address = cpu.get_operand_address(&mode);
            assert_eq!(effective_address, 0xE092);

            cpu.reset();
            cpu.memory[cpu.program_counter as usize] = 0x22;
            cpu.memory[0x22] = 0x50;
            cpu.memory[0x23] = 0xAC;
            mode = AddressingMode::Indirect;
            effective_address = cpu.get_operand_address(&mode);
            assert_eq!(effective_address, 0xAC50);

            cpu.reset();
            cpu.memory[cpu.program_counter as usize] = 0x40;
            cpu.index_register_x = 0x05;
            cpu.memory[0x45] = 0x10;
            cpu.memory[0x46] = 0x09;
            mode = AddressingMode::Indirect_X;
            effective_address = cpu.get_operand_address(&mode);
            assert_eq!(effective_address, 0x0910);

            cpu.reset();
            cpu.memory[cpu.program_counter as usize] = 0xA0;
            cpu.index_register_y = 0x05;
            cpu.memory[0xA0] = 0x50;
            cpu.memory[0xA1] = 0xB2;
            mode = AddressingMode::Indirect_Y;
            effective_address = cpu.get_operand_address(&mode);
            assert_eq!(effective_address, 0xB255);

            cpu.reset();
            cpu.memory[cpu.program_counter as usize] = 0x60;
            mode = AddressingMode::Relative;
            effective_address = cpu.get_operand_address(&mode);
            assert_eq!(effective_address, 0x60);

            cpu.reset();
            cpu.accumulator = 0x42;
            mode = AddressingMode::Accumulator;
            effective_address = cpu.get_operand_address(&mode);
            assert_eq!(effective_address, 0x42);

            cpu.reset();
            mode = AddressingMode::Implicit;
            effective_address = cpu.get_operand_address(&mode);
            assert_eq!(effective_address, 0);
        }
    }

    mod memory_access {

        use super::*;

        #[test]
        fn test_mem_read_write() {
            let mut cpu = CPU::new();

            cpu.mem_write(0x8000, 0xAB);
            cpu.mem_write(0x8001, 0xCD);

            let data1 = cpu.mem_read(0x8000);
            let data2 = cpu.mem_read(0x8001);

            assert_eq!(data1, 0xAB);
            assert_eq!(data2, 0xCD);
        }

        #[test]
        fn test_mem_read_write_u16() {
            let mut cpu = CPU::new();
            cpu.mem_write_u16(0x8000, 0xABCD);
            let value = cpu.mem_read_u16(0x8000);
            assert_eq!(value, 0xABCD)
        }
    }

    mod cpu_instruction_tests {

        use super::*;

        #[test]
        fn test_load() {
            let mut cpu = CPU::new();
            let program: Vec<u8> = vec![0x01, 0x02, 0x03];
            cpu.load(program.clone());

            for (i, &byte) in program.iter().enumerate() {
                let memory_index = 0x8000 + i;
                assert!(
                    memory_index < cpu.memory.len(),
                    "Memory index out of range: 0x{:X}",
                    memory_index
                );
                assert_eq!(cpu.memory[memory_index], byte);
            }
            assert_eq!(cpu.program_counter, 0);
        }

        #[test]
        fn test_reset() {
            let mut cpu = CPU::new();
            cpu.accumulator = 1;
            cpu.index_register_x = 1;
            cpu.status.insert(ProcessorStatus::NEGATIVE);
            cpu.reset();
            assert_eq!(cpu.accumulator, 0);
            assert_eq!(cpu.index_register_x, 0);
            assert_eq!(cpu.status.contains(ProcessorStatus::NEGATIVE), false);
        }

        #[test]
        fn test_5_ops_working_together() {
            let mut cpu = CPU::new();
            cpu.load_and_run(vec![0xa9, 0xc0, 0xaa, 0xe8, 0x00]);

            assert_eq!(cpu.index_register_x, 0xc1)
        }
    }
}
