@echo off
rem Hace que EDI despierte al iniciar sesión (acceso directo en la carpeta Inicio).
powershell -NoProfile -Command "$s=(New-Object -ComObject WScript.Shell).CreateShortcut([Environment]::GetFolderPath('Startup')+'\EDI.lnk'); $s.TargetPath='%~dp0EDI.cmd'; $s.Arguments='--widget'; $s.WorkingDirectory='%~dp0'; $s.WindowStyle=7; $s.Save()"
echo EDI despertara al iniciar sesion.
