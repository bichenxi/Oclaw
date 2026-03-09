// Xiaohongshu (Little Red Book) site-specific DOM helpers.
// Injected into all webviews; only activates on XHS pages.
(function () {
  'use strict';

  function $(sels) {
    if (typeof sels === 'string') sels = [sels];
    for (var i = 0; i < sels.length; i++) {
      var el = document.querySelector(sels[i]);
      if (el) return el;
    }
    return null;
  }

  window.__clawXhs = {

    // ── Login check ───────────────────────────────────────────────────────
    isLoggedIn: function () {
      if ($(['.login-btn', '.login-container'])) return false;
      return !!$(['.user', '.user-info', '.sidebar-user']);
    },

    // ── Search ────────────────────────────────────────────────────────────

    setSearchInput: function (keyword) {
      var input = $(['#search-input', 'input[placeholder*="搜索"]', '.search-input input']);
      if (!input) return false;
      input.focus();
      var desc = Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, 'value');
      if (desc && desc.set) desc.set.call(input, keyword);
      else input.value = keyword;
      input.dispatchEvent(new Event('input', { bubbles: true }));
      input.dispatchEvent(new Event('change', { bubbles: true }));
      return true;
    },

    clickSearch: function () {
      var btn = $(['.search-icon', '#search-icon', '.search-btn']);
      if (btn) { btn.click(); return true; }
      var input = $(['#search-input', 'input[placeholder*="搜索"]']);
      if (input) {
        input.dispatchEvent(new KeyboardEvent('keydown', { key: 'Enter', code: 'Enter', keyCode: 13, bubbles: true }));
        return true;
      }
      return false;
    },

    // ── Feed / Results ────────────────────────────────────────────────────

    getResults: function (limit) {
      limit = limit || 20;
      var container = $(['.feeds-container', '#feeds-container']);
      if (!container) return [];
      var items = [];
      var children = Array.from(container.children);
      for (var i = 0; i < children.length && items.length < limit; i++) {
        var el = children[i];
        var isVideo = !!el.querySelector('.play-icon');
        var titleEl = el.querySelector('.title') || el.querySelector('.note-title');
        var title = titleEl ? titleEl.textContent.trim() : '';
        var cover = (el.querySelector('img') || {}).src || '';
        var authorEl = el.querySelector('.author-wrapper .name') || el.querySelector('.author .name');
        var author = authorEl ? authorEl.textContent.trim() : '';
        var likesEl = el.querySelector('.like-wrapper .count') || el.querySelector('.engagement .like .count');
        var likes = likesEl ? likesEl.textContent.trim() : '';
        if (!title && !el.textContent.trim()) continue;
        items.push({
          index: i,
          type: isVideo ? 'video' : 'note',
          title: title || el.textContent.trim().replace(/\s+/g, ' ').slice(0, 120),
          cover: cover,
          author: author,
          likes: likes
        });
      }
      return items;
    },

    // ── Note preview (overlay) ────────────────────────────────────────────

    openNote: function (index) {
      var container = $(['.feeds-container', '#feeds-container']);
      if (!container || !container.children[index]) return false;
      var target = container.children[index];
      var cover = target.querySelector('a.cover') || target.querySelector('a') || target.querySelector('img');
      if (cover) { cover.click(); return true; }
      target.click();
      return true;
    },

    closeNote: function () {
      var btn = $([
        '.close-circle', '.note-close',
        '.close', '[class*="close-btn"]'
      ]);
      if (btn) { btn.click(); return true; }
      document.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', keyCode: 27, bubbles: true }));
      return true;
    },

    // Whether a note overlay is currently open
    isNoteOpen: function () {
      return !!$([
        '.note-detail-mask', '.note-overlay',
        '.note-scroller', '#noteContainer'
      ]);
    },

    // ── Read note content ─────────────────────────────────────────────────

    getNoteContent: function () {
      var title = ($(['#detail-title', '.note-scroller .title', '.note-content .title']) || {}).textContent || '';
      var desc = ($(['#detail-desc', '.note-scroller .desc', '.note-content .desc', '.note-text']) || {}).innerText || '';
      var author = ($(['.author-wrapper .username', '.author-wrapper .name', '.user-nickname']) || {}).textContent || '';
      var date = ($(['.note-scroller .date', '.bottom-container .date', '.publish-date', '.date']) || {}).textContent || '';
      var likes = ($(['.interactions .like .count', '.engage-bar .like .count', '.like-wrapper .count']) || {}).textContent || '';
      var collects = ($(['.interactions .collect .count', '.engage-bar .collect .count', '.collect-wrapper .count']) || {}).textContent || '';
      var comments = ($(['.interactions .chat .count', '.engage-bar .chat .count', '.comment-wrapper .count']) || {}).textContent || '';

      var tags = [];
      var tagEls = document.querySelectorAll('#hash-tag a, .tag, .note-tag a, a[href*="search_result"]');
      for (var i = 0; i < tagEls.length; i++) {
        var t = tagEls[i].textContent.trim();
        if (t && tags.indexOf(t) === -1) tags.push(t);
      }

      var topComments = [];
      var cmtEls = document.querySelectorAll('.comment-item, .comments-container .comment, .parent-comment');
      for (var j = 0; j < cmtEls.length && j < 10; j++) {
        var cName = (cmtEls[j].querySelector('.name, .user-name, .author') || {}).textContent || '';
        var cText = (cmtEls[j].querySelector('.content, .comment-text, .text') || {}).textContent || '';
        if (cText.trim()) topComments.push({ author: cName.trim(), text: cText.trim() });
      }

      return {
        title: title.trim(),
        content: desc.trim(),
        author: author.trim(),
        date: date.trim(),
        likes: likes.trim(),
        collects: collects.trim(),
        comments: comments.trim(),
        tags: tags,
        topComments: topComments
      };
    },

    // ── Note images ───────────────────────────────────────────────────────

    getNoteImages: function () {
      var swiper = $(['.swiper-wrapper', '.carousel-wrapper', '.note-slider']);
      if (!swiper) return [];
      var imgs = swiper.querySelectorAll('img');
      var list = [];
      for (var i = 0; i < imgs.length; i++) {
        var img = imgs[i];
        var w = img.naturalWidth || img.width;
        var h = img.naturalHeight || img.height;
        if (img.src && /^https?:/.test(img.src) && w > 200 && h > 200 && list.indexOf(img.src) === -1) {
          list.push(img.src);
        }
      }
      if (list.length === 0) {
        var slides = swiper.querySelectorAll('[style*="background-image"]');
        for (var j = 0; j < slides.length; j++) {
          var m = slides[j].style.backgroundImage.match(/url\(["']?(https?:\/\/[^"')]+)["']?\)/);
          if (m && list.indexOf(m[1]) === -1) list.push(m[1]);
        }
      }
      return list;
    },

    // ── Feed scroll ───────────────────────────────────────────────────────

    scrollFeed: function () {
      window.scrollBy({ top: 800, behavior: 'smooth' });
    }
  };
})();
