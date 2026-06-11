# LITE happy-path measurement — deterministic metrics from snapshots + receipts.
# H1 primary metric: output chars/hop (chars the agent authored, from receipts/hopNN-output/).
# H2 primary metric: injected chars/hop (what each hop must read to orient).
$ErrorActionPreference = "Continue"
$env:CLAN_NO_HINTS = "1"
$base = "C:\Users\syson\Documents\Code\clan\test-sandbox\happy\lite"

function CharsOfCmd([scriptblock]$sb) {
    try { $out = & $sb 2>$null | Out-String; return $out.Length } catch { return -1 }
}
function DirChars([string]$dir) {
    if (-not (Test-Path $dir)) { return 0 }
    $c = 0
    foreach ($f in (Get-ChildItem $dir -Recurse -File)) {
        $c += (Get-Content $f.FullName -Raw -ErrorAction SilentlyContinue).Length
    }
    return $c
}

$result = @{ h1 = @{ clan = @(); adhoc = @() }; h2 = @{ clan = @(); adhoc = @() } }

# ---------- H1: output chars per hop (authored payload bytes) ----------
foreach ($arm in @("clan","adhoc")) {
    $armDir = Join-Path $base "h1-$arm"
    $recDir = Join-Path $armDir "receipts"
    for ($n = 1; $n -le 8; $n++) {
        $nn = "{0:00}" -f $n
        $outDir = Join-Path $recDir "hop$nn-output"
        $authored = DirChars $outDir
        # receipt self-report (cross-check)
        $rec = Get-ChildItem $recDir -Filter "receipt-hop$nn*.json" -ErrorAction SilentlyContinue | Select-Object -First 1
        $reported = $null; $role = $null
        if ($rec) {
            try { $j = Get-Content $rec.FullName -Raw | ConvertFrom-Json; $reported = $j.output_chars; $role = $j.role } catch {}
        }
        $result.h1.$arm += @{ hop = $n; role = $role; authored_chars = $authored; reported_output_chars = $reported }
    }
    # final fidelity facts
    if ($arm -eq "clan") {
        $doc = Join-Path $armDir "work\doc.clan"
        if (Test-Path $doc) {
            clan validate $doc *> $null
            $result.h1_clan_final = @{
                validates = ($LASTEXITCODE -eq 0)
                data_chars = CharsOfCmd { clan read data $doc }
                human_chars = CharsOfCmd { clan read human $doc }
                n_decisions = ([regex]::Matches((clan read chain $doc 2>$null | Out-String), "- agent:")).Count
                bytes = (Get-Item $doc).Length
            }
        }
    } else {
        $work = Join-Path $armDir "work"
        $result.h1_adhoc_final = @{
            report_chars = (Get-Content (Join-Path $work "report.html") -Raw -ErrorAction SilentlyContinue).Length
            data_chars = (Get-Content (Join-Path $work "data.json") -Raw -ErrorAction SilentlyContinue).Length
            log_lines = (Get-Content (Join-Path $work "decisions.log") -ErrorAction SilentlyContinue | Measure-Object -Line).Lines
        }
    }
}

# ---------- H2: injected chars per hop (orientation cost) ----------
# CLAN: clan read agent --skip-guide on the snapshot the NEXT hop would read.
# adhoc: sum of brief.md + findings.md + handoff.md at that snapshot.
$clanSnaps = Get-ChildItem (Join-Path $base "h2-clan\snapshots") -Filter "hop-*.clan" -ErrorAction SilentlyContinue | Sort-Object Name
foreach ($s in $clanSnaps) {
    $result.h2.clan += @{
        hop = $s.BaseName
        injected_skipguide = CharsOfCmd { clan read agent $s.FullName --skip-guide }
        injected_full = CharsOfCmd { clan read agent $s.FullName }
        data_chars = CharsOfCmd { clan read data $s.FullName }
        artifact_bytes = $s.Length
    }
}
$adhocSnaps = Get-ChildItem (Join-Path $base "h2-adhoc\snapshots") -Directory -ErrorAction SilentlyContinue | Sort-Object Name
foreach ($d in $adhocSnaps) {
    # what a next hop must read = brief (constant) + findings + handoff at this snapshot
    $briefChars = (Get-Content (Join-Path $base "h2-adhoc\work\brief.md") -Raw -ErrorAction SilentlyContinue).Length
    $chars = $briefChars + (DirChars $d.FullName)
    $result.h2.adhoc += @{ hop = $d.Name; injected_full = $chars; artifact_bytes = (Get-ChildItem $d.FullName -Recurse -File | Measure-Object -Sum Length).Sum }
}

# ---------- receipts summary (errors / friction across all arms) ----------
$result.receipts = @()
foreach ($flowArm in @("h1-clan","h1-adhoc","h2-clan","h2-adhoc")) {
    $recDir = Join-Path $base "$flowArm\receipts"
    if (Test-Path $recDir) {
        foreach ($r in (Get-ChildItem $recDir -Filter "receipt-*.json")) {
            try {
                $j = Get-Content $r.FullName -Raw | ConvertFrom-Json
                $result.receipts += @{
                    arm = $flowArm; role = $j.role; hop = $j.hop
                    n_errors = @($j.errors).Count
                    friction = $j.problems_and_friction
                    n_files_read = @($j.files_read).Count
                }
            } catch { $result.receipts += @{ arm = $flowArm; file = $r.Name; parse_error = $true } }
        }
    }
}

$result | ConvertTo-Json -Depth 8 | Set-Content -Encoding utf8 (Join-Path $base "metrics-lite.json")
Write-Output "metrics-lite.json written: $((Get-Item (Join-Path $base 'metrics-lite.json')).Length) bytes"
