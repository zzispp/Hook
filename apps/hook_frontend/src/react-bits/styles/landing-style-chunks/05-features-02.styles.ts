export const landingStyles05Features02 = String.raw`
.ln-feat-orbit-center {
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  width: 54px;
  height: 54px;
  border-radius: 50%;
  background: #1a1520;
  border: 1px solid rgba(255, 255, 255, 0.1);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 3;
}

.ln-feat-orbit-center img {
  width: 28px;
  height: 28px;
  opacity: 0.7;
}

.ln-feat-orbit-ring {
  position: absolute;
  top: 50%;
  left: 50%;
  border-radius: 50%;
  border: 1px solid rgba(255, 255, 255, 0.05);
}

.ln-feat-orbit-ring--1 {
  width: 140px;
  height: 140px;
  margin: -70px 0 0 -70px;
  animation: spinCW 20s linear infinite;
}

.ln-feat-orbit-ring--2 {
  width: 260px;
  height: 260px;
  margin: -130px 0 0 -130px;
  animation: spinCCW 30s linear infinite;
}

.ln-feat-orbit-node {
  position: absolute;
  width: 42px;
  height: 42px;
  border-radius: 50%;
  background: #1a1520;
  border: 1px solid rgba(255, 255, 255, 0.08);
  display: flex;
  align-items: center;
  justify-content: center;
  color: rgba(255, 255, 255, 0.5);
  z-index: 2;
}

/* Ring 1 icon positions */
.ln-feat-orbit-ring--1 .ln-feat-orbit-node--top {
  top: -21px;
  left: calc(50% - 21px);
}

.ln-feat-orbit-ring--1 .ln-feat-orbit-node--right {
  right: -21px;
  top: calc(50% - 21px);
}

.ln-feat-orbit-ring--1 .ln-feat-orbit-node--bottom {
  bottom: -21px;
  left: calc(50% - 21px);
}

.ln-feat-orbit-ring--1 .ln-feat-orbit-node--left {
  left: -21px;
  top: calc(50% - 21px);
}

/* Ring 1 spins CW → icons counter-rotate CCW */
.ln-feat-orbit-ring--1 .ln-feat-orbit-node svg {
  animation: spinCCW 20s linear infinite;
}

/* Ring 2 icon positions — 8 nodes at 45° intervals */
.ln-feat-orbit-ring--2 .ln-feat-orbit-node--top {
  top: -21px;
  left: calc(50% - 21px);
}

.ln-feat-orbit-ring--2 .ln-feat-orbit-node--tr {
  top: calc(14.65% - 21px);
  left: calc(85.35% - 21px);
}

.ln-feat-orbit-ring--2 .ln-feat-orbit-node--right {
  top: calc(50% - 21px);
  left: calc(100% - 21px);
}

.ln-feat-orbit-ring--2 .ln-feat-orbit-node--br {
  top: calc(85.35% - 21px);
  left: calc(85.35% - 21px);
}

.ln-feat-orbit-ring--2 .ln-feat-orbit-node--bottom {
  top: calc(100% - 21px);
  left: calc(50% - 21px);
}

.ln-feat-orbit-ring--2 .ln-feat-orbit-node--bl {
  top: calc(85.35% - 21px);
  left: calc(14.65% - 21px);
}

.ln-feat-orbit-ring--2 .ln-feat-orbit-node--left {
  top: calc(50% - 21px);
  left: -21px;
}

.ln-feat-orbit-ring--2 .ln-feat-orbit-node--tl {
  top: calc(14.65% - 21px);
  left: calc(14.65% - 21px);
}

/* Ring 2 spins CCW → icons counter-rotate CW */
.ln-feat-orbit-ring--2 .ln-feat-orbit-node svg {
  animation: spinCW 30s linear infinite;
}

@keyframes spinCW {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}

@keyframes spinCCW {
  from { transform: rotate(0deg); }
  to { transform: rotate(-360deg); }
}

/* ═══════════════════════════════════════
   4. Variant Rows
   ═══════════════════════════════════════ */

.ln-feat-vrows {
  display: flex;
  flex-direction: column;
  gap: 6px;
  width: 100%;
  height: 100%;
  padding: 14px;
  box-sizing: border-box;
}

.ln-feat-vrow {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 0 14px;
  border-radius: 10px;
  background: rgba(255, 255, 255, 0.02);
  border: 1px solid rgba(255, 255, 255, 0.05);
  flex: 1;
}

.ln-feat-vrow-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  flex-shrink: 0;
}

.ln-feat-vrow-label {
  font-family: 'Geist Mono', monospace;
  font-size: 12px;
  color: rgba(255, 255, 255, 0.7);
  white-space: nowrap;
  flex-shrink: 0;
  min-width: 80px;
}

.ln-feat-vrow-bars {
  display: flex;
  align-items: center;
  gap: 5px;
  flex: 1;
  min-width: 0;
}

.ln-feat-vrow-bar {
  height: 4px;
  border-radius: 2px;
  background: rgba(255, 255, 255, 0.06);
}

/* ═══════════════════════════════════════
   5. AI Chat
   ═══════════════════════════════════════ */

.ln-feat-aichat {
  width: 100%;
  height: 100%;
  padding: 14px;
  box-sizing: border-box;
}

.ln-feat-aichat-inner {
  width: 100%;
  height: 100%;
  display: flex;
  flex-direction: column;
  background: rgba(255, 255, 255, 0.005);
  border: 1px solid rgba(255, 255, 255, 0.06);
  border-radius: 10px;
  overflow: hidden;
}

/* Header bar */
.ln-feat-aichat-head {
  display: flex;
`;
