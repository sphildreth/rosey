# -*- mode: python ; coding: utf-8 -*-
"""
PyInstaller spec for Rosey CLI (command-line only).
Usage:
  pyinstaller rosey_cli.spec
"""

import sys

block_cipher = None

a = Analysis(
    ["src/rosey/cli.py"],
    pathex=[],
    binaries=[],
    datas=[],
    hiddenimports=[
        "rosey.scanner",
        "rosey.identifier",
        "rosey.scorer",
        "rosey.planner",
        "rosey.mover",
        "rosey.providers",
        "rosey.config",
        "rosey.utils",
    ],
    hookspath=[],
    hooksconfig={},
    runtime_hooks=[],
    excludes=[],
    win_no_prefer_redirects=False,
    win_private_assemblies=False,
    cipher=block_cipher,
    noarchive=False,
)

pyz = PYZ(a.pure, a.zipped_data, cipher=block_cipher)

exe = EXE(
    pyz,
    a.scripts,
    [],
    exclude_binaries=False,  # Include binaries in the executable
    name="rosey",
    debug=False,
    bootloader_ignore_signals=False,
    strip=False,
    upx=True,
    console=True,  # CLI app - show console window
    disable_windowed_traceback=False,
    target_arch=None,
    codesign_identity=None,
    entitlements_file=None,
)
