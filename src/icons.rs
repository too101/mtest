//! Button pictures, drawn as simple shapes in the spirit of the original's icon set.

use eframe::egui::{pos2, vec2, Align2, Color32, CornerRadius, FontId, Painter, Pos2, Rect, Shape, Stroke, StrokeKind};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Icon {
    Geometry,
    Brightness,
    HighVoltage,
    Web,
    Colors,
    ControlPanel,
    Help,
    Convergence,
    Focus,
    Resolution,
    Moire,
    Readability,
    Jitter,
    Sound,
    Quit,
}

const INK: Color32 = Color32::from_rgb(20, 20, 24);

fn frame(p: &Painter, r: Rect) -> Rect {
    p.rect(r, CornerRadius::same(7), Color32::WHITE, Stroke::new(1.5, INK), StrokeKind::Inside);
    r.shrink2(vec2(r.width() * 0.2, r.height() * 0.22))
}

pub fn draw(p: &Painter, r: Rect, icon: Icon) {
    let i = frame(p, r);
    let grey = Color32::from_gray(150);
    let th = (r.height() / 22.0).max(1.0);
    match icon {
        Icon::Geometry => {
            let s = i.width() * 0.18;
            let pts = vec![
                pos2(i.left() + s, i.top()),
                pos2(i.right(), i.top()),
                pos2(i.right() - s, i.bottom()),
                pos2(i.left(), i.bottom()),
            ];
            p.add(Shape::closed_line(pts, Stroke::new(th * 2.0, INK)));
        }
        Icon::Brightness => {
            let c = i.center();
            let half = Rect::from_center_size(c, vec2(i.height() * 1.1, i.height() * 0.8));
            p.rect_filled(Rect::from_min_max(half.min, pos2(c.x, half.max.y)), 0.0, grey);
            p.rect_filled(Rect::from_min_max(pos2(c.x, half.min.y), half.max), 0.0, INK);
            for k in 0..8 {
                let a = k as f32 * std::f32::consts::TAU / 8.0;
                let d = vec2(a.cos(), a.sin());
                let rr = i.height() * 0.75;
                p.line_segment([c + d * rr, c + d * (rr + i.height() * 0.2)], Stroke::new(th, grey));
            }
        }
        Icon::HighVoltage => {
            let r2 = r.shrink(2.0);
            let h = r2.height() / 3.0;
            p.rect_filled(Rect::from_min_size(pos2(r2.left(), r2.top() + h), vec2(r2.width(), h)), 0.0, Color32::WHITE);
            p.rect_filled(Rect::from_min_size(r2.min, vec2(r2.width(), h)), CornerRadius { nw: 6, ne: 6, sw: 0, se: 0 }, INK);
            p.rect_filled(
                Rect::from_min_size(pos2(r2.left(), r2.top() + 2.0 * h), vec2(r2.width(), h)),
                CornerRadius { nw: 0, ne: 0, sw: 6, se: 6 },
                INK,
            );
        }
        Icon::Web => {
            p.text(i.center(), Align2::CENTER_CENTER, "GitHub", FontId::proportional((r.height() * 0.42).min(r.width() * 0.2)), Color32::from_rgb(230, 120, 20));
        }
        Icon::Colors => {
            let w = i.width() / 3.0;
            for (k, col) in [Color32::from_rgb(230, 30, 60), Color32::from_rgb(60, 220, 40), Color32::from_rgb(40, 40, 240)].iter().enumerate() {
                p.rect_filled(Rect::from_min_size(pos2(i.left() + w * k as f32, i.top()), vec2(w, i.height())), 2.0, *col);
            }
        }
        Icon::ControlPanel => {
            let m = Rect::from_center_size(i.center(), vec2(i.height() * 1.3, i.height() * 1.05));
            p.rect(m, 2.0, Color32::from_gray(200), Stroke::new(1.0, INK), StrokeKind::Inside);
            p.rect_filled(m.shrink(m.height() * 0.15), 0.0, Color32::from_rgb(30, 30, 200));
            p.line_segment([m.right_top() + vec2(2.0, -2.0), m.left_bottom() + vec2(m.width() * 0.6, 2.0)], Stroke::new(th * 1.5, Color32::from_rgb(230, 190, 30)));
        }
        Icon::Help => {
            p.rect_filled(i, 3.0, Color32::from_gray(140));
            p.text(i.center(), Align2::CENTER_CENTER, "?", FontId::proportional(r.height() * 0.75), Color32::from_rgb(240, 130, 20));
        }
        Icon::Convergence => {
            let a = i.shrink(1.0);
            p.rect_stroke(a.translate(vec2(1.5, 0.0)), 4.0, Stroke::new(th, Color32::from_rgb(40, 210, 40)), StrokeKind::Middle);
            p.rect_stroke(a, 4.0, Stroke::new(th, Color32::from_rgb(230, 30, 60)), StrokeKind::Middle);
        }
        Icon::Focus => {
            for k in 0..4 {
                let g = 130 + k * 30;
                p.rect_stroke(i.expand(3.0 - k as f32 * 1.5), 5.0, Stroke::new(th * 2.0, Color32::from_gray(g as u8)), StrokeKind::Middle);
            }
        }
        Icon::Resolution | Icon::Moire => {
            let n = 10;
            let cw = i.width() / n as f32;
            let ch = i.height() / 6.0;
            for y in 0..6 {
                for x in 0..n {
                    let on = if icon == Icon::Resolution { (x + y) % 2 == 0 } else { (x + y / 2) % 2 == 0 };
                    let skew = if icon == Icon::Moire { ((y as f32) * 0.8).sin() * cw * 0.6 } else { 0.0 };
                    if on {
                        p.rect_filled(Rect::from_min_size(pos2(i.left() + x as f32 * cw + skew, i.top() + y as f32 * ch), vec2(cw, ch)), 0.0, INK);
                    }
                }
            }
        }
        Icon::Readability => {
            p.rect_filled(i, 2.0, INK);
            let f = FontId::monospace(i.height() * 0.42);
            p.text(pos2(i.center().x, i.top() + i.height() * 0.28), Align2::CENTER_CENTER, "ABCD", f.clone(), Color32::WHITE);
            p.text(pos2(i.center().x, i.top() + i.height() * 0.74), Align2::CENTER_CENTER, "abcd", f, Color32::WHITE);
        }
        Icon::Jitter => {
            p.rect_filled(i, 3.0, Color32::from_gray(150));
            let mut x = i.left();
            while x < i.right() {
                p.line_segment([pos2(x, i.top() - 3.0), pos2(x + 2.0, i.top() - 3.0)], Stroke::new(th, INK));
                p.line_segment([pos2(x, i.bottom() + 3.0), pos2(x + 2.0, i.bottom() + 3.0)], Stroke::new(th, INK));
                x += 4.0;
            }
        }
        Icon::Sound => {
            let c = i.center() - vec2(i.width() * 0.2, 0.0);
            let h = i.height() * 0.45;
            let body = vec![
                pos2(c.x - h * 0.9, c.y - h * 0.4),
                pos2(c.x - h * 0.3, c.y - h * 0.4),
                pos2(c.x + h * 0.4, c.y - h),
                pos2(c.x + h * 0.4, c.y + h),
                pos2(c.x - h * 0.3, c.y + h * 0.4),
                pos2(c.x - h * 0.9, c.y + h * 0.4),
            ];
            p.add(Shape::convex_polygon(body, Color32::from_gray(170), Stroke::new(th, INK)));
            for k in 1..=3 {
                arc(p, c + vec2(h * 0.4, 0.0), h * 0.35 * k as f32, Stroke::new(th, grey));
            }
        }
        Icon::Quit => {
            p.rect_filled(i.expand(2.0), 3.0, INK);
            p.text(i.center(), Align2::CENTER_CENTER, "EXIT", FontId::proportional(r.height() * 0.36), Color32::WHITE);
        }
    }
}

fn arc(p: &Painter, c: Pos2, rad: f32, s: Stroke) {
    let pts: Vec<Pos2> = (0..=12)
        .map(|k| {
            let a = -0.9 + 1.8 * k as f32 / 12.0;
            c + vec2(a.cos(), a.sin()) * rad
        })
        .collect();
    p.add(Shape::line(pts, s));
}
