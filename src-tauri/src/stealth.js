// Stealth script — injected before any page JS runs.
// Hides signals that sites (Xiaohongshu, Douyin, etc.) use to detect embedded/automated browsers.
(function () {
  'use strict';

  // 1. navigator.webdriver → undefined (most critical check)
  Object.defineProperty(navigator, 'webdriver', {
    get: function () { return undefined; },
    configurable: true
  });

  // 2. navigator.plugins — mimic a real Chrome with common plugins
  var fakePlugins = [
    { name: 'PDF Viewer', description: 'Portable Document Format', filename: 'internal-pdf-viewer', length: 1 },
    { name: 'Chrome PDF Viewer', description: '', filename: 'internal-pdf-viewer', length: 1 },
    { name: 'Chromium PDF Viewer', description: '', filename: 'internal-pdf-viewer', length: 1 }
  ];
  try {
    Object.defineProperty(navigator, 'plugins', {
      get: function () {
        var arr = fakePlugins.slice();
        arr.item = function (i) { return arr[i] || null; };
        arr.namedItem = function (n) {
          for (var j = 0; j < arr.length; j++) { if (arr[j].name === n) return arr[j]; }
          return null;
        };
        arr.refresh = function () {};
        return arr;
      },
      configurable: true
    });
  } catch (e) {}

  // 3. navigator.mimeTypes — match plugins
  try {
    Object.defineProperty(navigator, 'mimeTypes', {
      get: function () {
        var arr = [{ type: 'application/pdf', suffixes: 'pdf', description: 'Portable Document Format' }];
        arr.item = function (i) { return arr[i] || null; };
        arr.namedItem = function (n) {
          for (var j = 0; j < arr.length; j++) { if (arr[j].type === n) return arr[j]; }
          return null;
        };
        return arr;
      },
      configurable: true
    });
  } catch (e) {}

  // 4. navigator.languages — ensure proper value
  try {
    Object.defineProperty(navigator, 'languages', {
      get: function () { return ['zh-CN', 'zh', 'en-US', 'en']; },
      configurable: true
    });
  } catch (e) {}

  // 5. navigator.platform — consistent with macOS Chrome
  try {
    if (navigator.platform === '' || navigator.platform === 'MacIntel') {
      // already fine for macOS
    } else {
      Object.defineProperty(navigator, 'platform', {
        get: function () { return 'MacIntel'; },
        configurable: true
      });
    }
  } catch (e) {}

  // 6. navigator.hardwareConcurrency — should be > 0
  if (!navigator.hardwareConcurrency || navigator.hardwareConcurrency < 2) {
    try {
      Object.defineProperty(navigator, 'hardwareConcurrency', {
        get: function () { return 8; },
        configurable: true
      });
    } catch (e) {}
  }

  // 7. navigator.deviceMemory — Chrome exposes this
  if (!navigator.deviceMemory) {
    try {
      Object.defineProperty(navigator, 'deviceMemory', {
        get: function () { return 8; },
        configurable: true
      });
    } catch (e) {}
  }

  // 8. navigator.connection — mimic real browser
  if (!navigator.connection) {
    try {
      Object.defineProperty(navigator, 'connection', {
        get: function () {
          return { effectiveType: '4g', rtt: 50, downlink: 10, saveData: false };
        },
        configurable: true
      });
    } catch (e) {}
  }

  // 9. window.chrome — many sites check for this on Chrome UA
  if (!window.chrome) {
    window.chrome = {
      runtime: {
        id: undefined,
        connect: function () {},
        sendMessage: function () {},
        onMessage: { addListener: function () {} }
      },
      loadTimes: function () { return {}; },
      csi: function () { return {}; }
    };
  }

  // 10. Permissions API — prevent "notification denied" fingerprinting
  var origQuery = window.Permissions && Permissions.prototype.query;
  if (origQuery) {
    Permissions.prototype.query = function (params) {
      if (params && params.name === 'notifications') {
        return Promise.resolve({ state: 'prompt', onchange: null });
      }
      return origQuery.call(this, params);
    };
  }

  // 11. WebGL vendor/renderer — don't leak "SwiftShader" or empty strings
  var getParam = WebGLRenderingContext.prototype.getParameter;
  if (getParam) {
    WebGLRenderingContext.prototype.getParameter = function (param) {
      // UNMASKED_VENDOR_WEBGL = 0x9245, UNMASKED_RENDERER_WEBGL = 0x9246
      if (param === 0x9245) return 'Google Inc. (Apple)';
      if (param === 0x9246) return 'ANGLE (Apple, Apple M1 Pro, OpenGL 4.1)';
      return getParam.call(this, param);
    };
  }
  if (typeof WebGL2RenderingContext !== 'undefined') {
    var getParam2 = WebGL2RenderingContext.prototype.getParameter;
    if (getParam2) {
      WebGL2RenderingContext.prototype.getParameter = function (param) {
        if (param === 0x9245) return 'Google Inc. (Apple)';
        if (param === 0x9246) return 'ANGLE (Apple, Apple M1 Pro, OpenGL 4.1)';
        return getParam2.call(this, param);
      };
    }
  }

  // 12. Prevent detection via iframe/toString checks
  // Some sites do `navigator.webdriver.toString()` or check function source
  try {
    var nativeToString = Function.prototype.toString;
    Function.prototype.toString = function () {
      // If it's one of our patched getters, return native-looking source
      if (this === Object.getOwnPropertyDescriptor(navigator.__proto__, 'webdriver')?.get) {
        return 'function get webdriver() { [native code] }';
      }
      return nativeToString.call(this);
    };
  } catch (e) {}
})();
