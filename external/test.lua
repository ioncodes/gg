FEATURES = {
    "cpu",
    "vdp",
    -- "memory"
}

function post_sega_license_hook()
    log("post_sega_license_hook")
    if cpu["pc"] == 0x9f then
        -- dump vram
        for i = 1,0x4000 do
            io.write(string.format("%02x ", vdp["vram"][i]))

            if i % 16 == 15 then
                io.write("\n")
            end
        end
    end
end

install_hook(0x9f, PRE_TICK, "post_sega_license_hook")