use std::collections::HashMap;

use lazy_static::lazy_static;
use log::{debug, info};
use rlua::{Context, Function, Lua};
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
    PreTick,
    PostTick,
    Both,
}

pub(crate) struct LuaEngine {
    lua: Option<Lua>,
}

impl LuaEngine {
    pub(crate) fn new(script: Option<String>) -> LuaEngine {
        let lua = if let Some(script) = script {
            let lua = Lua::new();
            lua.context(|ctx| {
                let globals = ctx.globals();

                // Logging function
                globals
                    .set(
                        "log",
                        ctx.create_function(|_, msg: String| {
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
                        ctx.create_function(|_, (address, pre_tick_hook, function_name): (u16, u8, String)| {
                            match pre_tick_hook {
                                0 => HOOKS.lock().unwrap().insert(address, (function_name, HookType::PreTick)),
                                1 => HOOKS.lock().unwrap().insert(address, (function_name, HookType::PostTick)),
                                2 => HOOKS.lock().unwrap().insert(address, (function_name, HookType::Both)),
                                _ => panic!("Invalid hook type in Lua script"),
                            };
                            Ok(())
                        })
                        .unwrap(),
                    )
                    .unwrap();

                // Hook type constants
                globals.set("PRE_TICK", 0).unwrap();
                globals.set("POST_TICK", 1).unwrap();
                globals.set("BOTH_TICK", 2).unwrap();

                ctx.load(&script).exec().unwrap();
            });

            Some(lua)
        } else {
            None
        };

        LuaEngine { lua }
    }

    pub(crate) fn hook_exists(&self, address: u16, current_instruction_state: HookType) -> bool {
        if self.lua.is_none() {
            return false;
        }

        if let Some((_, hook_type)) = HOOKS.lock().unwrap().get(&address) {
            return *hook_type == current_instruction_state || *hook_type == HookType::Both;
        }

        false
    }

    pub(crate) fn execute_hook(&self, address: u16, current_instruction_state: HookType) {
        if let Some(lua) = &self.lua {
            if let Some((function_name, hook_type)) = HOOKS.lock().unwrap().get(&address) {
                if *hook_type != current_instruction_state || *hook_type == HookType::Both {
                    return;
                }

                lua.context(|ctx| {
                    let globals = ctx.globals();
                    let function_name = function_name.clone();

                    debug!("Executing Lua hook: {}", &function_name);
                    let func = globals.get::<_, Function>(function_name).unwrap();
                    func.call::<_, ()>(())
                })
                .unwrap();
            }
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
            memory_table.set("rom", bus.rom.memory().buffer.clone()).unwrap();
            memory_table.set("ram", bus.ram.buffer.clone()).unwrap();
            memory_table.set("bios_rom", bus.bios_rom.buffer.clone()).unwrap();
            globals.set("memory", memory_table).unwrap();
        }
    }
}
