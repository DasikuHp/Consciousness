# EDI oye qué comandos usas en PowerShell: solo el NOMBRE (git, cargo…), nunca los argumentos.
# Lo instala «edi.cmd hook» (añade una línea a tu $PROFILE). Para quitarlo, borra esa línea.
$global:__ediN = -1
if (-not $global:__ediPrompt) { $global:__ediPrompt = $function:prompt }
function global:prompt {
  $ok = $?
  $h = Get-History -Count 1
  if ($h -and $h.Id -ne $global:__ediN) {
    $global:__ediN = $h.Id
    $w = $h.CommandLine.Trim() -split '\s+'
    $c = if ($w[0] -in 'sudo', 'gsudo') { $w[1] } else { $w[0] }
    if ($c) {
      $c = [IO.Path]::GetFileNameWithoutExtension($c)
      try { $null = Invoke-WebRequest -UseBasicParsing -Method Post -TimeoutSec 1 -Body "CMD:$c" -Uri "http://127.0.0.1:7077/sense?ok=$([int]$ok)" } catch {}
    }
  }
  & $global:__ediPrompt
}
