@echo off
cls
echo ========================================================
echo   MAMORI — Compilando y Ejecutando Terminal Windows
echo ========================================================

:: 1. Cerrar instancia previa si esta abierta (evita error de Acceso Denegado)
taskkill /IM mamori-client.exe /F >nul 2>&1

:: 2. Asegurar que cargo este en el PATH de esta sesion
set PATH=%USERPROFILE%\.cargo\bin;%PATH%

:: 3. Compilar y lanzar la aplicacion nativa
cargo run -p mamori-client

if %ERRORLEVEL% NEQ 0 (
    echo.
    echo [ERROR] Hubo un problema al compilar. Revisa los mensajes arriba.
    pause
)
