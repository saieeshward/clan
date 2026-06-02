#!/usr/bin/env bash
# Run the FitLux CEO scenario test.
set -euo pipefail
CLAN="./target/release/clan"
SANDBOX="$(dirname "$0")"

echo "=== FitLux Q3 Campaign — CLAN Pipeline Test ==="

# Stage 1: Create the campaign document
$CLAN create \
  --title "FitLux Q3 AirFlex Campaign" \
  --brief "Plan, execute, and report on the Q3 Google + Meta campaign for FitLux activewear. Budget: €8,000. Target: 150 sales minimum." \
  "$SANDBOX/fitlux_q3.clan"

# Stage 1: Pack the media plan
$CLAN pack --output "$SANDBOX/fitlux_s1_plan.clan" \
  --delta "Planning agent: computed media plan from client brief" \
  "$SANDBOX/fitlux_q3.clan" "$SANDBOX/stage1_campaign_plan.json"

# Stage 2: Pack the copy
$CLAN pack --output "$SANDBOX/fitlux_s2_copy.clan" \
  --delta "Copy agent: wrote Google + Meta ad copy" \
  "$SANDBOX/fitlux_s1_plan.clan" "$SANDBOX/stage2_copy.json"

# Stage 3 (simulated): Apply Aoife's human patch to meta_primary_text
echo "=== [Stage 3] Simulating Aoife's edit to Meta copy ===" 
cat > "$SANDBOX/aoife_patch.html" << 'EOF'
---
mode: patch-html
patch_selector: "[data-adf-id='meta_primary_text']"
patch_action: replace
---
Your gym deserves better than ordinary. FitLux AirFlex leggings are engineered for performance without sacrificing style. Shop the collection →
EOF
$CLAN patch-html "$SANDBOX/fitlux_s2_copy.clan" "$SANDBOX/aoife_patch.html" \
  --delta "Human edit by Aoife: replaced 'basic' with 'ordinary' per FitLux brand voice"

# Stage 4: Pack the performance report
$CLAN pack --output "$SANDBOX/fitlux_s4_report.clan" \
  --delta "Reporting agent: generated Q3 end-of-campaign report with variance analysis" \
  "$SANDBOX/fitlux_s2_copy.clan" "$SANDBOX/stage4_actuals.json"

# Final validation
echo "=== Final Validation ==="
$CLAN validate --strict "$SANDBOX/fitlux_s4_report.clan"
$CLAN info "$SANDBOX/fitlux_s4_report.clan"
$CLAN read chain "$SANDBOX/fitlux_s4_report.clan"

echo ""
echo "=== CEO SCORECARD ==="
echo "1. Numbers survived all hops:    $(grep -c 'planned_conversions' <<< $($CLAN read data "$SANDBOX/fitlux_s4_report.clan") && echo PASS || echo FAIL)"
echo "2. Audit chain present:          $($CLAN read chain "$SANDBOX/fitlux_s4_report.clan" | grep -c 'agent:' ) decision entries"
echo "3. Human patch preserved:        $($CLAN read agent "$SANDBOX/fitlux_s4_report.clan" | grep -c 'ordinary' && echo PASS || echo FAIL)"
echo "4. Validate --strict:            PASS (would have exited above if failed)"
