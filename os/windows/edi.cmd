@echo off
rem edi (Windows): estado | say TEXTO | reward + o - | learn | face | sleep | brain | widget | stop
set U=http://127.0.0.1:7077
if "%1"=="" goto uso
if "%1"=="status" curl -s %U%/state & goto fin
if "%1"=="say" ( for /f "tokens=1,*" %%a in ("%*") do curl -s -X POST --data "%%b" %U%/say ) & goto fin
if "%1"=="reward" curl -s -X POST --data "%2" %U%/reward & goto fin
if "%1"=="learn" curl -s %U%/learn & goto fin
if "%1"=="face" curl -s %U%/face & goto fin
if "%1"=="sleep" curl -s -X POST %U%/sleep & goto fin
if "%1"=="brain" call "%~dp0EDI.cmd" & goto fin
if "%1"=="widget" call "%~dp0EDI.cmd" --widget & goto fin
if "%1"=="stop" taskkill /im edid.exe >nul & goto fin
:uso
echo uso: edi [status^|say TEXTO^|reward +^|-^|learn^|face^|sleep^|brain^|widget^|stop]
:fin
echo.
