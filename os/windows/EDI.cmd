@echo off
rem EDI para Windows: arranca su cerebro (edid.exe) y abre su ventana 3D y el widget en Edge (modo app).
setlocal
cd /d "%~dp0"
curl -s -m 1 http://127.0.0.1:7077/state >nul 2>&1
if errorlevel 1 (
  start "EDI (cerebro)" /min "%~dp0edid.exe"
  echo Despertando a EDI...
  for /l %%i in (1,1,40) do (
    curl -s -m 1 http://127.0.0.1:7077/state >nul 2>&1 && goto listo
    timeout /t 1 /nobreak >nul
  )
)
:listo
set EDGE=msedge
if exist "%ProgramFiles(x86)%\Microsoft\Edge\Application\msedge.exe" set EDGE="%ProgramFiles(x86)%\Microsoft\Edge\Application\msedge.exe"
if "%1"=="--widget" goto widget
start "" %EDGE% --app=http://127.0.0.1:7077/ --window-size=1280,800
:widget
start "" %EDGE% --app=http://127.0.0.1:7077/widget --window-size=540,430
endlocal
