(function () {
  'use strict';

  // ── Selector generation ──────────────────────────────────────────────────────
  // Priority: #id > [data-testid] > tag[name] > tag[aria-label] > nth-of-type path
  function makeSelector(el) {
    if (el.id && /^[a-zA-Z_][\w-]*$/.test(el.id)) return '#' + el.id;
    var dt = el.getAttribute('data-testid');
    if (dt) return '[data-testid="' + dt.replace(/"/g, '\\"') + '"]';
    var tag = el.tagName.toLowerCase();
    var name = el.getAttribute('name');
    if (name && ['input', 'button', 'select', 'textarea'].indexOf(tag) !== -1)
      return tag + '[name="' + name.replace(/"/g, '\\"') + '"]';
    var al = el.getAttribute('aria-label');
    if (al)
      return tag + '[aria-label="' + al.slice(0, 40).replace(/"/g, '\\"') + '"]';
    // Build nth-of-type path up to 4 ancestors
    var parts = [];
    var cur = el;
    for (var d = 0; d < 4 && cur && cur.parentElement && cur !== document.documentElement; d++) {
      var p = cur.parentElement;
      var t = cur.tagName;
      var sibs = Array.prototype.filter.call(p.children, function (c) { return c.tagName === t; });
      var nth = sibs.indexOf(cur) + 1;
      if (cur.id && /^[a-zA-Z_][\w-]*$/.test(cur.id)) {
        parts.unshift('#' + cur.id);
        break;
      }
      parts.unshift(t.toLowerCase() + (sibs.length > 1 ? ':nth-of-type(' + nth + ')' : ''));
      cur = p;
    }
    return parts.join(' > ') || tag;
  }

  // ── Role detection ───────────────────────────────────────────────────────────
  function getRole(el) {
    var tag = el.tagName.toLowerCase();
    var role = (el.getAttribute('role') || '').toLowerCase();
    if (['button', 'menuitem', 'tab', 'checkbox', 'radio', 'link', 'option'].indexOf(role) !== -1) return role;
    if (tag === 'a') return 'link';
    if (tag === 'button') return 'button';
    if (tag === 'input') {
      var t = (el.type || 'text').toLowerCase();
      if (t === 'submit' || t === 'button' || t === 'reset') return 'button';
      if (t === 'checkbox') return 'checkbox';
      if (t === 'radio') return 'radio';
      return 'input';
    }
    if (tag === 'select') return 'select';
    if (tag === 'textarea') return 'textarea';
    return 'clickable';
  }

  // ── Visibility check ─────────────────────────────────────────────────────────
  function isVisible(el, r) {
    if (r.width === 0 && r.height === 0) return false;
    try {
      var cs = window.getComputedStyle(el);
      if (cs.visibility === 'hidden' || cs.display === 'none' || cs.opacity === '0') return false;
    } catch (e) { /* cross-origin frame, skip */ }
    return true;
  }

  // ── Core: get full page context ───────────────────────────────────────────────
  // Returns { meta, elements } — richer than the old getSimplifiedDOM()
  function getPageContext() {
    var meta = {
      url: location.href,
      title: document.title,
      viewport: { w: window.innerWidth, h: window.innerHeight },
      scroll: {
        y: Math.round(window.scrollY),
        maxY: Math.max(0, Math.round(document.documentElement.scrollHeight - window.innerHeight))
      }
    };

    var cssQuery = [
      'a[href]', 'button', 'input:not([type="hidden"])', 'select', 'textarea',
      '[role="button"]', '[role="link"]', '[role="tab"]',
      '[role="checkbox"]', '[role="menuitem"]', '[role="radio"]',
      '[onclick]', '[tabindex]:not([tabindex="-1"])'
    ].join(',');

    var nodes = document.querySelectorAll(cssQuery);
    var elements = [];
    var seen = 0;
    for (var i = 0; i < nodes.length && seen < 200; i++) {
      var el = nodes[i];
      var r = el.getBoundingClientRect();
      if (!isVisible(el, r)) continue;

      var tag = el.tagName.toLowerCase();
      var role = getRole(el);
      var ariaLabel = el.getAttribute('aria-label') || '';
      var rawText = ariaLabel
        || (el.value !== undefined && role !== 'link' ? el.value : '')
        || el.textContent
        || '';
      var text = rawText.trim().replace(/\s+/g, ' ').slice(0, 100);
      var inViewport = r.bottom > 0 && r.top < window.innerHeight;

      var item = {
        id: seen + 1,
        role: role,
        tag: tag,
        text: text,
        selector: makeSelector(el),
        rect: { x: Math.round(r.left), y: Math.round(r.top), w: Math.round(r.width), h: Math.round(r.height) },
        inViewport: inViewport
      };

      if (tag === 'input') {
        item.inputType = (el.type || 'text').toLowerCase();
        if (el.placeholder) item.placeholder = el.placeholder;
        if (el.value) item.value = el.value.slice(0, 200);
        if (item.inputType === 'checkbox' || item.inputType === 'radio') item.checked = el.checked;
      }
      if (tag === 'textarea') {
        if (el.placeholder) item.placeholder = el.placeholder;
        if (el.value) item.value = el.value.slice(0, 200);
      }
      if (tag === 'a' && el.href) item.href = el.href;
      if (el.disabled) item.disabled = true;
      if (tag === 'select') {
        item.options = Array.prototype.slice.call(el.options, 0, 20).map(function (o) {
          return { value: o.value, text: o.text.trim(), selected: o.selected };
        });
      }

      elements.push(item);
      seen++;
    }
    return { meta: meta, elements: elements };
  }

  // ── Transport helper ─────────────────────────────────────────────────────────
  function b64url(str) {
    return btoa(unescape(encodeURIComponent(str))).replace(/\+/g, '-').replace(/\//g, '_');
  }

  // ── Batch content extraction ────────────────────────────────────────────────
  // Extract text/attributes from all elements matching a CSS selector.
  // Returns [{ text, href?, src?, selector }] up to `limit` items.
  function extractContent(selector, limit) {
    limit = limit || 50;
    var nodes = document.querySelectorAll(selector);
    var out = [];
    for (var i = 0; i < nodes.length && i < limit; i++) {
      var el = nodes[i];
      var text = (el.innerText || el.textContent || '').trim().replace(/\s+/g, ' ').slice(0, 500);
      if (!text) continue;
      var item = { text: text };
      if (el.tagName === 'A' && el.href) item.href = el.href;
      if (el.tagName === 'IMG' && el.src) item.src = el.src;
      var dt = el.getAttribute('data-testid');
      if (dt) item.selector = '[data-testid="' + dt.replace(/"/g, '\\"') + '"]';
      else item.selector = makeSelector(el);
      out.push(item);
    }
    return out;
  }

  // Extract text from all visible text nodes within a region, yielding readable prose.
  // Useful for reading article bodies, search result snippets, etc.
  function extractText(selector) {
    var root = selector ? document.querySelector(selector) : document.body;
    if (!root) return '';
    // Walk through semantic block elements to produce structured text
    var blocks = root.querySelectorAll('p, h1, h2, h3, h4, h5, h6, li, td, th, blockquote, pre, figcaption, [role="heading"]');
    if (blocks.length > 0) {
      var parts = [];
      for (var i = 0; i < blocks.length && i < 200; i++) {
        var t = (blocks[i].innerText || '').trim();
        if (t) parts.push(t);
      }
      return parts.join('\n');
    }
    return (root.innerText || '').trim().slice(0, 10000);
  }

  // ── Public bridge API ────────────────────────────────────────────────────────
  window.__clawBridge = {
    highlight: function (selector) {
      var el = document.querySelector(selector);
      if (!el) return false;
      var r = el.getBoundingClientRect();
      var wrap = document.createElement('div');
      wrap.setAttribute('data-claw-highlight', '1');
      wrap.style.cssText =
        'position:fixed;left:' + r.left + 'px;top:' + r.top + 'px;' +
        'width:' + r.width + 'px;height:' + r.height + 'px;' +
        'border:3px solid #e11;box-shadow:0 0 0 2px rgba(255,136,136,.6);' +
        'pointer-events:none;z-index:2147483647;box-sizing:border-box;border-radius:3px;';
      document.body.appendChild(wrap);
      setTimeout(function () { if (wrap.parentNode) wrap.parentNode.removeChild(wrap); }, 2500);
      return true;
    },
    getSimplifiedDOM: function () { return getPageContext().elements; },
    getPageContext: getPageContext,
    extractContent: extractContent,
    extractText: extractText,
    sendDomSnapshot: function () {
      var json = JSON.stringify(getPageContext());
      window.location.assign('claw://dom-snapshot#' + b64url(json));
    }
  };

  // ── Click tracking ────────────────────────────────────────────────────────────
  document.addEventListener('click', function (e) {
    if (!window.__clawBridgeLabel) return;
    var tag = (e.target && e.target.tagName) || '';
    var q = 'label=' + encodeURIComponent(window.__clawBridgeLabel) +
      '&x=' + e.clientX + '&y=' + e.clientY + '&tag=' + encodeURIComponent(tag);
    var iframe = document.createElement('iframe');
    iframe.style.display = 'none';
    iframe.src = 'claw://webview-click?' + q;
    document.body.appendChild(iframe);
    setTimeout(function () { iframe.remove(); }, 50);
  });
})();
