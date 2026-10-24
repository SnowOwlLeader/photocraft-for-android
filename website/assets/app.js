/* No framework, build step, network API or server-side runtime required. */
(() => {
  "use strict";

  const root = document.documentElement;
  const motionQuery = window.matchMedia("(prefers-reduced-motion: reduce)");
  const colorQuery = window.matchMedia("(prefers-color-scheme: dark)");
  let chosenTheme = null;
  try {
    chosenTheme = localStorage.getItem("photocraft-theme");
  } catch (_) {
    /* Storage is optional. */
  }

  const themeButtons = document.querySelectorAll(".theme-toggle");
  function syncThemeButtons() {
    const isDark = root.dataset.theme === "dark";
    themeButtons.forEach((button) => {
      const label = isDark ? "切换浅色模式" : "切换深色模式";
      button.setAttribute("aria-label", label);
      button.title = label;
    });
    const themeMeta = document.querySelector('meta[name="theme-color"]');
    if (themeMeta) themeMeta.content = isDark ? "#202122" : "#f5f1e9";
  }
  themeButtons.forEach((button) =>
    button.addEventListener("click", () => {
      chosenTheme = root.dataset.theme === "dark" ? "light" : "dark";
      root.dataset.theme = chosenTheme;
      try {
        localStorage.setItem("photocraft-theme", chosenTheme);
      } catch (_) {
        /* Keep the session choice. */
      }
      syncThemeButtons();
    }),
  );
  colorQuery.addEventListener("change", (event) => {
    if (!chosenTheme) {
      root.dataset.theme = event.matches ? "dark" : "light";
      syncThemeButtons();
    }
  });
  syncThemeButtons();

  const menuToggle = document.querySelector(".menu-toggle");
  const mobileMenu = document.querySelector(".mobile-nav");
  function closeMenu() {
    if (!menuToggle || !mobileMenu) return;
    menuToggle.setAttribute("aria-expanded", "false");
    menuToggle.setAttribute("aria-label", "打开导航菜单");
    mobileMenu.hidden = true;
  }
  if (menuToggle && mobileMenu) {
    menuToggle.addEventListener("click", () => {
      const open = menuToggle.getAttribute("aria-expanded") !== "true";
      menuToggle.setAttribute("aria-expanded", String(open));
      menuToggle.setAttribute(
        "aria-label",
        open ? "关闭导航菜单" : "打开导航菜单",
      );
      mobileMenu.hidden = !open;
    });
    mobileMenu.addEventListener("click", (event) => {
      if (event.target.closest("a")) closeMenu();
    });
    document.addEventListener("keydown", (event) => {
      if (event.key === "Escape" && !mobileMenu.hidden) {
        closeMenu();
        menuToggle.focus();
      }
    });
    document.addEventListener("click", (event) => {
      if (!mobileMenu.hidden && !event.target.closest(".site-header"))
        closeMenu();
    });
    window
      .matchMedia("(min-width: 761px)")
      .addEventListener("change", (event) => {
        if (event.matches) closeMenu();
      });
  }

  if ("IntersectionObserver" in window && !motionQuery.matches) {
    const observer = new IntersectionObserver(
      (entries) => {
        entries.forEach((entry) => {
          if (entry.isIntersecting) {
            entry.target.classList.add("is-visible");
            observer.unobserve(entry.target);
          }
        });
      },
      { threshold: 0.08 },
    );
    root.classList.add("reveal-ready");
    document
      .querySelectorAll(".reveal")
      .forEach((element) => observer.observe(element));
    motionQuery.addEventListener("change", (event) => {
      if (event.matches) {
        root.classList.remove("reveal-ready");
        observer.disconnect();
      }
    });
  }

  // CSS preserves true 3D planes. Pointer events only update the parent rotation.
  const stage = document.querySelector(".hero-stage");
  const artwork = document.querySelector(".artwork");
  const sceneControl = document.querySelector(".scene-control");
  if (stage && artwork) {
    let frame = 0;
    let pointerX = 0;
    let pointerY = 0;
    function resetTilt() {
      cancelAnimationFrame(frame);
      frame = 0;
      artwork.style.transform = "";
    }
    stage.addEventListener("pointermove", (event) => {
      if (motionQuery.matches || event.pointerType === "touch") return;
      const bounds = stage.getBoundingClientRect();
      pointerX = (event.clientX - bounds.left) / bounds.width - 0.5;
      pointerY = (event.clientY - bounds.top) / bounds.height - 0.5;
      if (frame) return;
      frame = requestAnimationFrame(() => {
        artwork.style.transform = `rotateX(${12 - pointerY * 15}deg) rotateY(${-24 + pointerX * 22}deg) rotateZ(${11 + pointerX * 3}deg)`;
        frame = 0;
      });
    });
    stage.addEventListener("pointerleave", resetTilt);
    motionQuery.addEventListener("change", resetTilt);
  }
  if (stage && sceneControl) {
    sceneControl.addEventListener("click", () => {
      const expanded = stage.classList.toggle("is-expanded");
      sceneControl.setAttribute("aria-pressed", String(expanded));
      sceneControl.querySelector("span").textContent = expanded
        ? "收起图层"
        : "展开图层";
    });
  }

  const previewTabs = [...document.querySelectorAll("[data-preview]")];
  const previewImage = document.getElementById("editor-preview");
  const previewPanel = document.getElementById("workspace-panel");
  const previews = {
    layers: {
      image: "assets/editor-wave.webp",
      filename: "the-great-wave.pcraft",
      alt: "PhotoCraft 上游桌面编辑器：使用图层和曲线编辑《神奈川冲浪里》",
      caption: "从图层到曲线，让每一步创作都有迹可循。",
    },
    filters: {
      image: "assets/editor-filters.webp",
      filename: "the-starry-night.pcraft",
      alt: "PhotoCraft 上游桌面编辑器：对梵高《星月夜》应用图像滤镜",
      caption: "在光影与色彩之间，找到属于你的表达。",
    },
    masks: {
      image: "assets/editor-masks.webp",
      filename: "girl-with-a-pearl-earring.pcraft",
      alt: "PhotoCraft 上游桌面编辑器：在《戴珍珠耳环的少女》上演示蒙版工作流",
      caption: "用蒙版雕琢细节，让修改保留更多余地。",
    },
  };
  function selectPreview(tab, focus = false) {
    if (!previewImage || !previewPanel) return;
    const preview = previews[tab.dataset.preview];
    if (!preview) return;
    previewTabs.forEach((item) => {
      const selected = item === tab;
      item.setAttribute("aria-selected", String(selected));
      item.tabIndex = selected ? 0 : -1;
    });
    previewImage.src = preview.image;
    previewImage.alt = preview.alt;
    document.getElementById("preview-filename").textContent = preview.filename;
    document.getElementById("preview-caption").textContent = preview.caption;
    previewPanel.setAttribute("aria-labelledby", tab.id);
    if (focus) tab.focus();
  }
  previewTabs.forEach((tab, index) => {
    tab.addEventListener("click", () => selectPreview(tab));
    tab.addEventListener("keydown", (event) => {
      let nextIndex;
      if (event.key === "ArrowRight")
        nextIndex = (index + 1) % previewTabs.length;
      if (event.key === "ArrowLeft")
        nextIndex = (index - 1 + previewTabs.length) % previewTabs.length;
      if (event.key === "Home") nextIndex = 0;
      if (event.key === "End") nextIndex = previewTabs.length - 1;
      if (nextIndex !== undefined) {
        event.preventDefault();
        selectPreview(previewTabs[nextIndex], true);
      }
    });
  });

  // Pin actual release URLs: the site stays usable without CORS or an API request.
  const downloadButton = document.getElementById("primary-download");
  const architectures = document.querySelectorAll('input[name="architecture"]');
  const releaseBase =
    "https://gh.b52m.cn/https://github.com/SnowOwlLeader/photocraft-for-android/releases/download/v0.14.0-android.1/";
  function syncDownload() {
    if (!downloadButton) return;
    const selected = document.querySelector(
      'input[name="architecture"]:checked',
    );
    const isX86 = selected && selected.value === "x86_64";
    const abi = isX86 ? "x86_64" : "arm64-v8a";
    downloadButton.href = `${releaseBase}PhotoCraft-0.14.0-android.1-14001-${abi}.apk`;
    document.getElementById("download-button-label").textContent =
      `下载 ${isX86 ? "x86_64" : "ARM64"} 安装包`;
    document.getElementById("download-meta").textContent =
      `v0.14.0-android.1 · 14001 · ${abi} · APK`;
  }
  architectures.forEach((input) =>
    input.addEventListener("change", syncDownload),
  );
  syncDownload();
})();
