"""Muestra los informes HID sin procesar del mando (VID 2563, PID 0575).

Uso: python leer_hid_crudo.py [segundos]
Imprime una línea cada vez que cambia el informe, para comparar botones.
"""
import ctypes
import struct
import sys
import threading
import time
from ctypes import wintypes as w

hid = ctypes.WinDLL("hid")
setup = ctypes.WinDLL("setupapi")
k32 = ctypes.WinDLL("kernel32", use_last_error=True)

VID, PID = 0x2563, 0x0575


class GUID(ctypes.Structure):
    _fields_ = [("a", w.DWORD), ("b", w.WORD), ("c", w.WORD), ("d", ctypes.c_ubyte * 8)]


class InterfaceData(ctypes.Structure):
    _fields_ = [("cb", w.DWORD), ("guid", GUID), ("flags", w.DWORD), ("res", ctypes.c_size_t)]


class Attrs(ctypes.Structure):
    _fields_ = [("size", w.ULONG), ("vid", w.USHORT), ("pid", w.USHORT), ("ver", w.USHORT)]


class Caps(ctypes.Structure):
    _fields_ = [("usage", w.USHORT), ("page", w.USHORT), ("entrada", w.USHORT),
                ("salida", w.USHORT), ("feature", w.USHORT), ("resto", w.USHORT * 27)]


setup.SetupDiGetClassDevsW.restype = ctypes.c_void_p
setup.SetupDiGetClassDevsW.argtypes = [ctypes.c_void_p, w.LPCWSTR, ctypes.c_void_p, w.DWORD]
setup.SetupDiEnumDeviceInterfaces.argtypes = [ctypes.c_void_p, ctypes.c_void_p, ctypes.c_void_p,
                                              w.DWORD, ctypes.c_void_p]
setup.SetupDiGetDeviceInterfaceDetailW.argtypes = [ctypes.c_void_p, ctypes.c_void_p, ctypes.c_void_p,
                                                   w.DWORD, ctypes.c_void_p, ctypes.c_void_p]
k32.CreateFileW.restype = ctypes.c_void_p
k32.CreateFileW.argtypes = [w.LPCWSTR, w.DWORD, w.DWORD, ctypes.c_void_p, w.DWORD, w.DWORD, ctypes.c_void_p]
k32.ReadFile.argtypes = [ctypes.c_void_p, ctypes.c_void_p, w.DWORD, ctypes.c_void_p, ctypes.c_void_p]
hid.HidD_GetAttributes.argtypes = [ctypes.c_void_p, ctypes.c_void_p]
hid.HidD_GetPreparsedData.argtypes = [ctypes.c_void_p, ctypes.c_void_p]
hid.HidP_GetCaps.argtypes = [ctypes.c_void_p, ctypes.c_void_p]


def rutas():
    guid = GUID()
    hid.HidD_GetHidGuid(ctypes.byref(guid))
    h = setup.SetupDiGetClassDevsW(ctypes.byref(guid), None, None, 0x12)
    i = 0
    while True:
        d = InterfaceData()
        d.cb = ctypes.sizeof(d)
        if not setup.SetupDiEnumDeviceInterfaces(h, None, ctypes.byref(guid), i, ctypes.byref(d)):
            return
        i += 1
        req = w.DWORD()
        setup.SetupDiGetDeviceInterfaceDetailW(h, ctypes.byref(d), None, 0, ctypes.byref(req), None)
        buf = ctypes.create_string_buffer(req.value)
        struct.pack_into("I", buf, 0, 8)
        setup.SetupDiGetDeviceInterfaceDetailW(h, ctypes.byref(d), buf, req.value, None, None)
        yield ctypes.wstring_at(ctypes.addressof(buf) + 4)


def abrir(ruta):
    h = k32.CreateFileW(ruta, 0x80000000, 3, None, 3, 0, None)
    if h in (None, ctypes.c_void_p(-1).value):
        return None
    a = Attrs()
    a.size = ctypes.sizeof(a)
    hid.HidD_GetAttributes(h, ctypes.byref(a))
    if (a.vid, a.pid) != (VID, PID):
        k32.CloseHandle(ctypes.c_void_p(h))
        return None
    pp = ctypes.c_void_p()
    hid.HidD_GetPreparsedData(h, ctypes.byref(pp))
    caps = Caps()
    hid.HidP_GetCaps(pp, ctypes.byref(caps))
    return h, caps


def leer(nombre, h, largo, t0):
    previo = b""
    buf = ctypes.create_string_buffer(largo)
    n = w.DWORD()
    while k32.ReadFile(h, buf, largo, ctypes.byref(n), None):
        datos = buf.raw[:n.value]
        if datos != previo:
            print(f"{time.time() - t0:5.1f}s  {nombre}  {datos.hex(' ')}", flush=True)
            previo = datos


def main():
    segundos = float(sys.argv[1]) if len(sys.argv) > 1 else 40
    t0 = time.time()
    hilos = 0
    for n, ruta in enumerate(rutas()):
        r = abrir(ruta)
        if not r:
            continue
        h, caps = r
        print(f"interfaz {n}: uso {caps.page:#x}/{caps.usage:#x}, informe de {caps.entrada} bytes", flush=True)
        if caps.entrada:
            threading.Thread(target=leer, args=(f"if{n}", h, caps.entrada, t0), daemon=True).start()
            hilos += 1
    if not hilos:
        print("No se pudo abrir ninguna interfaz del mando", flush=True)
    time.sleep(segundos)


main()
