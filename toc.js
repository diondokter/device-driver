// Populate the sidebar
//
// This is a script, and not included directly in the page, to control the total size of the book.
// The TOC contains an entry for each page, so if each page includes a copy of the TOC,
// the total size of the page becomes O(n**2).
class MDBookSidebarScrollbox extends HTMLElement {
    constructor() {
        super();
    }
    connectedCallback() {
        this.innerHTML = '<ol class="chapter"><li class="chapter-item expanded affix "><a href="preface.html">Preface</a></li><li class="chapter-item expanded affix "><li class="spacer"></li><li class="chapter-item expanded "><div><strong aria-hidden="true">1.</strong> v2</div></li><li><ol class="section"><li class="chapter-item expanded "><div><strong aria-hidden="true">1.1.</strong> TODO</div></li></ol></li><li class="chapter-item expanded "><div><strong aria-hidden="true">2.</strong> v1</div></li><li><ol class="section"><li class="chapter-item expanded "><a href="v1/intro.html"><strong aria-hidden="true">2.1.</strong> Intro</a></li><li class="chapter-item expanded "><a href="v1/using-the-macro.html"><strong aria-hidden="true">2.2.</strong> Using the macro</a></li><li class="chapter-item expanded "><a href="v1/using-the-cli.html"><strong aria-hidden="true">2.3.</strong> Using the cli</a></li><li class="chapter-item expanded "><a href="v1/writing-an-interface.html"><strong aria-hidden="true">2.4.</strong> Writing an interface</a></li><li class="chapter-item expanded "><a href="v1/defining-the-device.html"><strong aria-hidden="true">2.5.</strong> Defining the device</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="v1/global-config.html"><strong aria-hidden="true">2.5.1.</strong> Global config</a></li><li class="chapter-item expanded "><a href="v1/registers.html"><strong aria-hidden="true">2.5.2.</strong> Registers</a></li><li class="chapter-item expanded "><a href="v1/commands.html"><strong aria-hidden="true">2.5.3.</strong> Commands</a></li><li class="chapter-item expanded "><a href="v1/field-sets.html"><strong aria-hidden="true">2.5.4.</strong> Field sets</a></li><li class="chapter-item expanded "><a href="v1/buffers.html"><strong aria-hidden="true">2.5.5.</strong> Buffers</a></li><li class="chapter-item expanded "><a href="v1/blocks.html"><strong aria-hidden="true">2.5.6.</strong> Blocks</a></li><li class="chapter-item expanded "><a href="v1/refs.html"><strong aria-hidden="true">2.5.7.</strong> Refs</a></li><li class="chapter-item expanded "><a href="v1/dsl-syntax.html"><strong aria-hidden="true">2.5.8.</strong> DSL syntax</a></li><li class="chapter-item expanded "><a href="v1/manifest-syntax.html"><strong aria-hidden="true">2.5.9.</strong> Manifest syntax</a></li></ol></li><li class="chapter-item expanded "><div><strong aria-hidden="true">2.6.</strong> Addendum</div></li><li><ol class="section"><li class="chapter-item expanded "><a href="v1/cfg.html"><strong aria-hidden="true">2.6.1.</strong> Cfg</a></li></ol></li></ol></li><li class="chapter-item expanded "><li class="part-title">Addendum</li><li class="chapter-item expanded "><a href="memory.html"><strong aria-hidden="true">3.</strong> Memory</a></li></ol>';
        // Set the current, active page, and reveal it if it's hidden
        let current_page = document.location.href.toString().split("#")[0].split("?")[0];
        if (current_page.endsWith("/")) {
            current_page += "index.html";
        }
        var links = Array.prototype.slice.call(this.querySelectorAll("a"));
        var l = links.length;
        for (var i = 0; i < l; ++i) {
            var link = links[i];
            var href = link.getAttribute("href");
            if (href && !href.startsWith("#") && !/^(?:[a-z+]+:)?\/\//.test(href)) {
                link.href = path_to_root + href;
            }
            // The "index" page is supposed to alias the first chapter in the book.
            if (link.href === current_page || (i === 0 && path_to_root === "" && current_page.endsWith("/index.html"))) {
                link.classList.add("active");
                var parent = link.parentElement;
                if (parent && parent.classList.contains("chapter-item")) {
                    parent.classList.add("expanded");
                }
                while (parent) {
                    if (parent.tagName === "LI" && parent.previousElementSibling) {
                        if (parent.previousElementSibling.classList.contains("chapter-item")) {
                            parent.previousElementSibling.classList.add("expanded");
                        }
                    }
                    parent = parent.parentElement;
                }
            }
        }
        // Track and set sidebar scroll position
        this.addEventListener('click', function(e) {
            if (e.target.tagName === 'A') {
                sessionStorage.setItem('sidebar-scroll', this.scrollTop);
            }
        }, { passive: true });
        var sidebarScrollTop = sessionStorage.getItem('sidebar-scroll');
        sessionStorage.removeItem('sidebar-scroll');
        if (sidebarScrollTop) {
            // preserve sidebar scroll position when navigating via links within sidebar
            this.scrollTop = sidebarScrollTop;
        } else {
            // scroll sidebar to current active section when navigating via "next/previous chapter" buttons
            var activeSection = document.querySelector('#sidebar .active');
            if (activeSection) {
                activeSection.scrollIntoView({ block: 'center' });
            }
        }
        // Toggle buttons
        var sidebarAnchorToggles = document.querySelectorAll('#sidebar a.toggle');
        function toggleSection(ev) {
            ev.currentTarget.parentElement.classList.toggle('expanded');
        }
        Array.from(sidebarAnchorToggles).forEach(function (el) {
            el.addEventListener('click', toggleSection);
        });
    }
}
window.customElements.define("mdbook-sidebar-scrollbox", MDBookSidebarScrollbox);
