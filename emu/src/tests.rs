#[cfg(test)]
mod tests {
    use serde_json::Value;
    use crate::{bus::Passthrough, cpu::Flags, system::System};

    #[datatest::files("../external/jsmoo/misc/tests/GeneratedTests/z80/v1", {
        input in r"^.*\.json"
    })]
    fn test_cpu(input: &str) {
        let json: Value = serde_json::from_str(&input).unwrap();
        let tests = json.as_array().unwrap();
        for test in tests {
            let test = test.as_object().unwrap();
            let name = test.get("name").unwrap().as_str().unwrap();
            let bytes = name.split(" ").collect::<Vec<&str>>()[0];
            let bytes = u32::from_str_radix(bytes, 16).unwrap();
            let bytes = bytes.to_ne_bytes();

            let initial = test.get("initial").unwrap().as_object().unwrap();
            let final_ = test.get("final").unwrap().as_object().unwrap();

            let mut system = System::new(None, false);
            system.disable_bios();
            system.bus.rom.resize(0xffff);

            system.cpu.registers.a = initial.get("a").unwrap().as_u64().unwrap() as u8;
            system.cpu.registers.b = initial.get("b").unwrap().as_u64().unwrap() as u8;
            system.cpu.registers.c = initial.get("c").unwrap().as_u64().unwrap() as u8;
            system.cpu.registers.d = initial.get("d").unwrap().as_u64().unwrap() as u8;
            system.cpu.registers.e = initial.get("e").unwrap().as_u64().unwrap() as u8;
            system.cpu.registers.h = initial.get("h").unwrap().as_u64().unwrap() as u8;
            system.cpu.registers.l = initial.get("l").unwrap().as_u64().unwrap() as u8;
            system.cpu.registers.f = Flags::from_bits(initial.get("f").unwrap().as_u64().unwrap() as u8).unwrap();
            system.cpu.registers.pc = initial.get("pc").unwrap().as_u64().unwrap() as u16;
            system.cpu.registers.sp = initial.get("sp").unwrap().as_u64().unwrap() as u16;

            for idx in 0..bytes.len() {
                system
                    .bus
                    .write_passthrough(&Passthrough::Rom, (system.cpu.registers.pc + idx as u16) as usize, bytes[idx]);
            }

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

            match system.tick() {
                Ok(_) => {}
                Err(e) => panic!("{}", e),
            }

            assert_eq!(system.cpu.registers.a, final_.get("a").unwrap().as_u64().unwrap() as u8, "Testcase {}", name);
            assert_eq!(system.cpu.registers.b, final_.get("b").unwrap().as_u64().unwrap() as u8, "Testcase {}", name);
            assert_eq!(system.cpu.registers.c, final_.get("c").unwrap().as_u64().unwrap() as u8, "Testcase {}", name);
            assert_eq!(system.cpu.registers.d, final_.get("d").unwrap().as_u64().unwrap() as u8, "Testcase {}", name);
            assert_eq!(system.cpu.registers.e, final_.get("e").unwrap().as_u64().unwrap() as u8, "Testcase {}", name);
            assert_eq!(system.cpu.registers.h, final_.get("h").unwrap().as_u64().unwrap() as u8, "Testcase {}", name);
            assert_eq!(system.cpu.registers.l, final_.get("l").unwrap().as_u64().unwrap() as u8, "Testcase {}", name);
            assert_eq!(
                system.cpu.registers.f,
                Flags::from_bits(final_.get("f").unwrap().as_u64().unwrap() as u8).unwrap(),
                "Testcase {}", name
            );
            assert_eq!(system.cpu.registers.pc, final_.get("pc").unwrap().as_u64().unwrap() as u16, "Testcase {}", name);
            assert_eq!(system.cpu.registers.sp, final_.get("sp").unwrap().as_u64().unwrap() as u16, "Testcase {}", name);
        }
    }
}
