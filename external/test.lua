FEATURES = {
    "cpu",
    "vdp",
    -- "memory"
}

function post_sega_license_hook()
    log("VRAM:")
    
    str = ""
    for i = 1,0x4000 do
        str = str .. string.format("%02x ", vdp["vram"][i])

        if i % 16 == 0 then
            str = string.format("%04x: %s", i, str)
            log(str)
            str = ""
        end
    end
end

-- function vdp_set_address_hook()
--     log("VDP_set_address_register(de:" .. string.format("%04x", cpu["de"]) .. ")")
-- end

-- function out_hook()
--     log("hl:" .. string.format("%04x", cpu["hl"]))
--     log("out(a:" .. string.format("%02x", cpu["a"]) .. ")")
-- end

install_hook(0x9f, PRE_TICK, "post_sega_license_hook")
-- install_hook(0x135, PRE_TICK, "vdp_set_address_hook")
-- install_hook(0x139, PRE_TICK, "out_hook")