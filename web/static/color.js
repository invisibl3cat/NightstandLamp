const XHAIR_THICKNESS = 4;
const XHAIR_HALF_THICKNESS = XHAIR_THICKNESS / 2;
const XHAIR_LENGTH = 6;
const XHAIR_OFFSET = 2;
const MAX_CTRL_HEIGHT_PX = 500;

let current_h = 180;
let current_s = 0.5;
let current_v = 1;

let current_r = hsv2rgb(current_h, current_s, current_v).r;
let current_g = hsv2rgb(current_h, current_s, current_v).g;
let current_b = hsv2rgb(current_h, current_s, current_v).b;

let activeControl = null;

function clamp(v, min, max) {
    return v < min
        ? min
        : v > max
            ? max
            : v;
}


function hsv2rgb(h, s, v) {
    const f = (n, k = (n + h / 60) % 6) => v - v * s * Math.max(Math.min(k, 4 - k, 1), 0);
    return { r: f(5) * 255, g: f(3) * 255, b: f(1) * 255 };
}

function rgb2hsv(r, g, b) {
    const rabs = r / 255;
    const gabs = g / 255;
    const babs = b / 255;
    const v = Math.max(rabs, gabs, babs);
    const diff = v - Math.min(rabs, gabs, babs);
    const diffc = (c) => (v - c) / 6 / diff + 1 / 2;

    let h = 0;
    let s = 0;

    if (diff !== 0) {
        s = diff / v;
        const rr = diffc(rabs);
        const gg = diffc(gabs);
        const bb = diffc(babs);

        if (rabs === v) {
            h = bb - gg;
        } else if (gabs === v) {
            h = (1 / 3) + rr - bb;
        } else if (babs === v) {
            h = (2 / 3) + gg - rr;
        }

        if (h < 0) {
            h += 1;
        } else if (h > 1) {
            h -= 1;
        }
    }

    return { h: h * 360, s, v };
}

function drawColorSpace() {
    const hueStep = 360 / color_space.width;
    const satStep = 1 / color_space.height;

    const ctx = color_space.getContext('2d');

    for (let hue = 0, x = 0; hue < 360; hue += hueStep, x++) {
        for (let sat = 1, y = 0; sat >= 0; sat -= satStep, y++) {
            const { r, g, b } = hsv2rgb(hue, sat, 1.0);

            ctx.fillStyle = `rgb(${r}, ${g}, ${b})`;
            ctx.fillRect(x, y, 1, 1);
        }
    }
}

function drawColorSpaceCrosshair() {
    const hueStepInv = color_space.width / 360;
    const satStepInv = color_space.height;

    const ctx = color_space.getContext('2d');

    const x = current_h * hueStepInv;
    const y = (1 - current_s) * satStepInv;

    ctx.fillStyle = '#000000';

    ctx.fillRect(x - XHAIR_LENGTH - XHAIR_OFFSET, y - XHAIR_HALF_THICKNESS,        XHAIR_LENGTH,    XHAIR_THICKNESS);
    ctx.fillRect(x - XHAIR_HALF_THICKNESS,        y - XHAIR_LENGTH - XHAIR_OFFSET, XHAIR_THICKNESS, XHAIR_LENGTH);
    ctx.fillRect(x + XHAIR_OFFSET,                y - XHAIR_HALF_THICKNESS,        XHAIR_LENGTH,    XHAIR_THICKNESS);
    ctx.fillRect(x - XHAIR_HALF_THICKNESS,        y + XHAIR_OFFSET,                XHAIR_THICKNESS, XHAIR_LENGTH);
}

function drawValueBar() {
    const valStep = 1 / value_bar.height;
    const width = value_bar.width;

    const ctx = value_bar.getContext('2d');

    for (let val = 1, y = 0; val >= 0; val -= valStep, y++) {
        const { h, s } = rgb2hsv(current_r, current_g, current_b);
        const { r, g, b } = hsv2rgb(h, s, val);

        ctx.fillStyle = `rgb(${r}, ${g}, ${b})`;
        ctx.fillRect(0, y, width, 1);
    }
}

function drawValueBarCrosshair() {
    const THICKNESS = 4;
    const HALF_THICKNESS = THICKNESS / 2;

    const valStep = value_bar.height;

    const ctx = value_bar.getContext('2d');

    const y = (1 - current_v) * valStep;
    const half_w = value_bar.width / 2;

    ctx.fillStyle = '#000000';
    ctx.fillRect(0, y - HALF_THICKNESS, half_w, THICKNESS);
    ctx.fillStyle = '#FFFFFF';
    ctx.fillRect(half_w + 1, y - HALF_THICKNESS, half_w - 1, THICKNESS);
}

function eraseColorSpaceCrosshair(h, s) {
    const hueStepInv = color_space.width / 360;
    const satStepInv = color_space.height;
    const hueStep = 360 / color_space.width;
    const satStep = 1 / color_space.height;

    const fx = Math.floor(h * hueStepInv - XHAIR_LENGTH - XHAIR_OFFSET);
    const fy = Math.floor((1 - s) * satStepInv - XHAIR_LENGTH - XHAIR_OFFSET);
    const tx = Math.ceil(fx + (XHAIR_LENGTH + XHAIR_OFFSET) * 2);
    const ty = Math.ceil(fy + (XHAIR_LENGTH + XHAIR_OFFSET) * 2);

    const ctx = color_space.getContext('2d');
    for (let hue = fx * hueStep, x = fx; x <= tx; x++, hue += hueStep) {
        for (let sat = (color_space.height - fy) * satStep, y = fy; y <= ty; y++, sat -= satStep) {
            const { r, g, b } = hsv2rgb(hue, sat, 1);

            ctx.fillStyle = `rgb(${r}, ${g}, ${b})`;
            ctx.fillRect(x, y, 1, 1);
        }
    }
}

function updateColorPicker() {
    drawColorSpaceCrosshair();

    drawValueBar();
    drawValueBarCrosshair();
}

function updateCurrentColorRgb(r, g, b) {
    eraseColorSpaceCrosshair(current_h, current_s);

    current_r = r;
    current_g = g;
    current_b = b;

    const { h, s, v } = rgb2hsv(current_r, current_g, current_b);
    current_h = h;
    current_s = s;
    current_v = v;

    redrawEverything();
}

function updateCurrentColorHsv(h, s, v) {
    eraseColorSpaceCrosshair(current_h, current_s);

    current_h = h;
    current_s = s;
    current_v = v;

    const { r, g, b } = hsv2rgb(current_h, current_s, current_v);
    current_r = r;
    current_g = g;
    current_b = b;

    redrawEverything();
}

function redrawEverything() {
    const ctx = color_picker_preview.getContext('2d');
    ctx.fillStyle = `rgb(${current_r}, ${current_g}, ${current_b})`;
    ctx.fillRect(0, 0, 1, 1);

    updateColorPicker();
    updateNumericInput();
}

function updateNumericInput() {
    color_r.value = Math.round(current_r);
    color_g.value = Math.round(current_g);
    color_b.value = Math.round(current_b);

    color_h.value = Math.round(current_h);
    color_s.value = Math.round(current_s * 100);
    color_v.value = Math.round(current_v * 100);
}

function updateHueSatFromCursor(cursorX, cursorY) {
    const brect = color_space.getBoundingClientRect();
    const hueStep = 360 / brect.width;
    const satStep = 1 / brect.height;

    const xR = clamp(cursorX - brect.x, 0, brect.width);
    const yR = clamp(cursorY - brect.y, 0, brect.height);

    const hue = xR * hueStep;
    const sat = 1 - (yR * satStep);

    updateCurrentColorHsv(hue, sat, current_v);
}

function updateValFromCursor(cursorY) {
    const brect = value_bar.getBoundingClientRect();
    const valStep = 1 / brect.height;

    const rY = clamp(cursorY - brect.y, 0, brect.height);
    const val = 1 - (rY * valStep);

    updateCurrentColorHsv(current_h, current_s, val);
}

color_space.onclick = (ev) => {
    updateHueSatFromCursor(ev.clientX, ev.clientY);
};
value_bar.onclick = (ev) => {
    updateValFromCursor(ev.clientY);
};

color_space.onmousedown = () => {
    activeControl = 'color-space';
};

value_bar.onmousedown = () => {
    activeControl = 'value-bar';
};

value_bar.onwheel = (ev) => {
    let v = clamp(current_v - ev.deltaY / 2000, 0.0, 1.0);
    updateCurrentColorHsv(current_h, current_s, v);
};

document.addEventListener('mousemove', (ev) => {
    if (activeControl === 'color-space') {
        updateHueSatFromCursor(ev.clientX, ev.clientY);
    } else if (activeControl === 'value-bar') {
        updateValFromCursor(ev.clientY);
    }
});
document.addEventListener('mouseup', () => {
    activeControl = null;
});
