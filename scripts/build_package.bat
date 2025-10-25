@echo off
REM Build Rosey package with PyInstaller (Windows)
REM Usage: scripts\build_package.bat

setlocal

cd /d "%~dp0\.."

echo ==^> Installing PyInstaller...
pip install pyinstaller

echo.
echo ==^> Building Rosey package...
pyinstaller rosey.spec --clean --noconfirm

echo.
echo ==^> Package built successfully!
echo     Binary location: dist\rosey\
echo.
echo To run smoke tests:
echo     scripts\smoke_test.bat

endlocal
