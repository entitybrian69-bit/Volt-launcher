/* Volt Launcher site — original animation code. */

/* Year stamp */
const y = document.getElementById("year");
if (y) y.textContent = new Date().getFullYear();

/* Reveal on scroll */
const revealer = new IntersectionObserver(
  (entries) => {
    for (const e of entries) {
      if (e.isIntersecting) {
        e.target.classList.add("shown");
        revealer.unobserve(e.target);
      }
    }
  },
  { threshold: 0.14 }
);
document.querySelectorAll(".reveal").forEach((el, i) => {
  el.style.transitionDelay = `${(i % 4) * 70}ms`;
  revealer.observe(el);
});

/* Animated counters */
const counter = new IntersectionObserver(
  (entries) => {
    for (const e of entries) {
      if (!e.isIntersecting) continue;
      counter.unobserve(e.target);
      const el = e.target;
      const target = parseInt(el.dataset.count, 10) || 0;
      const started = performance.now();
      const dur = 1400;
      const tick = (now) => {
        const t = Math.min((now - started) / dur, 1);
        const eased = 1 - Math.pow(1 - t, 3);
        el.textContent = `${Math.round(target * eased)}${target >= 30 && target < 100 ? "+" : ""}`;
        if (t < 1) requestAnimationFrame(tick);
      };
      requestAnimationFrame(tick);
    }
  },
  { threshold: 0.6 }
);
document.querySelectorAll("[data-count]").forEach((el) => counter.observe(el));

/* Card cursor glow */
document.querySelectorAll(".card").forEach((card) => {
  card.addEventListener("pointermove", (ev) => {
    const r = card.getBoundingClientRect();
    card.style.setProperty("--mx", `${ev.clientX - r.left}px`);
    card.style.setProperty("--my", `${ev.clientY - r.top}px`);
  });
});

/* Ember particle field */
const canvas = document.getElementById("particles");
if (canvas && !window.matchMedia("(prefers-reduced-motion: reduce)").matches) {
  const ctx = canvas.getContext("2d");
  let w, h, dpr;
  const motes = [];
  const COUNT = 70;

  function size() {
    dpr = Math.min(window.devicePixelRatio || 1, 2);
    w = canvas.width = Math.floor(innerWidth * dpr);
    h = canvas.height = Math.floor(innerHeight * dpr);
    canvas.style.width = innerWidth + "px";
    canvas.style.height = innerHeight + "px";
  }
  size();
  addEventListener("resize", size);

  function spawn(fresh) {
    return {
      x: Math.random() * w,
      y: fresh ? h + 10 * dpr : Math.random() * h,
      r: (0.8 + Math.random() * 2.2) * dpr,
      vy: (0.18 + Math.random() * 0.5) * dpr,
      vx: (Math.random() - 0.5) * 0.22 * dpr,
      a: 0.15 + Math.random() * 0.5,
      tw: Math.random() * Math.PI * 2,
    };
  }
  for (let i = 0; i < COUNT; i++) motes.push(spawn(false));

  (function frame(now) {
    ctx.clearRect(0, 0, w, h);
    for (let m of motes) {
      m.y -= m.vy;
      m.x += m.vx + Math.sin(now / 1800 + m.tw) * 0.12 * dpr;
      if (m.y < -12 * dpr) Object.assign(m, spawn(true));
      const flicker = 0.75 + 0.25 * Math.sin(now / 320 + m.tw * 3);
      ctx.beginPath();
      ctx.arc(m.x, m.y, m.r, 0, Math.PI * 2);
      ctx.fillStyle = `rgba(220, 38, 56, ${m.a * flicker})`;
      ctx.shadowColor = "rgba(220, 38, 56, 0.9)";
      ctx.shadowBlur = 8 * dpr;
      ctx.fill();
    }
    ctx.shadowBlur = 0;
    requestAnimationFrame(frame);
  })(performance.now());
}
