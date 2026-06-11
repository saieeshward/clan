# HEAVY happy-path measurement — aggregates across N reps.
# H1 primary: output chars/hop ratio (clan / adhoc) per rep → mean ± stdev.
# H2 primary: injected chars/hop per arm per rep → crossover hop distribution.
$ErrorActionPreference = "Continue"
$env:CLAN_NO_HINTS = "1"
$base = "$PSScriptRoot"
$clan = "clan"

function CharsOfCmd([scriptblock]$sb) {
    try { $out = & $sb 2>$null | Out-String; return $out.Length } catch { return -1 }
}
function DirChars([string]$dir) {
    if (-not (Test-Path $dir)) { return 0 }
    $c = 0
    foreach ($f in (Get-ChildItem $dir -Recurse -File)) {
        try { $c += (Get-Content $f.FullName -Raw -ErrorAction SilentlyContinue).Length } catch {}
    }
    return $c
}
function Mean([double[]]$a) { if ($a.Count -eq 0) { return 0 }; ($a | Measure-Object -Sum).Sum / $a.Count }
function Stdev([double[]]$a) {
    if ($a.Count -lt 2) { return 0 }
    $m = Mean $a
    $var = ($a | ForEach-Object { [Math]::Pow($_ - $m, 2) } | Measure-Object -Sum).Sum / ($a.Count - 1)
    return [Math]::Sqrt($var)
}

$N_REPS = 5
$result = @{ h1 = @{ reps = @() }; h2 = @{ reps = @() }; receipts = @() }

# ---- H1: output chars per hop, ratio clan/adhoc ----
$h1Ratios = @()
for ($rep = 1; $rep -le $N_REPS; $rep++) {
    $clanDir = Join-Path $base "h1-clan-rep$rep"
    $adhocDir = Join-Path $base "h1-adhoc-rep$rep"
    $repRow = @{ rep = $rep; clan_hops = @(); adhoc_hops = @() }

    foreach ($arm in @("clan","adhoc")) {
        $armDir = if ($arm -eq "clan") { $clanDir } else { $adhocDir }
        $recDir = Join-Path $armDir "receipts"
        for ($n = 1; $n -le 8; $n++) {
            $nn = "{0:00}" -f $n
            $outDir = Join-Path $recDir "hop$nn-output"
            $authored = DirChars $outDir
            $rec = Get-ChildItem $recDir -Filter "receipt-hop$nn*.json" -ErrorAction SilentlyContinue | Select-Object -First 1
            $reported = $null; $role = $null
            if ($rec) {
                try { $j = Get-Content $rec.FullName -Raw | ConvertFrom-Json; $reported = $j.output_chars; $role = $j.role } catch {}
            }
            $repRow."${arm}_hops" += @{ hop = $n; role = $role; authored_chars = $authored; reported = $reported }
        }
    }

    $clanTotal = ($repRow.clan_hops | Measure-Object -Property authored_chars -Sum).Sum
    $adhocTotal = ($repRow.adhoc_hops | Measure-Object -Property authored_chars -Sum).Sum
    $ratio = if ($adhocTotal -gt 0) { [Math]::Round($clanTotal / $adhocTotal, 4) } else { $null }
    $repRow.clan_total_chars = $clanTotal
    $repRow.adhoc_total_chars = $adhocTotal
    $repRow.ratio = $ratio
    if ($ratio -ne $null) { $h1Ratios += $ratio }

    # final artifact facts
    $doc = Join-Path $clanDir "work\doc.clan"
    if (Test-Path $doc) {
        & $clan validate $doc *> $null
        $repRow.clan_final = @{
            validates = ($LASTEXITCODE -eq 0)
            n_decisions = ([regex]::Matches((& $clan read chain $doc 2>$null | Out-String), "- agent:")).Count
            bytes = (Get-Item $doc).Length
        }
    }
    $result.h1.reps += $repRow
}
if ($h1Ratios.Count -gt 0) {
    $result.h1.ratio_mean = [Math]::Round((Mean $h1Ratios), 4)
    $result.h1.ratio_stdev = [Math]::Round((Stdev $h1Ratios), 4)
    $result.h1.win = ($result.h1.ratio_mean -le 0.50)
}

# ---- H2: injected chars per hop, crossover detection ----
$crossoverHops = @()
for ($rep = 1; $rep -le $N_REPS; $rep++) {
    $clanDir = Join-Path $base "h2-clan-rep$rep"
    $adhocDir = Join-Path $base "h2-adhoc-rep$rep"
    $repRow = @{ rep = $rep; clan = @(); adhoc = @() }

    $clanSnaps = Get-ChildItem (Join-Path $clanDir "snapshots") -Filter "hop-*.clan" -ErrorAction SilentlyContinue | Sort-Object Name
    foreach ($s in $clanSnaps) {
        $repRow.clan += @{
            hop = $s.BaseName
            injected_skipguide = CharsOfCmd { & $clan read agent $s.FullName --skip-guide }
            artifact_bytes = $s.Length
        }
    }

    $adhocSnaps = Get-ChildItem (Join-Path $adhocDir "snapshots") -Directory -ErrorAction SilentlyContinue | Sort-Object Name
    $briefChars = (Get-Content (Join-Path $adhocDir "work\brief.md") -Raw -ErrorAction SilentlyContinue).Length
    foreach ($d in $adhocSnaps) {
        $repRow.adhoc += @{
            hop = $d.Name
            injected_full = $briefChars + (DirChars $d.FullName)
            artifact_bytes = (Get-ChildItem $d.FullName -Recurse -File -ErrorAction SilentlyContinue | Measure-Object -Sum Length).Sum
        }
    }

    # Crossover: first hop where clan injected_skipguide < adhoc injected_full
    $crossover = $null
    $minHops = [Math]::Min($repRow.clan.Count, $repRow.adhoc.Count)
    for ($i = 0; $i -lt $minHops; $i++) {
        if ($repRow.clan[$i].injected_skipguide -gt 0 -and $repRow.adhoc[$i].injected_full -gt 0) {
            if ($repRow.clan[$i].injected_skipguide -lt $repRow.adhoc[$i].injected_full) {
                $crossover = $i + 1; break
            }
        }
    }
    $repRow.crossover_hop = $crossover
    if ($crossover -ne $null) { $crossoverHops += $crossover }
    $result.h2.reps += $repRow
}
if ($crossoverHops.Count -gt 0) {
    $result.h2.crossover_mean = [Math]::Round((Mean $crossoverHops), 2)
    $result.h2.crossover_stdev = [Math]::Round((Stdev $crossoverHops), 2)
    $result.h2.win = ($result.h2.crossover_mean -le 10)
}

# ---- receipts summary ----
foreach ($arm in @("h1-clan","h1-adhoc","h2-clan","h2-adhoc")) {
    for ($rep = 1; $rep -le $N_REPS; $rep++) {
        $recDir = Join-Path $base "${arm}-rep$rep\receipts"
        if (Test-Path $recDir) {
            foreach ($r in (Get-ChildItem $recDir -Filter "receipt-*.json" -ErrorAction SilentlyContinue)) {
                try {
                    $j = Get-Content $r.FullName -Raw | ConvertFrom-Json
                    $result.receipts += @{ arm = $arm; rep = $rep; role = $j.role; hop = $j.hop; n_errors = @($j.errors).Count; friction = $j.problems_and_friction }
                } catch { $result.receipts += @{ arm = $arm; rep = $rep; file = $r.Name; parse_error = $true } }
            }
        }
    }
}

$out = Join-Path $base "metrics-heavy.json"
$result | ConvertTo-Json -Depth 8 | Set-Content -Encoding utf8 $out
Write-Host "metrics-heavy.json written ($((Get-Item $out).Length) bytes)"
Write-Host "H1 ratio: mean=$($result.h1.ratio_mean) stdev=$($result.h1.ratio_stdev) win=$($result.h1.win)"
Write-Host "H2 crossover: mean=$($result.h2.crossover_mean) stdev=$($result.h2.crossover_stdev) win=$($result.h2.win)"
