"""Muestra qué botones del mando se pulsan y su número (1 a 32), usando winmm.

Uso: python leer_botones.py [segundos]
"""
import ctypes
import sys
import time
from ctypes import wintypes

winmm = ctypes.WinDLL("winmm")


class JOYINFOEX(ctypes.Structure):
    _fields_ = [(n, wintypes.DWORD) for n in (
        "dwSize", "dwFlags", "dwXpos", "dwYpos", "dwZpos", "dwRpos",
        "dwUpos", "dwVpos", "dwButtons", "dwButtonNumber", "dwPOV",
        "dwReserved1", "dwReserved2")]


JOY_RETURNALL = 0xFF


def leer(idx):
    info = JOYINFOEX()
    info.dwSize = ctypes.sizeof(info)
    info.dwFlags = JOY_RETURNALL
    return info if winmm.joyGetPosEx(idx, ctypes.byref(info)) == 0 else None


def main():
    segundos = float(sys.argv[1]) if len(sys.argv) > 1 else 40
    ids = [i for i in range(16) if leer(i)]
    print(f"Dispositivos de juego activos (ids winmm): {ids}", flush=True)
    previo = {i: 0 for i in ids}
    fin = time.time() + segundos
    t0 = time.time()
    while time.time() < fin:
        for i in ids:
            info = leer(i)
            if not info:
                continue
            nuevos = info.dwButtons & ~previo[i]
            for b in range(32):
                if nuevos >> b & 1:
                    print(f"{time.time() - t0:5.1f}s  id {i}  botón {b + 1}", flush=True)
            previo[i] = info.dwButtons
        time.sleep(0.01)


main()
