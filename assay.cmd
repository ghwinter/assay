@echo off
rem assay -- single entry point (thin shell; logic lives in tools\assay-cli.ps1)
rem Usage: assay help / doctor / next / check / run / gate / lint / ...
rem This file is ASCII-only on purpose: cmd.exe mangles non-ASCII rem lines.
pwsh -NoProfile -ExecutionPolicy Bypass -File "%~dp0tools\assay-cli.ps1" %*
exit /b %ERRORLEVEL%
