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
        this.innerHTML = '<ol class="chapter"><li class="chapter-item expanded affix "><a href="introduction.html">Introduction</a></li><li class="chapter-item expanded affix "><a href="installation.html">Installation</a></li><li class="chapter-item expanded affix "><li class="part-title">Reference Guide</li><li class="chapter-item expanded "><a href="stickfigures.html"><strong aria-hidden="true">1.</strong> Stickfigures</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="stickfigures/anatomy.html"><strong aria-hidden="true">1.1.</strong> Anatomy of a Stickfigure</a></li><li class="chapter-item expanded "><a href="stickfigures/create.html"><strong aria-hidden="true">1.2.</strong> Create</a></li><li class="chapter-item expanded "><a href="stickfigures/modify.html"><strong aria-hidden="true">1.3.</strong> Modify</a></li><li class="chapter-item expanded "><a href="stickfigures/create-from-bytes.html"><strong aria-hidden="true">1.4.</strong> Create from bytes</a></li><li class="chapter-item expanded "><a href="stickfigures/export-from-bytes.html"><strong aria-hidden="true">1.5.</strong> Export to bytes</a></li></ol></li><li class="chapter-item expanded "><a href="nodes.html"><strong aria-hidden="true">2.</strong> Nodes</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="nodes/anatomy.html"><strong aria-hidden="true">2.1.</strong> Anatomy of a Node</a></li><li class="chapter-item expanded "><a href="nodes/get.html"><strong aria-hidden="true">2.2.</strong> Get</a></li><li class="chapter-item expanded "><a href="nodes/create.html"><strong aria-hidden="true">2.3.</strong> Create</a></li><li class="chapter-item expanded "><a href="nodes/modify.html"><strong aria-hidden="true">2.4.</strong> Modify</a></li><li class="chapter-item expanded "><a href="nodes/remove.html"><strong aria-hidden="true">2.5.</strong> Remove</a></li><li class="chapter-item expanded "><a href="nodes/bulk.html"><strong aria-hidden="true">2.6.</strong> Bulk operations</a></li><li class="chapter-item expanded "><a href="nodes/examples.html"><strong aria-hidden="true">2.7.</strong> Examples</a></li></ol></li><li class="chapter-item expanded "><a href="polyfills.html"><strong aria-hidden="true">3.</strong> Polyfills</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="polyfills/anatomy.html"><strong aria-hidden="true">3.1.</strong> Anatomy of a Polyfill</a></li><li class="chapter-item expanded "><a href="polyfills/get.html"><strong aria-hidden="true">3.2.</strong> Get</a></li><li class="chapter-item expanded "><a href="polyfills/create.html"><strong aria-hidden="true">3.3.</strong> Create</a></li><li class="chapter-item expanded "><a href="polyfills/modify.html"><strong aria-hidden="true">3.4.</strong> Modify</a></li><li class="chapter-item expanded "><a href="polyfills/remove.html"><strong aria-hidden="true">3.5.</strong> Remove</a></li><li class="chapter-item expanded "><a href="polyfills/examples.html"><strong aria-hidden="true">3.6.</strong> Examples</a></li></ol></li></ol>';
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
