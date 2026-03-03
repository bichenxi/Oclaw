(function() {
  console.log('Claw Bridge Injected');
  function makeSelector(el) {
    if (el.id && /^[a-zA-Z][\w-]*$/.test(el.id)) return '#' + el.id;
    var tag = el.tagName.toLowerCase();
    if (el.name && (tag === 'input' || tag === 'button')) return tag + '[name="' + el.name.replace(/"/g, '\\"') + '"]';
    var txt = (el.textContent || '').trim().slice(0, 30);
    if (txt) return tag + ':contains("' + txt.replace(/"/g, '') + '")';
    return tag;
  }
  function getSimplifiedDOM() {
    var nodes = document.querySelectorAll('a, button, input, select, textarea, [role="button"], [onclick]');
    var out = [];
    var seen = 0;
    for (var i = 0; i < nodes.length && seen < 80; i++) {
      var el = nodes[i];
      var r = el.getBoundingClientRect();
      if (r.width === 0 && r.height === 0) continue;
      var tag = el.tagName.toLowerCase();
      var text = (el.value !== undefined ? el.value : el.textContent || '').trim().slice(0, 50);
      var sel = makeSelector(el);
      out.push({ tag: tag, selector: sel, text: text, rect: { x: Math.round(r.left), y: Math.round(r.top), w: Math.round(r.width), h: Math.round(r.height) } });
      seen++;
    }
    return out;
  }
  function sendDomSnapshot() {
    var data = getSimplifiedDOM();
    var json = JSON.stringify(data);
    var b64 = btoa(unescape(encodeURIComponent(json)));
    var iframe = document.createElement('iframe');
    iframe.style.display = 'none';
    iframe.src = 'claw://dom-snapshot#' + b64.replace(/\+/g, '-').replace(/\//g, '_');
    document.body.appendChild(iframe);
    setTimeout(function() { iframe.remove(); }, 50);
  }
  window.__clawBridge = {
    highlight: function(selector) {
      var el = document.querySelector(selector);
      if (!el) return false;
      var r = el.getBoundingClientRect();
      var wrap = document.createElement('div');
      wrap.setAttribute('data-claw-highlight', '1');
      wrap.style.cssText = 'position:fixed;left:' + r.left + 'px;top:' + r.top + 'px;width:' + r.width + 'px;height:' + r.height + 'px;border:3px solid #e11;box-shadow:0 0 0 2px #f88;pointer-events:none;z-index:2147483647;box-sizing:border-box;';
      document.body.appendChild(wrap);
      setTimeout(function() { if (wrap.parentNode) wrap.parentNode.removeChild(wrap); }, 2500);
      return true;
    },
    getSimplifiedDOM: getSimplifiedDOM,
    sendDomSnapshot: sendDomSnapshot
  };

  // 子 Webview 内没有 __TAURI__，通过 claw:// 协议触发导航，由 Rust on_navigation 拦截并处理
  document.addEventListener('click', (e) => {
    if (!window.__clawBridgeLabel) return;
    var tag = (e.target && e.target.tagName) || '';
    var q = 'label=' + encodeURIComponent(window.__clawBridgeLabel) +
      '&x=' + e.clientX + '&y=' + e.clientY + '&tag=' + encodeURIComponent(tag);
    var iframe = document.createElement('iframe');
    iframe.style.display = 'none';
    iframe.src = 'claw://webview-click?' + q;
    document.body.appendChild(iframe);
    setTimeout(function() { iframe.remove(); }, 50);
  });
})();
