FEATURES = {
    "cpu",
    "vdp",
    -- "memory"
}

function _dump_vram()
    log("VRAM:")
    
    str = ""
    for i = 1,0x4000 do
        str = str .. string.format("%02x ", vdp["vram"][i])

        if i % 16 == 0 then
            str = string.format("%04x: %s", i - 0x10, str)
            log(str)
            str = ""
        end
    end
end

function post_sega_license_hook()
    _dump_vram()
end

-- function vdp_set_address_hook()
--     log("VDP_set_address_register(de:" .. string.format("%04x", cpu["de"]) .. ")")
-- end

-- function out_hook()
--     log("hl:" .. string.format("%04x", cpu["hl"]))
--     log("out(a:" .. string.format("%02x", cpu["a"]) .. ")")
-- end

-- function outi_hook()
--     value = memory["bios_rom"][cpu["hl"]]
--     log("outi($" .. string.format("%02x", value) .. ", hl:$" .. string.format("%04x", cpu["hl"]) .. ", =b:$" .. string.format("%02x", cpu["b"]) .. ")")
-- end

-- function post_unknown_function_hook()
--     log(string.format("%02x %02x %02x %02x %02x %02x %02x", vdp["vram"][0x3a51], vdp["vram"][0x3a52], vdp["vram"][0x3a53], vdp["vram"][0x3a54], vdp["vram"][0x3a55], vdp["vram"][0x3a56], vdp["vram"][0x3a57]))
-- end

function post_vram_copy_hook()
    _dump_vram()
end

install_hook(0x9f, PRE_TICK, "post_sega_license_hook")
-- install_hook(0x135, PRE_TICK, "vdp_set_address_hook")
-- install_hook(0x139, PRE_TICK, "out_hook")
-- install_hook(0x157, POST_TICK, "outi_hook")
-- install_hook(0xbc, PRE_TICK, "post_unknown_function_hook")
install_hook(0xd4, PRE_TICK, "post_vram_copy_hook")