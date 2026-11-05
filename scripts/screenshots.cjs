// Regenerates the README screenshots of the web UI (docs/img/ui-*.png).
// Usage: start the UI first (cd web && npm run dev, or npm run build && npx vite preview),
// then: node scripts/screenshots.cjs [url]   (default http://localhost:5173/)
// Requires Playwright with Chromium (npm i -D playwright && npx playwright install chromium).
const path = require("path");
const { chromium, devices } = require("playwright");

const url = process.argv[2] || "http://localhost:5173/";
const out = path.join(__dirname, "..", "docs", "img");

async function ready(page) {
  await page.goto(url);
  await page.waitForSelector(".status.idle", { timeout: 120000 });
  await page.waitForTimeout(600); // let canvases finish drawing
}

(async () => {
  const browser = await chromium.launch();

  // Desktop: whole page (1x) and the two key panels (2x, sharp on retina screens).
  const full = await browser.newPage({ viewport: { width: 1440, height: 900 }, deviceScaleFactor: 1, colorScheme: "light" });
  await ready(full);
  await full.screenshot({ path: path.join(out, "ui-overview.png"), fullPage: true });

  const hi = await browser.newPage({ viewport: { width: 1440, height: 900 }, deviceScaleFactor: 2, colorScheme: "light" });
  await ready(hi);
  await hi.screenshot({ path: path.join(out, "ui-hero.png"), clip: { x: 0, y: 0, width: 1440, height: 900 } });
  const panels = await hi.$$("section.panel");
  const names = ["network", "potential-vs-realizable", "liquidity", "sweep", "day", "constrained"];
  for (let i = 0; i < panels.length && i < names.length; i++) {
    await panels[i].screenshot({ path: path.join(out, `ui-${names[i]}.png`) });
  }

  // Phone (reduced mode): the network panel as seen on an iPhone 13.
  const phone = await browser.newPage({ ...devices["iPhone 13"] });
  await ready(phone);
  const hero = await phone.$("section.panel.hero");
  await hero.scrollIntoViewIfNeeded();
  await phone.screenshot({ path: path.join(out, "ui-mobile.png") });

  await browser.close();
  console.log("screenshots written to", out);
})();
