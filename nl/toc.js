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
        this.innerHTML = '<ol class="chapter"><li class="chapter-item expanded affix "><a href="introduction.html">Introductie</a></li><li class="chapter-item expanded "><a href="overview.html"><strong aria-hidden="true">1.</strong> Overzicht</a></li><li class="chapter-item expanded "><a href="digital-artifacts.html"><strong aria-hidden="true">2.</strong> Digitale artefacten</a></li><li class="chapter-item expanded "><a href="inscriptions.html"><strong aria-hidden="true">3.</strong> Inscripties</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="inscriptions/delegate.html"><strong aria-hidden="true">3.1.</strong> Gemachtigde</a></li><li class="chapter-item expanded "><a href="inscriptions/metadata.html"><strong aria-hidden="true">3.2.</strong> Metadata</a></li><li class="chapter-item expanded "><a href="inscriptions/pointer.html"><strong aria-hidden="true">3.3.</strong> Pointer</a></li><li class="chapter-item expanded "><a href="inscriptions/provenance.html"><strong aria-hidden="true">3.4.</strong> Provenance</a></li><li class="chapter-item expanded "><a href="inscriptions/recursion.html"><strong aria-hidden="true">3.5.</strong> Recursie</a></li><li class="chapter-item expanded "><a href="inscriptions/rendering.html"><strong aria-hidden="true">3.6.</strong> Rendering</a></li><li class="chapter-item expanded "><a href="inscriptions/uris.html"><strong aria-hidden="true">3.7.</strong> URIs</a></li><li class="chapter-item expanded "><a href="inscriptions/examples.html"><strong aria-hidden="true">3.8.</strong> Voorbeelden</a></li></ol></li><li class="chapter-item expanded "><a href="runes.html"><strong aria-hidden="true">4.</strong> Runes</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="runes/specification.html"><strong aria-hidden="true">4.1.</strong> Specificatie</a></li></ol></li><li class="chapter-item expanded "><a href="faq.html"><strong aria-hidden="true">5.</strong> Veelgestelde Vragen</a></li><li class="chapter-item expanded "><a href="contributing.html"><strong aria-hidden="true">6.</strong> Bijdragen</a></li><li class="chapter-item expanded "><a href="donate.html"><strong aria-hidden="true">7.</strong> Donaties</a></li><li class="chapter-item expanded "><a href="guides.html"><strong aria-hidden="true">8.</strong> Handleidingen</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="guides/api.html"><strong aria-hidden="true">8.1.</strong> API</a></li><li class="chapter-item expanded "><a href="guides/explorer.html"><strong aria-hidden="true">8.2.</strong> Verkenner</a></li><li class="chapter-item expanded "><a href="guides/wallet.html"><strong aria-hidden="true">8.3.</strong> Wallet</a></li><li class="chapter-item expanded "><a href="guides/batch-inscribing.html"><strong aria-hidden="true">8.4.</strong> Batch Inscriptie</a></li><li class="chapter-item expanded "><a href="guides/splitting.html"><strong aria-hidden="true">8.5.</strong> Splitting</a></li><li class="chapter-item expanded "><a href="guides/collecting.html"><strong aria-hidden="true">8.6.</strong> Verzamelen</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="guides/collecting/sparrow-wallet.html"><strong aria-hidden="true">8.6.1.</strong> Sparrow Wallet</a></li></ol></li><li class="chapter-item expanded "><a href="guides/moderation.html"><strong aria-hidden="true">8.7.</strong> Beheer</a></li><li class="chapter-item expanded "><a href="guides/reindexing.html"><strong aria-hidden="true">8.8.</strong> Herindexeren</a></li><li class="chapter-item expanded "><a href="guides/sat-hunting.html"><strong aria-hidden="true">8.9.</strong> Sats Jagen</a></li><li class="chapter-item expanded "><a href="guides/settings.html"><strong aria-hidden="true">8.10.</strong> Instellingen</a></li><li class="chapter-item expanded "><a href="guides/teleburning.html"><strong aria-hidden="true">8.11.</strong> Teleburnen</a></li><li class="chapter-item expanded "><a href="guides/testing.html"><strong aria-hidden="true">8.12.</strong> Testen</a></li></ol></li><li class="chapter-item expanded "><a href="bounties.html"><strong aria-hidden="true">9.</strong> Beloningen</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="bounty/0.html"><strong aria-hidden="true">9.1.</strong> Beloning 0: 100.000 sats Geclaimd!</a></li><li class="chapter-item expanded "><a href="bounty/1.html"><strong aria-hidden="true">9.2.</strong> Beloning 1: 200.000 sats Geclaimd!</a></li><li class="chapter-item expanded "><a href="bounty/2.html"><strong aria-hidden="true">9.3.</strong> Beloning 2: 300.000 sats Geclaimd!</a></li><li class="chapter-item expanded "><a href="bounty/3.html"><strong aria-hidden="true">9.4.</strong> Beloning 3: 400.000 sats</a></li></ol></li></ol>';
        // Set the current, active page, and reveal it if it's hidden
        let current_page = document.location.href.toString();
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
