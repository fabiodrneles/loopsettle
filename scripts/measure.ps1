# Runs a command and reports wall time and peak RAM (sum of working sets of
# cargo/rustc/link/loopsettle/node processes, sampled every 200 ms).
# Usage: powershell -File scripts/measure.ps1 "cargo test --workspace"
param([Parameter(Mandatory=$true)][string]$Cmd)
$names = @('cargo','rustc','link','loopsettle','node','wasm-pack','wasm-bindgen')
$sw = [Diagnostics.Stopwatch]::StartNew()
$p = Start-Process -FilePath "cmd.exe" -ArgumentList "/c $Cmd" -NoNewWindow -PassThru
$peak = 0
while (-not $p.HasExited) {
  $sum = (Get-Process -Name $names -ErrorAction SilentlyContinue | Measure-Object WorkingSet64 -Sum).Sum
  if ($sum -gt $peak) { $peak = $sum }
  Start-Sleep -Milliseconds 200
}
$p.WaitForExit()
$sw.Stop()
"MEASURE cmd=""$Cmd"" exit=$($p.ExitCode) wall_s={0:N1} peak_ram_mb={1:N0}" -f $sw.Elapsed.TotalSeconds, ($peak/1MB)
