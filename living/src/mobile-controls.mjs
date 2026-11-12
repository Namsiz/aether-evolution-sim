// mobile-controls.mjs — Touch button bar + responsive layout for Phase 19+
// Adds a bottom toolbar of toggle buttons that mirror keyboard shortcuts.
// Pure DOM + CSS-in-JS; no external dependencies.

const BUTTONS = [
  // [label, key, hint]
  ['⏯', 'Space',   '暂停/继续'],
  ['M',   'KeyM',  '矿化'],
  ['N',   'KeyN',  '直方图'],
  ['P',   'KeyP',  '光谱'],
  ['T',   'KeyT',  '谱系树'],
  ['G',   'KeyG',  '地质'],
  ['B',   'KeyB',  '营养'],
  ['U',   'KeyU',  '生态位'],
  ['V',   'KeyV',  '形态对比'],
  ['←',   'ArrowLeft',  '回退'],
  ['→',   'ArrowRight', '前进'],
];

export function createMobileToolbar(opts = {}) {
  const isMobile =
    /Android|iPhone|iPad|iPod/i.test(navigator.userAgent) ||
    (navigator.maxTouchPoints > 0 && window.innerWidth < 900);

  if (!isMobile) return null; // desktop: skip entirely

  const root = document.createElement('div');
  root.id = 'mobile-toolbar';
  Object.assign(root.style, {
    position: 'fixed', bottom: '0', left: '0', right: '0',
    display: 'flex', overflowX: 'auto', overflowY: 'hidden',
    background: 'rgba(10,12,20,0.92)',
    borderTop: '1px solid rgba(255,255,255,0.08)',
    zIndex: 200, padding: '6px 4px 10px 4px',
    WebkitOverflowScrolling: 'touch',
    touchAction: 'pan-x',
  });

  const state = {}; // track toggle state (reflect keydown state)

  for (const [label, code, hint] of BUTTONS) {
    const btn = document.createElement('button');
    btn.textContent = label;
    btn.dataset.code = code;
    Object.assign(btn.style, {
      minWidth: '40px', height: '38px', margin: '0 3px',
      background: 'rgba(255,255,255,0.07)',
      border: '1px solid rgba(255,255,255,0.15)',
      borderRadius: '8px', color: '#fff',
      fontSize: '15px', fontWeight: '600',
      cursor: 'pointer', userSelect: 'none',
      WebkitTapHighlightColor: 'rgba(255,255,255,0.15)',
    });

    btn.addEventListener('touchstart', (e) => {
      e.preventDefault();
      // Simulate a keydown+keyup for the code
      const ev = new KeyboardEvent('keydown', { code, key: label, bubbles: true });
      window.dispatchEvent(ev);
      // Toggle button active style
      const on = btn.dataset.on === '1';
      btn.dataset.on = on ? '' : '1';
      btn.style.background = on ? 'rgba(255,255,255,0.07)' : 'rgba(80,160,255,0.35)';
      btn.style.borderColor = on ? 'rgba(255,255,255,0.15)' : 'rgba(80,160,255,0.6)';
    }, { passive: false });

    // Also support mouse click (desktop in narrow window)
    btn.addEventListener('click', (e) => {
      if (e.pointerType === 'mouse') {
        const ev = new KeyboardEvent('keydown', { code, key: label, bubbles: true });
        window.dispatchEvent(ev);
      }
    });

    root.appendChild(btn);
  }

  document.body.appendChild(root);

  // Responsive CSS: shrink HUD overlay on small screens
  const style = document.createElement('style');
  style.textContent = `
    @media (max-width: 700px) {
      /* Give room for the mobile toolbar */
      #hud-panel, .overlay-panel { font-size: 11px !important; }
      /* Prevent double-tap zoom on canvas */
      canvas { touch-action: none; }
      body { overscroll-behavior: none; }
    }
  `;
  document.head.appendChild(style);

  // Handle canvas resize on orientation change
  const handleResize = () => {
    const canvas = document.querySelector('canvas');
    if (!canvas) return;
    const renderer = canvas.__renderer; // set by createViewerRenderer
    if (!renderer) return;
    const w = window.innerWidth, h = window.innerHeight - root.offsetHeight;
    renderer.setSize(w, h, false);
    canvas.style.width = w + 'px';
    canvas.style.height = h + 'px';
    const camera = canvas.__camera;
    if (camera) {
      camera.aspect = w / h;
      camera.updateProjectionMatrix();
    }
  };
  window.addEventListener('resize', handleResize);
  window.addEventListener('orientationchange', () =>
    setTimeout(handleResize, 200));

  return {
    root,
    destroy() {
      root.remove();
      style.remove();
      window.removeEventListener('resize', handleResize);
    },
  };
}
