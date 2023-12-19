FEATURES = {
    "cpu",
    -- "vdp",
    -- "memory"
} 

counter = 0
jmp_flag_tmp = false

function pre_tick()
    -- if cpu["pc"] == 0x13b then
    --     log("HL: " .. string.format("%x", cpu["hl"]))
    -- end

    if cpu["pc"] == 0x13e then
        log("EEEEEEEEEEEEEEEEEE")
    end

    if cpu["pc"] == 0x135 then
        counter = counter + 1
        log("Calling VDP_set_address_register(de:" .. string.format("%x", cpu["de"]) .. ")")
    end

    if cpu["pc"] == 0x139 then
        log("Writing to data port with " .. string.format("%x", cpu["a"]))
    end

    if cpu["pc"] == 0x9f then
        log("Loop entered " .. counter .. " times")
    end

    -- if cpu["pc"] == 0x145 then
    --     jmp_flag_tmp = true
    -- end
end

function post_tick()
    -- if jmp_flag_tmp and cpu["pc"] == 0x12d then
    --     log("Jump taken to " .. string.format("%x", cpu["pc"]))
    --     jmp_flag_tmp = false
    -- elseif jmp_flag_tmp then
    --     log("Jump not taken")
    --     jmp_flag_tmp = false
    -- end
end