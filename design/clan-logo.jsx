// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

// clan-logo.jsx — CLAN brand mark system
// Exports: ClanMark, ClanLockup, SCHEMES, VARIANTS  (assigned to window)
//
// Concept: a CLAN is a lineage graph — a clutch of connected agent/document
// nodes descending from a root. Every mark is built ONLY from circles (nodes)
// and lines (edges) so it stays crisp from 16px favicon to a hero.

// ── one-time animation stylesheet ───────────────────────────────────────────
if (typeof document !== 'undefined' && !document.getElementById('clan-anim')) {
  const s = document.createElement('style');
  s.id = 'clan-anim';
  s.textContent = `
  .clan-node{transform-box:fill-box;transform-origin:center}
  .clan-live .clan-edge{animation:clanEdge var(--T,4.6s) cubic-bezier(.6,.02,.3,1) infinite}
  .clan-live .clan-node{animation:clanNode var(--T,4.6s) cubic-bezier(.34,1.4,.5,1) infinite}
  .clan-live .clan-ring{animation:clanRing var(--T,4.6s) ease-out infinite}
  @keyframes clanEdge{
    0%{stroke-dashoffset:1;opacity:0}
    10%{opacity:1}
    30%{stroke-dashoffset:0}
    78%{stroke-dashoffset:0;opacity:1}
    93%{opacity:1}
    100%{stroke-dashoffset:1;opacity:0}
  }
  @keyframes clanNode{
    0%{transform:scale(0);opacity:0}
    16%{transform:scale(1);opacity:1}
    82%{transform:scale(1);opacity:1}
    100%{transform:scale(0);opacity:0}
  }
  @keyframes clanRing{
    0%,38%{transform:scale(1);opacity:0}
    52%{opacity:.5}
    100%{transform:scale(2.6);opacity:0}
  }
  @media (prefers-reduced-motion: reduce){
    .clan-live .clan-edge,.clan-live .clan-node,.clan-live .clan-ring{animation:none}
    .clan-live .clan-edge{stroke-dashoffset:0;opacity:1}
    .clan-live .clan-node{transform:scale(1);opacity:1}
    .clan-live .clan-ring{opacity:0}
  }`;
  document.head.appendChild(s);
}

// ── geometry: nodes {x,y,r,gen}, edges [i,j], rings:[nodeIdx] for live pulse ──
const VARIANTS = {
  lineage: {
    label: 'Lineage',
    blurb: 'A root branches into descendants — provenance flowing left → right.',
    nodes: [
      { x: 16, y: 50, r: 7.5, gen: 0 },
      { x: 48, y: 30, r: 5.5, gen: 1 },
      { x: 48, y: 70, r: 5.5, gen: 1 },
      { x: 82, y: 30, r: 4.6, gen: 2 },
      { x: 82, y: 70, r: 4.6, gen: 2 },
    ],
    edges: [[0, 1], [0, 2], [1, 3], [2, 4]],
    rings: [0],
  },
  constellation: {
    label: 'Constellation',
    blurb: 'Nodes trace a C — a clan that doubles as the monogram.',
    nodes: [
      { x: 70, y: 24, r: 5.6, gen: 0 },
      { x: 37, y: 21, r: 5.6, gen: 1 },
      { x: 18, y: 50, r: 7, gen: 2 },
      { x: 37, y: 79, r: 5.6, gen: 3 },
      { x: 70, y: 76, r: 5.6, gen: 4 },
    ],
    edges: [[0, 1], [1, 2], [2, 3], [3, 4]],
    rings: [2],
  },
  cluster: {
    label: 'Cluster',
    blurb: 'A clan gathered around a shared core — kinship made visible.',
    nodes: [
      { x: 50, y: 50, r: 8, gen: 0 },
      { x: 50, y: 16, r: 5, gen: 1 },
      { x: 80, y: 39, r: 5, gen: 1 },
      { x: 68, y: 81, r: 5, gen: 1 },
      { x: 32, y: 81, r: 5, gen: 1 },
      { x: 20, y: 39, r: 5, gen: 1 },
    ],
    edges: [[0, 1], [0, 2], [0, 3], [0, 4], [0, 5]],
    rings: [0],
  },
  weave: {
    label: 'Weave',
    blurb: 'Two parents merge, then fork — the version DAG with a delta.',
    nodes: [
      { x: 20, y: 26, r: 5.6, gen: 0 },
      { x: 20, y: 74, r: 5.6, gen: 0 },
      { x: 50, y: 50, r: 7.5, gen: 1 },
      { x: 82, y: 30, r: 5, gen: 2 },
      { x: 82, y: 70, r: 5, gen: 2 },
    ],
    edges: [[0, 2], [1, 2], [2, 3], [2, 4]],
    rings: [2],
  },
};

// ── colour schemes ───────────────────────────────────────────────────────────
// Accents picked in oklch at matched L/C, hue-varied (indigo 277° → teal 200°).
const SCHEMES = {
  indigoDark: {
    label: 'Indigo · Dark', bg: '#0f1117', word: '#eceefb', tag: '#6b7283',
    node: '#6366f1', node2: '#6366f1', edge: 'rgba(129,140,248,.45)', ring: '#818cf8',
  },
  duoDark: {
    label: 'Dual-tone · Dark', bg: '#0f1117', word: '#eceefb', tag: '#6b7283',
    node: '#6366f1', node2: '#2dd4cf', edge: 'rgba(120,160,230,.42)', ring: '#7dd6e8',
  },
  mono: {
    label: 'Monochrome · Light', bg: '#f5f4ef', word: '#15171e', tag: '#9a988e',
    node: '#15171e', node2: '#15171e', edge: 'rgba(21,23,30,.4)', ring: '#15171e',
  },
  knockout: {
    label: 'Knockout · Indigo', bg: '#6366f1', word: '#ffffff', tag: 'rgba(255,255,255,.72)',
    node: '#ffffff', node2: '#ffffff', edge: 'rgba(255,255,255,.6)', ring: '#ffffff',
  },
};

function _hex(c) {
  const m = c.replace('#', '');
  return [parseInt(m.slice(0, 2), 16), parseInt(m.slice(2, 4), 16), parseInt(m.slice(4, 6), 16)];
}
function _mix(a, b, t) {
  const A = _hex(a), B = _hex(b);
  const r = Math.round(A[0] + (B[0] - A[0]) * t);
  const g = Math.round(A[1] + (B[1] - A[1]) * t);
  const bl = Math.round(A[2] + (B[2] - A[2]) * t);
  return `rgb(${r},${g},${bl})`;
}

// ── the mark ─────────────────────────────────────────────────────────────────
function ClanMark({ variant = 'constellation', scheme = 'indigoDark', size = 96, live = false, style }) {
  const V = VARIANTS[variant] || VARIANTS.constellation;
  const S = typeof scheme === 'string' ? (SCHEMES[scheme] || SCHEMES.indigoDark) : scheme;
  const maxGen = Math.max(...V.nodes.map((n) => n.gen)) || 1;
  const nodeColor = (n) => _mix(S.node, S.node2, maxGen ? n.gen / maxGen : 0);
  const cls = live ? 'clan-live' : '';
  const stroke = Math.max(2.4, size * 0.028);

  return (
    <svg className={cls} width={size} height={size} viewBox="0 0 100 100" style={style}
      fill="none" xmlns="http://www.w3.org/2000/svg">
      {/* edges behind nodes */}
      <g>
        {V.edges.map(([a, b], i) => (
          <line key={i} className="clan-edge"
            x1={V.nodes[a].x} y1={V.nodes[a].y} x2={V.nodes[b].x} y2={V.nodes[b].y}
            stroke={S.edge} strokeWidth={stroke} strokeLinecap="round"
            pathLength="1" strokeDasharray="1"
            style={live ? { animationDelay: `${V.nodes[b].gen * 0.13}s` } : { strokeDashoffset: 0 }} />
        ))}
      </g>
      {/* live pulse rings */}
      {live && V.rings.map((ri, i) => (
        <circle key={'r' + i} className="clan-ring node-anchor"
          cx={V.nodes[ri].x} cy={V.nodes[ri].y} r={V.nodes[ri].r}
          fill="none" stroke={S.ring} strokeWidth={stroke * 0.7}
          style={{ transformBox: 'fill-box', transformOrigin: 'center', animationDelay: `${i * 0.4}s` }} />
      ))}
      {/* nodes */}
      <g>
        {V.nodes.map((n, i) => (
          <circle key={i} className="clan-node" cx={n.x} cy={n.y} r={n.r}
            fill={nodeColor(n)}
            style={live ? { animationDelay: `${n.gen * 0.13}s` } : undefined} />
        ))}
      </g>
    </svg>
  );
}

// ── lockup (mark + wordmark) ──────────────────────────────────────────────────
function ClanLockup({
  variant = 'constellation', scheme = 'indigoDark', layout = 'horizontal',
  size = 88, live = false, tagline = true, style,
}) {
  const S = typeof scheme === 'string' ? (SCHEMES[scheme] || SCHEMES.indigoDark) : scheme;
  const stacked = layout === 'stacked';
  const wordSize = size * 0.62;
  const word = (
    <div style={{ display: 'flex', flexDirection: 'column', gap: size * 0.06, alignItems: stacked ? 'center' : 'flex-start' }}>
      <div style={{
        fontFamily: '"Space Grotesk", system-ui, sans-serif', fontWeight: 600,
        fontSize: wordSize, lineHeight: 0.9, letterSpacing: '0.16em', textIndent: '0.16em',
        color: S.word,
      }}>CLAN</div>
      {tagline && (
        <div style={{
          fontFamily: '"Space Mono", ui-monospace, monospace', fontWeight: 400,
          fontSize: Math.max(8.5, wordSize * 0.205), letterSpacing: '0.22em',
          color: S.tag, textTransform: 'uppercase', whiteSpace: 'nowrap',
        }}>Context · Live · Agent · Notation</div>
      )}
    </div>
  );
  return (
    <div style={{
      display: 'flex', flexDirection: stacked ? 'column' : 'row',
      alignItems: 'center', gap: stacked ? size * 0.22 : size * 0.34,
      ...style,
    }}>
      <ClanMark variant={variant} scheme={S} size={size} live={live} />
      {word}
    </div>
  );
}

Object.assign(window, { ClanMark, ClanLockup, SCHEMES, VARIANTS });
