FEATURES = {
    "cpu",
    -- "vdp",
    -- "memory"
} 

counter = 0

function pre_tick()
    -- print("pre_tick")
    if cpu["pc"] == 0x135 then
        counter = counter + 1
    end

    if cpu["pc"] == 0x9f then
        log("load_sega_license_message called " .. counter .. " times")
    end
end

function post_tick()
    -- print("post_tick")
end