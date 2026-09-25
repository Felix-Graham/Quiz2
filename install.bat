@echo off
setlocal enabledelayedexpansion
REM install.bat - installer for French Vocab Quiz on Windows
REM
REM What this does:
REM   1. Makes sure Rust (cargo) is available, installing it via the
REM      official rustup-init if not.
REM   2. Builds the quiz in release mode.
REM   3. Copies the binary + vocab\ + a fresh config.toml into
REM      %LOCALAPPDATA%\FrenchQuiz.
REM   4. Adds that folder to your user PATH so you can just type
REM      "frenchquiz" from any Command Prompt / PowerShell window.

set "SCRIPT_DIR=%~dp0"
set "INSTALL_DIR=%LOCALAPPDATA%\FrenchQuiz"

echo ==^> French Vocab Quiz installer

where cargo >nul 2>nul
if %errorlevel%==0 (
    echo ==^> Found Rust already installed.
) else (
    echo ==^> Rust not found - downloading the official installer...
    set "RUSTUP_EXE=%TEMP%\rustup-init.exe"
    powershell -NoProfile -Command "Invoke-WebRequest -Uri 'https://win.rustup.rs/x86_64' -OutFile '%RUSTUP_EXE%'"
    if not exist "%RUSTUP_EXE%" (
        echo xx Could not download rustup-init.exe. Please install Rust manually from https://rustup.rs and re-run this script.
        exit /b 1
    )
    "%RUSTUP_EXE%" -y --profile minimal
    call "%USERPROFILE%\.cargo\env.bat" 2>nul
    set "PATH=%USERPROFILE%\.cargo\bin;%PATH%"
)

echo ==^> Building French Vocab Quiz (release mode)...
pushd "%SCRIPT_DIR%"
cargo build --release
if not %errorlevel%==0 (
    echo xx Build failed. See the errors above.
    popd
    exit /b 1
)
popd

echo ==^> Installing into %INSTALL_DIR%
if not exist "%INSTALL_DIR%" mkdir "%INSTALL_DIR%"
copy /Y "%SCRIPT_DIR%target\release\frenchquiz.exe" "%INSTALL_DIR%\frenchquiz.exe" >nul

if not exist "%INSTALL_DIR%\vocab" mkdir "%INSTALL_DIR%\vocab"
if exist "%SCRIPT_DIR%vocab" (
    xcopy /Y /I /Q "%SCRIPT_DIR%vocab\*.txt" "%INSTALL_DIR%\vocab\" >nul 2>nul
)

REM Add install dir to the user's PATH if it isn't already there.
echo %PATH% | find /I "%INSTALL_DIR%" >nul
if not %errorlevel%==0 (
    echo ==^> Adding %INSTALL_DIR% to your user PATH...
    for /f "usebackq tokens=2,*" %%A in (`reg query HKCU\Environment /v Path 2^>nul`) do set "OLDPATH=%%B"
    setx PATH "!OLDPATH!;%INSTALL_DIR%" >nul
    echo ==^> Done. Open a NEW Command Prompt / PowerShell window for the PATH change to take effect.
)

echo ==^> Installed! Run it with: frenchquiz
endlocal
