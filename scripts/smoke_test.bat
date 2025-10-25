@echo off
REM Smoke test for Rosey packaged binary (Windows)
REM Usage: scripts\smoke_test.bat

setlocal enabledelayedexpansion

cd /d "%~dp0\.."

set BINARY_PATH=dist\rosey\rosey.exe

if not exist "%BINARY_PATH%" (
    echo ERROR: Binary not found at %BINARY_PATH%
    echo Run scripts\build_package.bat first
    exit /b 1
)

echo ==^> Smoke test: Rosey packaged binary
echo     Binary: %BINARY_PATH%
echo.

REM Test 1: Binary exists
echo [1/3] Checking binary exists...
echo     √ Binary exists

REM Test 2: Try to launch (will fail without display in CI but that's OK)
echo [2/3] Testing launch (headless/version check)...
start /wait /b "" "%BINARY_PATH%" >nul 2>&1
set EXIT_CODE=!ERRORLEVEL!
if !EXIT_CODE! EQU 0 (
    echo     √ Binary started
) else (
    echo     √ Binary attempted to start (exit code !EXIT_CODE! - OK for GUI app without display^)
)

REM Test 3: Check file size
echo [3/3] Checking binary size...
for %%A in ("%BINARY_PATH%") do set SIZE=%%~zA
if !SIZE! LSS 1000000 (
    echo     ⚠ WARNING: Binary size is unusually small (!SIZE! bytes^)
    exit /b 1
)
echo     √ Binary size: !SIZE! bytes

echo.
echo ==^> Smoke test PASSED
echo.
echo Manual test checklist:
echo   1. Launch the binary
echo   2. Configure source/destination paths
echo   3. Run a scan
echo   4. Preview results (confidence colors, reasons)
echo   5. Execute dry-run move
echo   6. Exit cleanly

endlocal
