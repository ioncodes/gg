use log::info;
use rlua::{Context, Lua, Function};
use z80::instruction::{Reg16, Reg8};

use crate::{bus::Bus, cpu::Cpu, vdp::Vdp};

pub(crate) struct LuaEngine {
    lua: Option<Lua>,
}

impl LuaEngine {
    pub(crate) fn new(script: Option<String>) -> LuaEngine {
        let lua = if let Some(script) = script {
            let lua = Lua::new();
            lua.context(|ctx| {
                ctx.load(&script).exec().unwrap();
                ctx.globals().set("log", ctx.create_function(|_, msg: String| {
                    info!("{}", msg);
                    Ok(())
                }).unwrap()).unwrap();
            });
            Some(lua)
        } else {
            None
        };

        LuaEngine { lua }
    }

    pub(crate) fn execute_function(&self, function_name: &str) {
        if let Some(lua) = &self.lua {
            lua.context(|ctx| {
                let globals = ctx.globals();
                let func = globals.get::<_, Function>(function_name)?;
                func.call::<_, ()>(())
            }).unwrap();
        }
    }

    pub(crate) fn create_tables(&self, cpu: &Cpu, vdp: &Vdp, bus: &Bus) {
        if let Some(lua) = &self.lua {
            lua.context(|ctx| {
                self.create_cpu_table(cpu, ctx);
                self.create_vdp_table(vdp, ctx);
                self.create_memory_table(bus, ctx);
            });
        }
    }

    fn create_cpu_table(&self, cpu: &Cpu, ctx: Context<'_>) {
        let globals = ctx.globals();
        let features = globals.get::<_, Vec<String>>("FEATURES").unwrap();

        if features.contains(&"cpu".to_string()) {
            let cpu_table = ctx.create_table().unwrap();
            cpu_table.set("af", cpu.get_register_u16(Reg16::AF)).unwrap();
            cpu_table.set("bc", cpu.get_register_u16(Reg16::BC)).unwrap();
            cpu_table.set("de", cpu.get_register_u16(Reg16::DE)).unwrap();
            cpu_table.set("hl", cpu.get_register_u16(Reg16::HL)).unwrap();
            cpu_table.set("sp", cpu.get_register_u16(Reg16::SP)).unwrap();
            cpu_table.set("pc", cpu.get_register_u16(Reg16::PC)).unwrap();
            cpu_table.set("a", cpu.get_register_u8(Reg8::A)).unwrap();
            cpu_table.set("b", cpu.get_register_u8(Reg8::B)).unwrap();
            cpu_table.set("c", cpu.get_register_u8(Reg8::C)).unwrap();
            cpu_table.set("d", cpu.get_register_u8(Reg8::D)).unwrap();
            cpu_table.set("e", cpu.get_register_u8(Reg8::E)).unwrap();
            cpu_table.set("h", cpu.get_register_u8(Reg8::H)).unwrap();
            cpu_table.set("l", cpu.get_register_u8(Reg8::L)).unwrap();
            cpu_table.set("f", cpu.get_register_u8(Reg8::F)).unwrap();
            globals.set("cpu", cpu_table).unwrap();
        }
    }

    fn create_vdp_table(&self, vdp: &Vdp, ctx: Context<'_>) {
        let globals = ctx.globals();
        let features = globals.get::<_, Vec<String>>("FEATURES").unwrap();

        if features.contains(&"vdp".to_string()) {
            let vdp_table = ctx.create_table().unwrap();
            vdp_table.set("vram", vdp.vram.buffer.clone()).unwrap();
            vdp_table.set("cram", vdp.cram.buffer.clone()).unwrap();
            vdp_table.set("h_counter", vdp.h).unwrap();
            vdp_table.set("v_counter", vdp.v).unwrap();
            globals.set("vdp", vdp_table).unwrap();
        }
    }

    fn create_memory_table(&self, bus: &Bus, ctx: Context<'_>) {
        let globals = ctx.globals();
        let features = globals.get::<_, Vec<String>>("FEATURES").unwrap();

        if features.contains(&"memory".to_string()) {
            let memory_table = ctx.create_table().unwrap();
            memory_table.set("rom", bus.rom.buffer.clone()).unwrap();
            memory_table.set("ram", bus.ram.buffer.clone()).unwrap();
            memory_table.set("bios_rom", bus.bios_rom.buffer.clone()).unwrap();
            globals.set("memory", memory_table).unwrap();
        }
    }
}
