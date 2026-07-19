const POLL_MS = 90;

const el = {
  status: document.getElementById("status"),
  permSize: document.getElementById("perm-size"),
  permGrid: document.getElementById("perm-grid"),
  attempts: document.getElementById("stat-attempts"),
  rate: document.getElementById("stat-rate"),
  elapsed: document.getElementById("stat-elapsed"),
  nonce: document.getElementById("stat-nonce"),
  seed: document.getElementById("hash-seed"),
  ticket: document.getElementById("hash-ticket"),
  target: document.getElementById("hash-target"),
  condSorted: document.getElementById("cond-sorted"),
  condTarget: document.getElementById("cond-target"),
  overlay: document.getElementById("found-overlay"),
  permGridWrap: document.getElementById("perm-grid"),
  permBarsWrap: document.getElementById("perm-bars"),
  barsPlot: document.getElementById("bars-plot"),
  barsAxis: document.getElementById("bars-axis"),
  barTooltip: document.getElementById("bar-tooltip"),
  viewTilesBtn: document.getElementById("view-tiles-btn"),
  viewBarsBtn: document.getElementById("view-bars-btn"),
};

let tiles = [];
let bars = [];
let lastPermutation = null;
let stopped = false;
let view = "tiles";

function setView(next) {
  view = next;
  el.permGridWrap.classList.toggle("hidden", view !== "tiles");
  el.permBarsWrap.classList.toggle("hidden", view !== "bars");
  el.viewTilesBtn.classList.toggle("active", view === "tiles");
  el.viewTilesBtn.setAttribute("aria-selected", view === "tiles");
  el.viewBarsBtn.classList.toggle("active", view === "bars");
  el.viewBarsBtn.setAttribute("aria-selected", view === "bars");
}

el.viewTilesBtn.addEventListener("click", () => setView("tiles"));
el.viewBarsBtn.addEventListener("click", () => setView("bars"));

function ensureBars(n) {
  if (bars.length === n) return;
  el.barsPlot.innerHTML = "";
  el.barsAxis.innerHTML = "";
  bars = [];
  for (let i = 0; i < n; i++) {
    const bar = document.createElement("div");
    bar.className = "bar-col";
    bar.addEventListener("mouseenter", () => showTooltip(bar));
    bar.addEventListener("mousemove", (ev) => positionTooltip(ev));
    bar.addEventListener("mouseleave", hideTooltip);
    el.barsPlot.appendChild(bar);
    bars.push(bar);

    const label = document.createElement("div");
    label.className = "bar-axis-label";
    label.textContent = i;
    el.barsAxis.appendChild(label);
  }
}

function showTooltip(bar) {
  const { index, value, match } = bar.dataset;
  el.barTooltip.textContent = `index ${index} → value ${value}${match === "true" ? "  ✓ in place" : ""}`;
  el.barTooltip.classList.remove("hidden");
}

function positionTooltip(ev) {
  const wrapRect = el.permBarsWrap.getBoundingClientRect();
  el.barTooltip.style.left = `${ev.clientX - wrapRect.left}px`;
  el.barTooltip.style.top = `${ev.clientY - wrapRect.top}px`;
}

function hideTooltip() {
  el.barTooltip.classList.add("hidden");
}

function renderBars(permutation) {
  ensureBars(permutation.length);
  const n = permutation.length;
  permutation.forEach((value, index) => {
    const bar = bars[index];
    const match = value === index;
    bar.style.height = n > 1 ? `${((value + 1) / n) * 100}%` : "100%";
    bar.classList.toggle("match", match);
    bar.dataset.index = index;
    bar.dataset.value = value;
    bar.dataset.match = match;
  });
}

function ensureTiles(n) {
  if (tiles.length === n) return;
  el.permGrid.innerHTML = "";
  tiles = [];
  for (let i = 0; i < n; i++) {
    const tile = document.createElement("div");
    tile.className = "perm-tile";
    tile.textContent = "?";
    el.permGrid.appendChild(tile);
    tiles.push(tile);
  }
  lastPermutation = null;
}

function renderPermutation(permutation) {
  ensureTiles(permutation.length);
  permutation.forEach((value, index) => {
    const tile = tiles[index];
    const changed = !lastPermutation || lastPermutation[index] !== value;
    tile.textContent = value;
    tile.classList.toggle("match", value === index);
    if (changed) {
      tile.classList.add("updated");
      requestAnimationFrame(() => {
        setTimeout(() => tile.classList.remove("updated"), 100);
      });
    }
  });
  lastPermutation = permutation.slice();
}

function fmtInt(n) {
  return n.toLocaleString("en-US");
}

function render(state) {
  el.permSize.textContent = `(N = ${state.permutation_size})`;
  renderPermutation(state.permutation);
  renderBars(state.permutation);

  el.attempts.textContent = fmtInt(state.attempts);
  el.rate.innerHTML = `${fmtInt(Math.round(state.rate))}<span class="unit">/s</span>`;
  el.elapsed.innerHTML = `${(state.elapsed_ms / 1000).toFixed(1)}<span class="unit">s</span>`;
  el.nonce.textContent = fmtInt(state.nonce);

  el.seed.textContent = state.seed;
  el.ticket.textContent = state.ticket;
  el.target.textContent = state.target;

  el.condSorted.classList.toggle("pass", state.sorted);
  el.condSorted.classList.toggle("fail", !state.sorted);
  el.condTarget.classList.toggle("pass", state.meets_target);
  el.condTarget.classList.toggle("fail", !state.meets_target);

  if (state.found) {
    el.status.textContent = "FOUND";
    el.status.className = "status status-found";
    el.overlay.classList.remove("hidden");
    stopped = true;
  } else {
    el.status.textContent = "MINING…";
    el.status.className = "status status-mining";
  }
}

async function poll() {
  if (stopped) return;
  try {
    const res = await fetch(`state.json?t=${Date.now()}`, { cache: "no-store" });
    if (res.ok) {
      render(await res.json());
    }
  } catch (e) {
    // state.json not written yet; keep polling.
  } finally {
    setTimeout(poll, POLL_MS);
  }
}

setView("tiles");
poll();
