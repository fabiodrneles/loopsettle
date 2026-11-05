// Regenerates the README screenshots of the web UI in both languages:
//   docs/img/ui-*.png     (Portuguese, ?lang=pt)
//   docs/img/en/ui-*.png  (English,    ?lang=en)
// Usage: start the UI first (cd web && npm run dev, or npm run build && npx vite preview),
// then: node scripts/screenshots.cjs [url]   (default http://localhost:5173/)
// Requires Playwright with Chromium (npm i -D playwright && npx playwright install chromium).
// If ImageMagick's `convert` is on PATH, the PNGs are also compressed (256 colours).
const fs = require("fs");
const path = require("path");
const { execFileSync } = require("child_process");
const { chromium, devices } = require("playwright");

const base = (process.argv[2] || "http://localhost:5173/").replace(/\/?$/, "/");
const root = path.join(__dirname, "..", "docs", "img");
const LANGS = { pt: root, en: path.join(root, "en") };
const PANELS = ["network", "potential-vs-realizable", "liquidity", "sweep", "day", "constrained", "fit"];

async function ready(page, lang) {
  await page.goto(`${base}?lang=${lang}`);
  await page.waitForSelector(".status.idle", { timeout: 120000 });
  await page.waitForTimeout(600); // let canvases finish drawing
}

function compress(file, resize) {
  try {
    const args = [file, "-strip", "-colors", "255", "-define", "png:compression-level=9"];
    if (resize) args.splice(1, 0, "-resize", resize);
    execFileSync("convert", [...args, file]);
  } catch {
    /* ImageMagick not installed: keep the uncompressed PNG */
  }
}

(async () => {
  const browser = await chromium.launch();
  for (const [lang, out] of Object.entries(LANGS)) {
    fs.mkdirSync(out, { recursive: true });
    const files = [];

    // Desktop: whole page (1x) and each panel (2x, sharp on retina screens).
    const full = await browser.newPage({ viewport: { width: 1440, height: 900 }, deviceScaleFactor: 1, colorScheme: "light" });
    await ready(full, lang);
    files.push([path.join(out, "ui-overview.png")]);
    await full.screenshot({ path: files.at(-1)[0], fullPage: true });

    const hi = await browser.newPage({ viewport: { width: 1440, height: 900 }, deviceScaleFactor: 2, colorScheme: "light" });
    await ready(hi, lang);
    const panels = await hi.$$("section.panel");
    for (let i = 0; i < panels.length && i < PANELS.length; i++) {
      files.push([path.join(out, `ui-${PANELS[i]}.png`)]);
      await panels[i].screenshot({ path: files.at(-1)[0] });
    }

    // Phone (reduced mode): the network panel as seen on an iPhone 13, downscaled 50%.
    const phone = await browser.newPage({ ...devices["iPhone 13"] });
    await ready(phone, lang);
    await (await phone.$("section.panel.hero")).scrollIntoViewIfNeeded();
    files.push([path.join(out, "ui-mobile.png"), "50%"]);
    await phone.screenshot({ path: files.at(-1)[0] });

    for (const [f, resize] of files) compress(f, resize);
    console.log(`${lang}: ${files.length} screenshots in ${out}`);
  }
  await browser.close();
})();
