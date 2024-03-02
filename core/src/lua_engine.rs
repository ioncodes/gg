use std::collections::HashMap;

use lazy_static::lazy_static;
use log::{debug, info};
use mlua::prelude::*;
use std::sync::Mutex;
use z80::instruction::{Reg16, Reg8};

use crate::bus::Bus;
use crate::cpu::Cpu;
use crate::vdp::Vdp;

lazy_static! {
    static ref HOOKS: Mutex<HashMap<u16, (String, HookType)>> = Mutex::new(HashMap::new());
}

#[derive(PartialEq)]
pub(crate) enum HookType {
    CpuExec = 0,
    VramWrite = 1,
    VramRead = 2,
}

pub(crate) struct LuaEngine {
    lua: Option<Lua>,
    features: Vec<String>,
}

impl LuaEngine {
    pub(crate) fn new(script: Option<String>) -> LuaEngine {
        let (lua, features) = if let Some(script) = script {
            let lua = Lua::new();
            let globals = lua.globals();

            // Logging function
            globals
                .set(
                    "log",
                    lua.create_function(|_, msg: String| {
                        info!("{}", msg);
                        Ok(())
                    })
                    .unwrap(),
                )
                .unwrap();

            // Instruction-level hook function
            globals
                .set(
                    "install_hook",
                    lua.create_function(|_, (address, pre_tick_hook, function_name): (u16, u8, String)| {
                        match pre_tick_hook {
                            0 => HOOKS.lock().unwrap().insert(address, (function_name, HookType::CpuExec)),
                            1 => HOOKS.lock().unwrap().insert(address, (function_name, HookType::VramWrite)),
                            2 => HOOKS.lock().unwrap().insert(address, (function_name, HookType::VramRead)),
                            _ => panic!("Invalid hook type in Lua script"),
                        };
                        Ok(())
                    })
                    .unwrap(),
                )
                .unwrap();

            // Hook type constants
            globals.set("CPU_EXEC", HookType::CpuExec as usize).unwrap();
            globals.set("VRAM_WRITE", HookType::VramWrite as usize).unwrap();
            globals.set("VRAM_READ", HookType::VramRead as usize).unwrap();

            lua.load(&script).exec().unwrap();

            let features = globals.get::<_, Vec<String>>("FEATURES").unwrap();

            drop(globals);

            (Some(lua), features)
        } else {
            (None, vec![])
        };

        LuaEngine { lua, features }
    }

    pub(crate) fn hook_exists(&self, address: u16, current_instruction_state: HookType) -> bool {
        if self.lua.is_none() {
            return false;
        }

        if let Some((_, hook_type)) = HOOKS.lock().unwrap().get(&address) {
            return *hook_type == current_instruction_state;
        }

        false
    }

    pub(crate) fn execute_hook(&self, address: u16, current_instruction_state: HookType) {
        if self.lua.is_none() {
            return;
        }

        if let Some(lua) = &self.lua {
            if let Some((function_name, hook_type)) = HOOKS.lock().unwrap().get(&address) {
                if *hook_type != current_instruction_state {
                    return;
                }

                let globals = lua.globals();
                let function_name = function_name.clone();

                debug!("Executing Lua hook: {}", &function_name);
                let func = globals.get::<_, LuaFunction>(function_name).unwrap();
                let _ = func.call::<_, ()>(());
            }
        }
    }

    pub(crate) fn create_tables(&self, cpu: &Cpu, vdp: &Vdp, bus: &Bus) {
        if self.lua.is_none() {
            return;
        }

        self.create_cpu_table(cpu);
        self.create_vdp_table(vdp);
        self.create_memory_table(bus);
    }

    pub(crate) fn create_cpu_table(&self, cpu: &Cpu) {
        if let Some(lua) = &self.lua {
            let globals = lua.globals();

            if self.features.contains(&"cpu".to_string()) {
                let cpu_table = lua.create_table().unwrap();
                cpu_table.set("af", cpu.get_register_u16(Reg16::AF)).unwrap();
                cpu_table.set("bc", cpu.get_register_u16(Reg16::BC)).unwrap();
                cpu_table.set("de", cpu.get_register_u16(Reg16::DE)).unwrap();
                cpu_table.set("hl", cpu.get_register_u16(Reg16::HL)).unwrap();
                cpu_table.set("sp", cpu.get_register_u16(Reg16::SP)).unwrap();
                cpu_table.set("pc", cpu.get_register_u16(Reg16::PC)).unwrap();
                cpu_table.set("ix", cpu.get_register_u16(Reg16::IX(None))).unwrap();
                cpu_table.set("iy", cpu.get_register_u16(Reg16::IY(None))).unwrap();
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
    }

    pub(crate) fn create_vdp_table(&self, vdp: &Vdp) {
        if let Some(lua) = &self.lua {
            let globals = lua.globals();

            if self.features.contains(&"vdp".to_string()) {
                let vdp_table = lua.create_table().unwrap();
                vdp_table.set("vram", vdp.vram.buffer.clone()).unwrap();
                vdp_table.set("cram", vdp.cram.buffer.clone()).unwrap();
                vdp_table.set("scanline", vdp.scanline).unwrap();
                globals.set("vdp", vdp_table).unwrap();
            }
        }
    }

    pub(crate) fn create_memory_table(&self, bus: &Bus) {
        if let Some(lua) = &self.lua {
            let globals = lua.globals();

            if self.features.contains(&"memory".to_string()) {
                let memory_table = lua.create_table().unwrap();
                memory_table.set("rom", bus.rom.memory().buffer.clone()).unwrap();
                memory_table.set("ram", bus.ram.buffer.clone()).unwrap();
                memory_table.set("bios_rom", bus.bios_rom.buffer.clone()).unwrap();
                globals.set("memory", memory_table).unwrap();
            }
        }
    }
}
