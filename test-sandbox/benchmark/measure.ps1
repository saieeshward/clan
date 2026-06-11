# Benchmark measurement pass — deterministic metrics from snapshots + receipts.
$ErrorActionPreference = "Continue"
$env:CLAN_NO_HINTS = "1"
$base = "C:\Users\syson\Documents\Code\clan\test-sandbox\benchmark"
$flows = Get-ChildItem $base -Directory | Where-Object { $_.Name -like "flow-*" }

function CharsOfCmd([scriptblock]$sb) {
    try { $out = & $sb 2>$null | Out-String; return $out.Length } catch { return -1 }
}

$report = @()
foreach ($flow in $flows) {
    $f = @{ flow = $flow.Name; snapshots = @(); receipts = @(); final = @{} }

    # --- snapshots: per-hop artifact size + injected-context size ---
    $snapDir = Join-Path $flow.FullName "snapshots"
    if (Test-Path $snapDir) {
        foreach ($item in (Get-ChildItem $snapDir)) {
            $s = @{ name = $item.Name }
            if ($item.Extension -eq ".clan") {
                $s.artifact_bytes = $item.Length
                $s.ctx_chars_full = CharsOfCmd { clan read agent $item.FullName }
                $s.ctx_chars_skipguide = CharsOfCmd { clan read agent $item.FullName --skip-guide }
                $s.data_chars = CharsOfCmd { clan read data $item.FullName }
            } elseif ($item.PSIsContainer) {
                $files = Get-ChildItem $item.FullName -Recurse -File
                $s.artifact_bytes = ($files | Measure-Object -Sum Length).Sum
                $s.file_count = $files.Count
                # what a next hop must read = all text content
                $chars = 0
                foreach ($file in $files) { $chars += (Get-Content $file.FullName -Raw -ErrorAction SilentlyContinue).Length }
                $s.ctx_chars_full = $chars
            }
            $f.snapshots += $s
        }
    }

    # --- receipts ---
    $recDir = Join-Path $flow.FullName "receipts"
    if (Test-Path $recDir) {
        foreach ($r in (Get-ChildItem $recDir -Filter "*.json")) {
            try {
                $j = Get-Content $r.FullName -Raw | ConvertFrom-Json
                $f.receipts += @{
                    role = $j.role
                    n_commands = @($j.commands_run).Count
                    n_files_read = @($j.files_read).Count
                    n_errors = @($j.errors).Count
                    errors = @($j.errors | ForEach-Object { "$($_.what) -> $($_.recovered_by)" })
                    friction = $j.problems_and_friction
                }
            } catch { $f.receipts += @{ role = $r.Name; parse_error = $true } }
        }
    }

    # --- final artifact ---
    $work = Join-Path $flow.FullName "work"
    $finalClan = Get-ChildItem $work -Filter "final.clan" -ErrorAction SilentlyContinue
    if ($finalClan) {
        $f.final.kind = "clan"
        $f.final.bytes = $finalClan.Length
        clan validate $finalClan.FullName *> $null
        $f.final.validates = ($LASTEXITCODE -eq 0)
        $f.final.next_hop_ctx_chars = CharsOfCmd { clan read agent $finalClan.FullName }
        $f.final.next_hop_ctx_chars_skipguide = CharsOfCmd { clan read agent $finalClan.FullName --skip-guide }
        $f.final.chain_chars = CharsOfCmd { clan read chain $finalClan.FullName }
        $chain = clan read chain $finalClan.FullName 2>$null | Out-String
        $f.final.n_decisions = ([regex]::Matches($chain, "- agent:")).Count
        $f.final.has_human_view = ((clan read human $finalClan.FullName 2>$null | Out-String).Length -gt 100)
        $f.final.data_chars = CharsOfCmd { clan read data $finalClan.FullName }
    } else {
        $files = Get-ChildItem $work -Recurse -File | Where-Object { $_.Name -ne "brief.md" }
        $f.final.kind = "adhoc"
        $f.final.bytes = ($files | Measure-Object -Sum Length).Sum
        $f.final.file_count = $files.Count
        $chars = 0; foreach ($file in $files) { $chars += (Get-Content $file.FullName -Raw -ErrorAction SilentlyContinue).Length }
        $f.final.next_hop_ctx_chars = $chars
        $f.final.has_human_view = ((Get-ChildItem $work -Filter "*.html" | Measure-Object).Count -gt 0)
        $f.final.has_structured_data = ((Get-ChildItem $work -Filter "*data*.json" | Measure-Object).Count -gt 0)
        $f.final.has_decision_log = (Test-Path (Join-Path $work "decisions.log"))
        $f.final.files = @($files | ForEach-Object { $_.Name })
    }
    $report += $f
}
$report | ConvertTo-Json -Depth 6 | Set-Content -Encoding utf8 (Join-Path $base "metrics.json")
Write-Output "metrics.json written: $((Get-Item (Join-Path $base 'metrics.json')).Length) bytes"
