#ifndef GG_HPP
#define GG_HPP

#include <QLibrary>
#include <gg-ffi.hpp>

class gg
{
public:
    static void load()
    {
        gg::init = resolve_gg_function<decltype(ffi::gg_init)>("gg_init");
        gg::tick = resolve_gg_function<decltype(ffi::gg_tick)>("gg_tick");
    }

    static inline decltype(ffi::gg_init)* init = nullptr;
    static inline decltype(ffi::gg_tick)* tick = nullptr;

private:
    static inline QLibrary ffi = QLibrary("core_ffi.dll");

    template<typename T>
    static T* resolve_gg_function(const char* name)
    {
        if (!ffi.isLoaded())
            ffi.load();

        T* ptr = reinterpret_cast<T*>(ffi.resolve(name));
        qDebug("%s => %p", name, ptr);
        return ptr;
    }
};

#endif // GG_HPP
