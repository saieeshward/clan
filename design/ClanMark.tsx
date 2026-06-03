// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

import { CSSProperties } from "react";

/**
 * CLAN brand mark — a lineage graph of connected nodes that also reads as a "C".
 * Built from pure circles + lines, so it stays crisp from 16px to hero.
 *
 * <ClanMark />                      → static, dual-tone (indigo → teal)
 * <ClanMark animated />             → lineage grows in, holds, recedes (loops)
 * <ClanMark tone="mono" />          → single-colour, inherits `color`
 * <ClanLogo />                      → mark + "CLAN" wordmark lockup
 */

type Tone = "duo" | "mono" | "knockout";

const NODES = [
  { x: 70, y: 24, r: 5.6, gen: 0 },
  { x: 37, y: 21, r: 5.6, gen: 1 },
  { x: 18, y: 50, r: 7.0, gen: 2 },
  { x: 37, y: 79, r: 5.6, gen: 3 },
  { x: 70, y: 76, r: 5.6, gen: 4 },
] as const;
const EDGES: [number, number][] = [[0, 1], [1, 2], [2, 3], [3, 4]];
const MAX_GEN = 4;
const SW = 2.8;

// indigo #6366f1 → teal #2dd4cf, interpolated by generation
const DUO = ["#6366f1", "#5682e9", "#489de0", "#3bb9d8", "#2dd4cf"];

function nodeFill(gen: number, tone: Tone) {
  if (tone === "duo") return DUO[gen];
  if (tone === "knockout") return "#ffffff";
  return "currentColor";
}
function edgeStroke(tone: Tone) {
  if (tone === "duo") return { stroke: "rgba(120,160,230,0.42)" };
  if (tone === "knockout") return { stroke: "#ffffff", strokeOpacity: 0.6 };
  return { stroke: "currentColor", strokeOpacity: 0.42 };
}

export function ClanMark({
  size = 32,
  tone = "duo",
  animated = false,
  style,
  title = "CLAN",
}: {
  size?: number;
  tone?: Tone;
  animated?: boolean;
  style?: CSSProperties;
  title?: string;
}) {
  const es = edgeStroke(tone);
  return (
    <svg
      width={size}
      height={size}
      viewBox="0 0 100 100"
      fill="none"
      xmlns="http://www.w3.org/2000/svg"
      className={animated ? "clan-live" : undefined}
      style={style}
      role="img"
      aria-label={title}
    >
      <title>{title}</title>
      {animated && <ClanKeyframes />}
      {EDGES.map(([a, b], i) => (
        <line
          key={i}
          className="clan-edge"
          x1={NODES[a].x}
          y1={NODES[a].y}
          x2={NODES[b].x}
          y2={NODES[b].y}
          strokeWidth={SW}
          strokeLinecap="round"
          pathLength={1}
          strokeDasharray={animated ? 1 : undefined}
          style={animated ? { animationDelay: `${NODES[b].gen * 0.13}s` } : undefined}
          {...es}
        />
      ))}
      {animated && (
        <circle
          className="clan-ring"
          cx={NODES[2].x}
          cy={NODES[2].y}
          r={NODES[2].r}
          fill="none"
          stroke="#7dd6e8"
          strokeWidth={SW * 0.7}
        />
      )}
      {NODES.map((n, i) => (
        <circle
          key={i}
          className="clan-node"
          cx={n.x}
          cy={n.y}
          r={n.r}
          fill={nodeFill(n.gen, tone)}
          style={animated ? { animationDelay: `${n.gen * 0.13}s` } : undefined}
        />
      ))}
    </svg>
  );
}

/** Mark + "CLAN" wordmark. Wordmark uses Space Grotesk; falls back to system sans. */
export function ClanLogo({
  size = 28,
  tone = "duo",
  animated = false,
  color = "#eceefb",
  style,
}: {
  size?: number;
  tone?: Tone;
  animated?: boolean;
  color?: string;
  style?: CSSProperties;
}) {
  return (
    <span style={{ display: "inline-flex", alignItems: "center", gap: size * 0.5, ...style }}>
      <ClanMark size={size * 1.45} tone={tone} animated={animated} />
      <span
        style={{
          fontFamily: '"Space Grotesk", system-ui, sans-serif',
          fontWeight: 600,
          fontSize: size,
          letterSpacing: "0.16em",
          textIndent: "0.16em",
          color,
          lineHeight: 1,
        }}
      >
        CLAN
      </span>
    </span>
  );
}

/** Injected once per animated mark; safe to duplicate (identical rules). */
function ClanKeyframes() {
  return (
    <style>{`
      .clan-live .clan-node{transform-box:fill-box;transform-origin:center;animation:clanNode var(--clanT,4.6s) cubic-bezier(.34,1.4,.5,1) infinite}
      .clan-live .clan-edge{animation:clanEdge var(--clanT,4.6s) cubic-bezier(.6,.02,.3,1) infinite}
      .clan-live .clan-ring{transform-box:fill-box;transform-origin:center;animation:clanRing var(--clanT,4.6s) ease-out infinite}
      @keyframes clanEdge{0%{stroke-dashoffset:1;opacity:0}10%{opacity:1}30%{stroke-dashoffset:0}78%{stroke-dashoffset:0;opacity:1}93%{opacity:1}100%{stroke-dashoffset:1;opacity:0}}
      @keyframes clanNode{0%{transform:scale(0);opacity:0}16%{transform:scale(1);opacity:1}82%{transform:scale(1);opacity:1}100%{transform:scale(0);opacity:0}}
      @keyframes clanRing{0%,38%{transform:scale(1);opacity:0}52%{opacity:.5}100%{transform:scale(2.6);opacity:0}}
      @media (prefers-reduced-motion:reduce){.clan-live .clan-edge,.clan-live .clan-node,.clan-live .clan-ring{animation:none}.clan-live .clan-edge{stroke-dashoffset:0;opacity:1}.clan-live .clan-node{transform:scale(1);opacity:1}.clan-live .clan-ring{display:none}}
    `}</style>
  );
}
