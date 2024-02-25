#[cfg(test)]
mod tests {
    use crate::bus::{Passthrough, RomWriteProtection};
    use crate::cpu::Flags;
    use crate::system::System;
    use serde_json::Value;
    use z80::instruction::Reg16;

    fn is_ignore(_path: &std::path::Path) -> bool {
        // _path.ends_with("ed b1.json")
        false
    }

    #[datatest::files("../external/jsmoo/misc/tests/GeneratedTests/z80/v1", {
        input in r"^.*\.json" if !is_ignore
    })]
    fn test_cpu(input: &str) {
        let json: Value = serde_json::from_str(&input).unwrap();
        let tests = json.as_array().unwrap();
        for test in tests {
            let test = test.as_object().unwrap();
            let name = test.get("name").unwrap().as_str().unwrap();
            // println!("Running test: {}", name);

            let initial = test.get("initial").unwrap().as_object().unwrap();
            let final_ = test.get("final").unwrap().as_object().unwrap();

            let mut system = System::new(None, false);
            system.disable_bios();
            system.set_abort_on_io_operation_behavior(false);
            system.bus.rom.resize(0xffff);
            system.bus.set_rom_write_protection(RomWriteProtection::Allow);
            system.bus.disable_bank_behavior(true);

            system.cpu.registers.a = initial.get("a").unwrap().as_u64().unwrap() as u8;
            system.cpu.registers.b = initial.get("b").unwrap().as_u64().unwrap() as u8;
            system.cpu.registers.c = initial.get("c").unwrap().as_u64().unwrap() as u8;
            system.cpu.registers.d = initial.get("d").unwrap().as_u64().unwrap() as u8;
            system.cpu.registers.e = initial.get("e").unwrap().as_u64().unwrap() as u8;
            system.cpu.registers.h = initial.get("h").unwrap().as_u64().unwrap() as u8;
            system.cpu.registers.l = initial.get("l").unwrap().as_u64().unwrap() as u8;
            system.cpu.registers.i = initial.get("i").unwrap().as_u64().unwrap() as u8;
            system.cpu.registers.r = initial.get("r").unwrap().as_u64().unwrap() as u8;
            system.cpu.registers.f = Flags::from_bits(initial.get("f").unwrap().as_u64().unwrap() as u8).unwrap();
            system
                .cpu
                .set_register_u16(Reg16::AFShadow, initial.get("af_").unwrap().as_u64().unwrap() as u16);
            system
                .cpu
                .set_register_u16(Reg16::BCShadow, initial.get("bc_").unwrap().as_u64().unwrap() as u16);
            system
                .cpu
                .set_register_u16(Reg16::DEShadow, initial.get("de_").unwrap().as_u64().unwrap() as u16);
            system
                .cpu
                .set_register_u16(Reg16::HLShadow, initial.get("hl_").unwrap().as_u64().unwrap() as u16);
            system
                .cpu
                .set_register_u16(Reg16::IX(None), initial.get("ix").unwrap().as_u64().unwrap() as u16);
            system
                .cpu
                .set_register_u16(Reg16::IY(None), initial.get("iy").unwrap().as_u64().unwrap() as u16);

            system.cpu.registers.pc = initial.get("pc").unwrap().as_u64().unwrap() as u16;
            system.cpu.registers.sp = initial.get("sp").unwrap().as_u64().unwrap() as u16;

            system.cpu.registers.iff1 = initial.get("iff1").unwrap().as_u64().unwrap() != 0;
            system.cpu.registers.iff2 = initial.get("iff2").unwrap().as_u64().unwrap() != 0;

            let ram = initial.get("ram").unwrap().as_array().unwrap();
            for value in ram {
                let addr = value.as_array().unwrap()[0].as_u64().unwrap() as usize;
                let value = value.as_array().unwrap()[1].as_u64().unwrap() as u8;

                if addr < 0xc000 {
                    system.bus.write_passthrough(&Passthrough::Rom, addr, value);
                } else {
                    system.bus.write_passthrough(&Passthrough::Ram, addr - 0xc000, value);
                }
            }

            let decoded = system.decode_instr_at_pc().unwrap().opcode;

            match system.tick() {
                Ok(_) => (),
                Err(e) => panic!("{}", e),
            }

            assert_eq!(
                system.cpu.registers.a,
                final_.get("a").unwrap().as_u64().unwrap() as u8,
                "Testcase {} ({})",
                name,
                decoded
            );
            assert_eq!(
                system.cpu.registers.b,
                final_.get("b").unwrap().as_u64().unwrap() as u8,
                "Testcase {} ({})",
                name,
                decoded
            );
            assert_eq!(
                system.cpu.registers.c,
                final_.get("c").unwrap().as_u64().unwrap() as u8,
                "Testcase {} ({})",
                name,
                decoded
            );
            assert_eq!(
                system.cpu.registers.d,
                final_.get("d").unwrap().as_u64().unwrap() as u8,
                "Testcase {} ({})",
                name,
                decoded
            );
            assert_eq!(
                system.cpu.registers.e,
                final_.get("e").unwrap().as_u64().unwrap() as u8,
                "Testcase {} ({})",
                name,
                decoded
            );
            assert_eq!(
                system.cpu.registers.h,
                final_.get("h").unwrap().as_u64().unwrap() as u8,
                "Testcase {} ({})",
                name,
                decoded
            );
            assert_eq!(
                system.cpu.registers.l,
                final_.get("l").unwrap().as_u64().unwrap() as u8,
                "Testcase {} ({})",
                name,
                decoded
            );
            assert_eq!(
                system.cpu.get_register_u16(Reg16::AFShadow),
                final_.get("af_").unwrap().as_u64().unwrap() as u16,
                "Testcase {} ({})",
                name,
                decoded
            );
            assert_eq!(
                system.cpu.get_register_u16(Reg16::BCShadow),
                final_.get("bc_").unwrap().as_u64().unwrap() as u16,
                "Testcase {} ({})",
                name,
                decoded
            );
            assert_eq!(
                system.cpu.get_register_u16(Reg16::DEShadow),
                final_.get("de_").unwrap().as_u64().unwrap() as u16,
                "Testcase {} ({})",
                name,
                decoded
            );
            assert_eq!(
                system.cpu.get_register_u16(Reg16::HLShadow),
                final_.get("hl_").unwrap().as_u64().unwrap() as u16,
                "Testcase {} ({})",
                name,
                decoded
            );
            assert_eq!(
                system.cpu.get_register_u16(Reg16::IX(None)),
                final_.get("ix").unwrap().as_u64().unwrap() as u16,
                "Testcase {} ({})",
                name,
                decoded
            );
            assert_eq!(
                system.cpu.get_register_u16(Reg16::IY(None)),
                final_.get("iy").unwrap().as_u64().unwrap() as u16,
                "Testcase {} ({})",
                name,
                decoded
            );

            let mut final_f = Flags::from_bits(final_.get("f").unwrap().as_u64().unwrap() as u8).unwrap();
            reset_undocumented_flags(&mut system.cpu.registers.f, &mut final_f);

            assert_eq!(system.cpu.registers.f, final_f, "Testcase {} ({})", name, decoded);

            assert_eq!(
                system.cpu.registers.pc,
                final_.get("pc").unwrap().as_u64().unwrap() as u16,
                "Testcase {} ({})",
                name,
                decoded
            );
            assert_eq!(
                system.cpu.registers.sp,
                final_.get("sp").unwrap().as_u64().unwrap() as u16,
                "Testcase {} ({})",
                name,
                decoded
            );

            assert_eq!(
                system.cpu.registers.iff1,
                final_.get("iff1").unwrap().as_u64().unwrap() != 0,
                "Testcase {} ({})",
                name,
                decoded
            );

            assert_eq!(
                system.cpu.registers.iff2,
                final_.get("iff2").unwrap().as_u64().unwrap() != 0,
                "Testcase {} ({})",
                name,
                decoded
            );

            let ram = final_.get("ram").unwrap().as_array().unwrap();
            for value in ram {
                let addr = value.as_array().unwrap()[0].as_u64().unwrap() as u16;
                let value = value.as_array().unwrap()[1].as_u64().unwrap() as u8;
                assert_eq!(system.bus.read(addr).unwrap(), value, "Testcase {} ({})", name, decoded);
            }
        }
    }

    // We don't care about F3 and F5 for now
    fn reset_undocumented_flags(lhs: &mut Flags, rhs: &mut Flags) {
        lhs.set(Flags::F3, false);
        lhs.set(Flags::F5, false);
        rhs.set(Flags::F3, false);
        rhs.set(Flags::F5, false);
    }
}
