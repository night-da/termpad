@echo off
setlocal EnableExtensions
set "ROOT=%~dp0"

if exist "%ROOT%target\release\termpad.exe" (
  "%ROOT%target\release\termpad.exe" %*
  exit /b %ERRORLEVEL%
)
if exist "%ROOT%target\debug\termpad.exe" (
  "%ROOT%target\debug\termpad.exe" %*
  exit /b %ERRORLEVEL%
)

cargo run --manifest-path "%ROOT%Cargo.toml" --quiet -- %*
exit /b %ERRORLEVEL%
