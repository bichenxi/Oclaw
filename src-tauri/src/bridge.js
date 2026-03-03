(function() {
  console.log('Claw Bridge Injected');
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
    }
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
