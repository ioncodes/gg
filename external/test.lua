FEATURES = {
    "cpu",
    -- "vdp",
    -- "memory"
} 

counter = 0

function pre_tick()
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
end

function post_tick()
end