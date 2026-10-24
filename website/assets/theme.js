/* Run before paint. Storage can be disabled on file:// or in private browsing. */
(() => {
  let preference = null;
  try {
    preference = localStorage.getItem("photocraft-theme");
  } catch (_) {
    /* Use system preference. */
  }
  const dark =
    preference === "dark" ||
    (preference !== "light" &&
      window.matchMedia &&
      window.matchMedia("(prefers-color-scheme: dark)").matches);
  document.documentElement.dataset.theme = dark ? "dark" : "light";
})();
