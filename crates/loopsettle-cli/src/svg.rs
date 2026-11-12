//! Minimal SVG charts (no plotting dependency). Deterministic output.

pub struct Series {
    pub label: String,
    pub points: Vec<(f64, f64)>,
    /// draw a connecting line (else markers only)
    pub line: bool,
}

pub struct Chart {
    pub title: String,
    pub x_label: String,
    pub y_label: String,
    pub series: Vec<Series>,
    pub log_x: bool,
}

const W: f64 = 720.0;
const H: f64 = 440.0;
const ML: f64 = 70.0;
const MR: f64 = 180.0;
const MT: f64 = 40.0;
const MB: f64 = 55.0;
// Colour-blind-safe categorical palette (Okabe-Ito).
const COLORS: [&str; 8] = [
    "#0072B2", "#D55E00", "#009E73", "#CC79A7", "#E69F00", "#56B4E9", "#F0E442", "#000000",
];

fn esc(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

fn nice_ticks(lo: f64, hi: f64, n: usize) -> Vec<f64> {
    if !(hi > lo) {
        return vec![lo];
    }
    let raw = (hi - lo) / n as f64;
    let mag = 10f64.powf(raw.log10().floor());
    let step = [1.0, 2.0, 2.5, 5.0, 10.0]
        .iter()
        .map(|m| m * mag)
        .find(|s| *s >= raw)
        .unwrap_or(10.0 * mag);
    let mut t = (lo / step).ceil() * step;
    let mut v = Vec::new();
    while t <= hi + step * 1e-9 {
        v.push(if t.abs() < step * 1e-9 { 0.0 } else { t });
        t += step;
    }
    v
}

fn fmt_tick(v: f64) -> String {
    if v.abs() >= 1e6 {
        format!("{:.1}M", v / 1e6)
    } else if v.abs() >= 1e4 {
        format!("{:.0}k", v / 1e3)
    } else if v == v.trunc() {
        format!("{}", v as i64)
    } else {
        format!("{:.2}", v)
    }
}

impl Chart {
    pub fn render(&self) -> String {
        let pts = self.series.iter().flat_map(|s| s.points.iter());
        let tx = |x: f64| if self.log_x { x.max(1e-9).log10() } else { x };
        let (mut x0, mut x1, mut y0, mut y1) = (f64::MAX, f64::MIN, f64::MAX, f64::MIN);
        for &(x, y) in pts {
            x0 = x0.min(tx(x));
            x1 = x1.max(tx(x));
            y0 = y0.min(y);
            y1 = y1.max(y);
        }
        if x0 > x1 {
            (x0, x1, y0, y1) = (0.0, 1.0, 0.0, 1.0);
        }
        if y0 > 0.0 {
            y0 = 0.0;
        }
        let pad = |a: f64, b: f64| if b > a { (b - a) * 0.05 } else { 1.0 };
        let (px, py) = (pad(x0, x1), pad(y0, y1));
        let (x0, x1, y1) = (x0 - px, x1 + px, y1 + py);
        let sx = |x: f64| ML + (tx(x) - x0) / (x1 - x0) * (W - ML - MR);
        let sxt = |t: f64| ML + (t - x0) / (x1 - x0) * (W - ML - MR);
        let sy = |y: f64| H - MB - (y - y0) / (y1 - y0) * (H - MT - MB);
        let mut s = format!(
            "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 {W} {H}\" width=\"{W}\" height=\"{H}\" font-family=\"Helvetica, Arial, sans-serif\" font-size=\"12\">\n<rect width=\"100%\" height=\"100%\" fill=\"#ffffff\"/>\n<text x=\"{}\" y=\"24\" font-size=\"15\" font-weight=\"bold\">{}</text>\n",
            ML,
            esc(&self.title)
        );
        // grid + ticks
        for t in nice_ticks(y0, y1, 5) {
            let y = sy(t);
            s += &format!("<line x1=\"{ML}\" y1=\"{y:.1}\" x2=\"{:.1}\" y2=\"{y:.1}\" stroke=\"#e5e5e5\"/>\n<text x=\"{:.1}\" y=\"{:.1}\" text-anchor=\"end\" fill=\"#444\">{}</text>\n", W - MR, ML - 6.0, y + 4.0, fmt_tick(t));
        }
        let xt: Vec<f64> = if self.log_x {
            (x0.ceil() as i32..=x1.floor() as i32)
                .map(|e| e as f64)
                .collect()
        } else {
            nice_ticks(x0, x1, 6)
        };
        for t in xt {
            let x = sxt(t);
            let label = if self.log_x {
                fmt_tick(10f64.powf(t))
            } else {
                fmt_tick(t)
            };
            s += &format!("<line x1=\"{x:.1}\" y1=\"{MT}\" x2=\"{x:.1}\" y2=\"{:.1}\" stroke=\"#f0f0f0\"/>\n<text x=\"{x:.1}\" y=\"{:.1}\" text-anchor=\"middle\" fill=\"#444\">{}</text>\n", H - MB, H - MB + 16.0, label);
        }
        s += &format!(
            "<line x1=\"{ML}\" y1=\"{:.1}\" x2=\"{:.1}\" y2=\"{:.1}\" stroke=\"#333\"/>\n",
            H - MB,
            W - MR,
            H - MB
        );
        s += &format!(
            "<line x1=\"{ML}\" y1=\"{MT}\" x2=\"{ML}\" y2=\"{:.1}\" stroke=\"#333\"/>\n",
            H - MB
        );
        s += &format!(
            "<text x=\"{:.1}\" y=\"{:.1}\" text-anchor=\"middle\">{}</text>\n",
            (ML + W - MR) / 2.0,
            H - 14.0,
            esc(&self.x_label)
        );
        s += &format!("<text transform=\"translate(18,{:.1}) rotate(-90)\" text-anchor=\"middle\">{}</text>\n", (MT + H - MB) / 2.0, esc(&self.y_label));
        for (i, se) in self.series.iter().enumerate() {
            let c = COLORS[i % COLORS.len()];
            if se.line && se.points.len() > 1 {
                let d: Vec<String> = se
                    .points
                    .iter()
                    .map(|&(x, y)| format!("{:.1},{:.1}", sx(x), sy(y)))
                    .collect();
                s += &format!(
                    "<polyline fill=\"none\" stroke=\"{c}\" stroke-width=\"2\" points=\"{}\"/>\n",
                    d.join(" ")
                );
            }
            for &(x, y) in &se.points {
                s += &format!(
                    "<circle cx=\"{:.1}\" cy=\"{:.1}\" r=\"3.5\" fill=\"{c}\"/>\n",
                    sx(x),
                    sy(y)
                );
            }
            let ly = MT + 10.0 + i as f64 * 18.0;
            s += &format!(
                "<rect x=\"{:.1}\" y=\"{:.1}\" width=\"12\" height=\"12\" fill=\"{c}\"/>\n<text x=\"{:.1}\" y=\"{:.1}\">{}</text>\n",
                W - MR + 12.0,
                ly - 10.0,
                W - MR + 30.0,
                ly,
                esc(&se.label)
            );
        }
        s += "</svg>\n";
        s
    }
}

/// Horizontal bar chart: (label, value) pairs.
pub fn bars(title: &str, x_label: &str, items: &[(String, f64)]) -> String {
    let h = 60.0 + items.len() as f64 * 28.0 + 40.0;
    let max = items.iter().map(|i| i.1).fold(0.0f64, f64::max).max(1e-12);
    let (left, right) = (260.0, W - 90.0);
    let mut s = format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 {W} {h}\" width=\"{W}\" height=\"{h}\" font-family=\"Helvetica, Arial, sans-serif\" font-size=\"12\">\n<rect width=\"100%\" height=\"100%\" fill=\"#ffffff\"/>\n<text x=\"16\" y=\"26\" font-size=\"15\" font-weight=\"bold\">{}</text>\n",
        esc(title)
    );
    for (i, (label, v)) in items.iter().enumerate() {
        let y = 50.0 + i as f64 * 28.0;
        let w = (v / max) * (right - left);
        s += &format!(
            "<text x=\"{:.1}\" y=\"{:.1}\" text-anchor=\"end\">{}</text>\n<rect x=\"{left}\" y=\"{:.1}\" width=\"{w:.1}\" height=\"18\" fill=\"{}\"/>\n<text x=\"{:.1}\" y=\"{:.1}\" fill=\"#333\">{}</text>\n",
            left - 8.0,
            y + 13.0,
            esc(label),
            y,
            COLORS[0],
            left + w + 6.0,
            y + 13.0,
            fmt_tick((v * 100.0).round() / 100.0)
        );
    }
    s += &format!(
        "<text x=\"{:.1}\" y=\"{:.1}\" text-anchor=\"middle\">{}</text>\n</svg>\n",
        (left + right) / 2.0,
        h - 12.0,
        esc(x_label)
    );
    s
}
