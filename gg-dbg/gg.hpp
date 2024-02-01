#ifndef GG_HPP
#define GG_HPP

#include <QLibrary>
#include <cstdint>

struct __attribute__((__packed__)) Registers
{
    uint8_t a;
    uint8_t b;
    uint8_t c;
    uint8_t d;
    uint8_t e;
    uint8_t h;
    uint8_t l;
    uint8_t f;
    uint16_t pc;
    uint16_t sp;
};

#include <gg-ffi.hpp>

class gg
{
public:
    static void load()
    {
        gg::init = resolve_gg_function<decltype(ffi::gg_init)>("gg_init");
        gg::tick = resolve_gg_function<decltype(ffi::gg_tick)>("gg_tick");
        gg::fetch_registers = resolve_gg_function<decltype(ffi::gg_fetch_registers)>("gg_fetch_registers");
    }

    static inline decltype(ffi::gg_init)* init = nullptr;
    static inline decltype(ffi::gg_tick)* tick = nullptr;
    static inline decltype(ffi::gg_fetch_registers)* fetch_registers = nullptr;

private:
    static inline QLibrary ffi = QLibrary("core_ffi.dll");

    template<typename T>
    static T* resolve_gg_function(const char* name)
    {
        if (!ffi.isLoaded())
            ffi.load();

        return reinterpret_cast<T*>(ffi.resolve(name));
    }
};

#endif // GG_HPP
