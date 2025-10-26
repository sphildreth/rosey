# -*- mode: python ; coding: utf-8 -*-
"""
PyInstaller spec for Rosey cross-platform builds.
Usage:
  pyinstaller rosey.spec
"""

import sys
from pathlib import Path

block_cipher = None

# Determine platform-specific settings
is_windows = sys.platform == "win32"
icon_file = "graphics/rosey.ico" if is_windows else "graphics/rosey_256.png"

a = Analysis(
    ["src/rosey/app.py"],
    pathex=[],
    binaries=[],
    datas=[
        ("graphics/rosey.ico", "graphics"),
        ("graphics/rosey_256.png", "graphics"),
        ("graphics/rosey_64.png", "graphics"),
    ],
    hiddenimports=[
        "rosey.scanner",
        "rosey.identifier",
        "rosey.scorer",
        "rosey.planner",
        "rosey.mover",
        "rosey.providers",
        "rosey.config",
        "rosey.ui",
        "rosey.tasks",
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
    exclude_binaries=True,
    name="rosey",
    debug=False,
    bootloader_ignore_signals=False,
    strip=False,
    upx=True,
    console=False,  # GUI app, no console window
    disable_windowed_traceback=False,
    target_arch=None,
    codesign_identity=None,
    entitlements_file=None,
    icon=icon_file,
)

coll = COLLECT(
    exe,
    a.binaries,
    a.zipfiles,
    a.datas,
    strip=False,
    upx=True,
    upx_exclude=[],
    name="rosey",
)

# On macOS, create app bundle (future support)
# app = BUNDLE(coll, name='Rosey.app', icon=icon_file, bundle_identifier='org.rosey.app')
