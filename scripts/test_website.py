"""Browser regression for the static marketing website; no Rust build needed.

Optional development dependency: pip install playwright
Usage: python scripts/test_website.py [--browser /path/to/chrome]
"""

from __future__ import annotations

import argparse
from functools import partial
from http.server import SimpleHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from threading import Thread
from urllib.parse import unquote, urlsplit

from playwright.sync_api import sync_playwright


ROOT = Path(__file__).resolve().parents[1]
SITE = ROOT / "website"


class QuietHandler(SimpleHTTPRequestHandler):
    def log_message(self, *_args):
        pass

    def copyfile(self, source, outputfile):
        try:
            super().copyfile(source, outputfile)
        except (ConnectionAbortedError, ConnectionResetError, BrokenPipeError):
            pass  # A browser may cancel a lazy image while navigating.


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--browser", help="Installed Chromium/Chrome/Edge executable")
    args = parser.parse_args()
    # Serve the parent to exercise both /website/ and /website/download.html.
    server = ThreadingHTTPServer(("127.0.0.1", 0), partial(QuietHandler, directory=str(ROOT)))
    Thread(target=server.serve_forever, daemon=True).start()
    origin = f"http://127.0.0.1:{server.server_port}"
    base = origin + "/website/"
    checks = []

    def check(condition, label):
        if not condition:
            raise AssertionError(label)
        checks.append(label)

    def load_images(page):
        page.locator("img").evaluate_all("els => els.forEach(i => i.loading = 'eager')")
        page.wait_for_function("[...document.images].every(i => i.complete && i.naturalWidth > 0)")

    try:
        with sync_playwright() as playwright:
            launch = {"headless": True}
            if args.browser:
                launch["executable_path"] = args.browser
            browser = playwright.chromium.launch(**launch)
            context = browser.new_context(viewport={"width": 1440, "height": 1000}, color_scheme="light")
            page = context.new_page()
            errors = []
            failures = []
            page.on("pageerror", lambda error: errors.append(str(error)))
            page.on("response", lambda response: failures.append(response.url) if response.status >= 400 and response.url.startswith(origin) else None)
            page.goto(base, wait_until="networkidle")

            # Every local page link, image, script and stylesheet resolves in a subdirectory.
            for name in ("index.html", "download.html"):
                page.goto(base + name, wait_until="networkidle")
                urls = page.locator("[src],link[href],a[href]").evaluate_all(
                    "els => els.map(e => e.src || e.href).filter(Boolean)"
                )
                for url in set(urls):
                    parts = urlsplit(url)
                    if not url.startswith(origin):
                        continue
                    local = ROOT / unquote(parts.path.lstrip("/"))
                    check(local.exists(), f"Local resource exists: {parts.path}")
                    if parts.fragment and local.suffix == ".html":
                        text = local.read_text(encoding="utf-8")
                        check(f'id="{unquote(parts.fragment)}"' in text, f"Anchor exists: {parts.path}#{parts.fragment}")
                load_images(page)
                check(page.locator("img").evaluate_all("els => els.every(i => i.complete && i.naturalWidth > 0)"), f"All images load: {name}")

            page.goto(base, wait_until="networkidle")
            check(page.locator("html").get_attribute("data-theme") == "light", "System light theme")
            page.locator(".theme-toggle").click()
            check(page.locator("html").get_attribute("data-theme") == "dark", "Theme toggle")
            page.goto(base + "download.html", wait_until="networkidle")
            check(page.locator("html").get_attribute("data-theme") == "dark", "Theme persists across pages")
            page.reload(wait_until="networkidle")
            check(page.locator("html").get_attribute("data-theme") == "dark", "Theme persists after reload")
            check(page.locator(".theme-toggle").get_attribute("aria-label") == "切换浅色模式", "Theme accessible label")

            # Proxy URLs must wrap actual published release URLs exactly once.
            readme = (ROOT / "README.md").read_text(encoding="utf-8")
            proxy_prefix = "https://gh.b52m.cn/"
            for abi, suffix in (("x86_64", "x86_64"), ("arm64", "arm64-v8a")):
                page.locator(f'.architecture-option:has(input[value="{abi}"])').click()
                href = page.locator("#primary-download").get_attribute("href")
                check(href.startswith(proxy_prefix) and href.removeprefix(proxy_prefix) in readme and href.endswith(f"-{suffix}.apk"), f"Correct proxied release URL: {abi}")
                check(suffix in page.locator("#download-meta").inner_text(), f"Correct architecture metadata: {abi}")
            # Capture navigation, avoiding a real 50+ MB download.
            navigated = []
            page.route("**/*.apk", lambda route: (navigated.append(route.request.url), route.abort()))
            page.locator("#primary-download").click()
            check(len(navigated) == 1 and navigated[0].startswith(proxy_prefix) and navigated[0].endswith("-arm64-v8a.apk"), "Download button navigates to selected proxied APK")

            page.goto(base, wait_until="networkidle")
            control = page.locator(".scene-control")
            control.click()
            check(control.get_attribute("aria-pressed") == "true", "3D layers expand")
            check("is-expanded" in page.locator(".hero-stage").get_attribute("class"), "Expanded scene state")
            control.click()
            check(control.get_attribute("aria-pressed") == "false", "3D layers collapse")
            stage = page.locator(".hero-stage").bounding_box()
            page.mouse.move(stage["x"] + stage["width"] * .7, stage["y"] + stage["height"] * .3)
            page.wait_for_timeout(60)
            check("rotateX" in (page.locator("#artwork").get_attribute("style") or ""), "Pointer drives 3D rotation")

            page.locator("#tab-layers").click()
            page.keyboard.press("ArrowRight")
            check(page.locator("#tab-filters").get_attribute("aria-selected") == "true", "Preview right-arrow navigation")
            check(page.locator("#editor-preview").get_attribute("src").endswith("editor-filters.webp"), "Preview image switches")
            page.keyboard.press("End")
            check(page.locator("#workspace-panel").get_attribute("aria-labelledby") == "tab-masks", "Preview End key and panel label")
            page.keyboard.press("Home")
            check(page.locator("#tab-layers").get_attribute("aria-selected") == "true", "Preview Home key")
            page.locator(".faq-list summary").first.click()
            check(page.locator(".faq-list details").first.get_attribute("open") is not None, "FAQ expands")

            for width in (320, 360, 390, 768, 1024, 1440):
                page.set_viewport_size({"width": width, "height": 900})
                for name in ("index.html", "download.html"):
                    page.goto(base + name, wait_until="networkidle")
                    check(page.evaluate("document.documentElement.scrollWidth <= window.innerWidth"), f"No horizontal overflow: {name} at {width}px")

            page.set_viewport_size({"width": 390, "height": 844})
            page.goto(base, wait_until="networkidle")
            menu = page.locator(".menu-toggle")
            menu.click()
            check(page.locator("#mobile-nav").is_visible(), "Mobile menu opens")
            page.keyboard.press("Escape")
            check(not page.locator("#mobile-nav").is_visible(), "Escape closes mobile menu")
            menu.click()
            page.locator("#mobile-nav a[href='download.html']").click()
            check(page.url.endswith("/website/download.html"), "Mobile navigation reaches download page")
            page.go_back(wait_until="networkidle")

            reduced = browser.new_context(reduced_motion="reduce", color_scheme="dark")
            quiet = reduced.new_page()
            quiet.goto(base, wait_until="networkidle")
            check(quiet.locator("html").get_attribute("data-theme") == "dark", "System dark theme")
            check(quiet.locator(".artwork-float").evaluate("e => getComputedStyle(e).animationName") == "none", "Reduced motion disables floating animation")
            check(quiet.locator(".reveal").first.evaluate("e => getComputedStyle(e).opacity") == "1", "Reduced motion leaves content visible")
            quiet.mouse.move(1000, 300)
            check(not quiet.locator("#artwork").get_attribute("style"), "Reduced motion disables pointer tilt")
            reduced.close()

            blocked = browser.new_context()
            blocked.add_init_script("Object.defineProperty(window, 'localStorage', {get(){throw new DOMException('blocked', 'SecurityError')}})")
            blocked_page = blocked.new_page()
            blocked_errors = []
            blocked_page.on("pageerror", lambda error: blocked_errors.append(str(error)))
            blocked_page.goto(base, wait_until="networkidle")
            before = blocked_page.locator("html").get_attribute("data-theme")
            blocked_page.locator(".theme-toggle").click()
            check(before != blocked_page.locator("html").get_attribute("data-theme"), "Theme works when storage is blocked")
            check(not blocked_errors, "No errors with blocked storage")
            blocked.close()

            nojs = browser.new_context(java_script_enabled=False, viewport={"width": 390, "height": 844})
            fallback = nojs.new_page()
            fallback.goto(base, wait_until="networkidle")
            check(fallback.locator("#hero-title").is_visible(), "No-JS homepage content")
            check(fallback.locator(".feature-card").first.is_visible(), "No-JS feature content")
            fallback.goto(base + "download.html", wait_until="networkidle")
            check(fallback.locator('.direct-links a[href$=".apk"]').count() == 2, "No-JS direct APK links")
            check(all(href.startswith(proxy_prefix) and href.removeprefix(proxy_prefix) in readme for href in fallback.locator('a[href$=".apk"]').evaluate_all("els => els.map(e => e.href)")), "No-JS APK downloads use proxy")
            check(fallback.locator(".direct-links").is_visible(), "No-JS downloads visible")
            nojs.close()

            page.goto((SITE / "index.html").as_uri(), wait_until="load")
            check(page.locator("#hero-title").is_visible(), "Direct file preview")
            load_images(page)
            check(page.locator("img").evaluate_all("els => els.every(i => i.complete && i.naturalWidth > 0)"), "Direct file images load")
            check(not errors, f"No browser errors: {errors}")
            check(not failures, f"No local HTTP errors: {failures}")
            context.close()
            browser.close()

        # Separate server document root proves root-directory deployment too.
        root_server = ThreadingHTTPServer(("127.0.0.1", 0), partial(QuietHandler, directory=str(SITE)))
        Thread(target=root_server.serve_forever, daemon=True).start()
        try:
            from urllib.request import urlopen
            for resource in ("/", "/download.html", "/assets/styles.css", "/assets/app.js"):
                with urlopen(f"http://127.0.0.1:{root_server.server_port}{resource}") as response:
                    check(response.status == 200, f"Root deployment: {resource}")
        finally:
            root_server.shutdown()
            root_server.server_close()
        print(f"PASS: {len(checks)} checks (Chrome browser, real local static HTTP, root + subdirectory + file preview)")
    finally:
        server.shutdown()
        server.server_close()


if __name__ == "__main__":
    main()
