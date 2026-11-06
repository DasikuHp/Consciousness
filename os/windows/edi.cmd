@echo off
rem edi (Windows): hablar ^| say TEXTO ^| + ^| - ^| teach TEXTO ^| diario ^| nota ^| notas ^| recuerdame ^| recuerda ^| ...
rem Si el texto lleva «=>» u otros símbolos, ponlo entre comillas:  edi ensena "hola => qué tal"
setlocal EnableExtensions
set "U=http://127.0.0.1:7077"
set "T=%TEMP%\edi_msg.txt"
chcp 65001 >nul
set "C=%~1"
if not defined C goto uso
rem el resto de palabras, en R
set "R="
:args
shift
if "%~1"=="" goto resto
set "R=%R% %~1"
goto args
:resto
if defined R set "R=%R:~1%"

if /i "%C%"=="hablar" goto hablar
if /i "%C%"=="status" (call :enviar "/estado" & goto fin)
if /i "%C%"=="+" (curl -s -X POST --data "+" %U%/reward >nul & echo 👍 a su ultima respuesta & goto fin)
if /i "%C%"=="-" (curl -s -X POST --data "-" %U%/reward >nul & echo 👎 a su ultima respuesta & goto fin)
if /i "%C%"=="say" (call :enviar "%R%" & goto fin)
if /i "%C%"=="teach" (call :ensenar "%R%" & goto fin)
if /i "%C%"=="teach-file" (curl -s -X POST --data-binary @"%R%" %U%/teach & goto fin)
if /i "%C%"=="ensena" (call :enviar "/enseña %R%" & goto fin)
if /i "%C%"=="diario" (call :enviar "/diario %R%" & goto fin)
if /i "%C%"=="nota" (call :enviar "/nota %R%" & goto fin)
if /i "%C%"=="notas" (call :enviar "/notas %R%" & goto fin)
if /i "%C%"=="recuerdame" (call :enviar "/recuerdame %R%" & goto fin)
if /i "%C%"=="recordatorios" (call :enviar "/recordatorios" & goto fin)
if /i "%C%"=="borra" (call :enviar "/borra %R%" & goto fin)
if /i "%C%"=="recuerda" (call :enviar "/recuerda %R%" & goto fin)
if /i "%C%"=="aprendido" (call :enviar "/aprendido" & goto fin)
if /i "%C%"=="ayuda" (call :enviar "/ayuda" & goto fin)
if /i "%C%"=="duerme" (call :enviar "/duerme" & goto fin)
if /i "%C%"=="chat" (curl -s %U%/chat & goto fin)
if /i "%C%"=="learn" (curl -s %U%/learn & goto fin)
if /i "%C%"=="face" (curl -s %U%/face & goto fin)
if /i "%C%"=="sleep" (curl -s -X POST %U%/sleep & goto fin)
if /i "%C%"=="brain" (call "%~dp0EDI.cmd" & goto fin)
if /i "%C%"=="widget" (call "%~dp0EDI.cmd" --widget & goto fin)
if /i "%C%"=="stop" (taskkill /im edid.exe >nul & goto fin)
if /i "%C%"=="hook" goto hook
:uso
echo uso: edi hablar ^| say TEXTO ^| + ^| - ^| teach TEXTO ^| teach-file ARCHIVO
echo        diario [horas] ^| nota TEXTO ^| notas [palabra] ^| recuerdame 10m^|2h^|18:30 TEXTO
echo        recordatorios ^| borra N ^| recuerda PALABRA ^| ensena "PREGUNTA => RESPUESTA" ^| aprendido
echo        status ^| chat ^| learn ^| face ^| sleep ^| brain ^| widget ^| stop ^| hook
goto fin

:enviar
rem el texto va por archivo (UTF-8) y con expansión retardada: acentos y «=>» llegan intactos
set "M=%~1"
setlocal EnableDelayedExpansion
> "%T%" echo:!M!
endlocal
curl -s -X POST --data-binary @"%T%" "%U%/say?fmt=text"
exit /b

:ensenar
set "M=%~1"
setlocal EnableDelayedExpansion
> "%T%" echo:!M!
endlocal
curl -s -X POST --data-binary @"%T%" %U%/teach >nul
echo ✏️ enseñado
exit /b

:hablar
echo Habla con EDI (/ayuda: ordenes · + / - premian o castigan · Enter vacio: salir)
:hl
set "L="
set /p "L=tu > "
if not defined L goto fin
setlocal EnableDelayedExpansion
if "!L!"=="+" (endlocal & curl -s -X POST --data "+" %U%/reward >nul & echo    👍 & goto hl)
if "!L!"=="-" (endlocal & curl -s -X POST --data "-" %U%/reward >nul & echo    👎 & goto hl)
> "%T%" echo:!L!
endlocal
curl -s -X POST --data-binary @"%T%" "%U%/say?fmt=text"
goto hl

:hook
rem que EDI oiga qué comandos usas en PowerShell (solo el nombre)
powershell -NoProfile -Command "$p=$PROFILE; if(!(Test-Path $p)){New-Item -Force -ItemType File $p|Out-Null}; $l='. ''%~dp0edi-hook.ps1'''; if(Select-String -Quiet -SimpleMatch -Path $p -Pattern 'edi-hook.ps1'){'Ya estaba instalado en '+$p}else{Add-Content $p $l; 'Instalado en '+$p}"
echo Abre una PowerShell nueva. Si dice que los scripts estan deshabilitados:
echo   Set-ExecutionPolicy -Scope CurrentUser RemoteSigned
goto fin

:fin
endlocal
