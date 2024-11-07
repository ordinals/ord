// Populate the sidebar
//
// This is a script, and not included directly in the page, to control the total size of the book.
// The TOC contains an entry for each page, so if each page includes a copy of the TOC,
// the total size of the page becomes O(n**2).
var sidebarScrollbox = document.querySelector("#sidebar .sidebar-scrollbox");
sidebarScrollbox.innerHTML = '<ol class="chapter"><li class="chapter-item expanded affix "><a href="introduction.html">Panimula</a></li><li class="chapter-item expanded "><a href="overview.html"><strong aria-hidden="true">1.</strong> Pangkalahatang-ideya</a></li><li class="chapter-item expanded "><a href="digital-artifacts.html"><strong aria-hidden="true">2.</strong> Digital Artifacts</a></li><li class="chapter-item expanded "><a href="inscriptions.html"><strong aria-hidden="true">3.</strong> Inscriptions</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="inscriptions/delegate.html"><strong aria-hidden="true">3.1.</strong> Delegate</a></li><li class="chapter-item expanded "><a href="inscriptions/metadata.html"><strong aria-hidden="true">3.2.</strong> Metadata</a></li><li class="chapter-item expanded "><a href="inscriptions/pointer.html"><strong aria-hidden="true">3.3.</strong> Pointer</a></li><li class="chapter-item expanded "><a href="inscriptions/provenance.html"><strong aria-hidden="true">3.4.</strong> Provenance</a></li><li class="chapter-item expanded "><a href="inscriptions/recursion.html"><strong aria-hidden="true">3.5.</strong> Recursion</a></li><li class="chapter-item expanded "><a href="inscriptions/rendering.html"><strong aria-hidden="true">3.6.</strong> Rendering</a></li><li class="chapter-item expanded "><a href="inscriptions/examples.html"><strong aria-hidden="true">3.7.</strong> Examples</a></li></ol></li><li class="chapter-item expanded "><a href="runes.html"><strong aria-hidden="true">4.</strong> Runes</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="runes/specification.html"><strong aria-hidden="true">4.1.</strong> Pagtutukoy</a></li></ol></li><li class="chapter-item expanded "><a href="faq.html"><strong aria-hidden="true">5.</strong> Mga Madalas Itanong</a></li><li class="chapter-item expanded "><a href="contributing.html"><strong aria-hidden="true">6.</strong> Mag-ambag</a></li><li class="chapter-item expanded "><a href="donate.html"><strong aria-hidden="true">7.</strong> Donate</a></li><li class="chapter-item expanded "><a href="guides.html"><strong aria-hidden="true">8.</strong> Mga gabay</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="guides/api.html"><strong aria-hidden="true">8.1.</strong> API</a></li><li class="chapter-item expanded "><a href="guides/explorer.html"><strong aria-hidden="true">8.2.</strong> Explorer</a></li><li class="chapter-item expanded "><a href="guides/wallet.html"><strong aria-hidden="true">8.3.</strong> Wallet</a></li><li class="chapter-item expanded "><a href="guides/batch-inscribing.html"><strong aria-hidden="true">8.4.</strong> Batch Inscribing</a></li><li class="chapter-item expanded "><a href="guides/collecting.html"><strong aria-hidden="true">8.5.</strong> Pagkolekta</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="guides/collecting/sparrow-wallet.html"><strong aria-hidden="true">8.5.1.</strong> Sparrow Wallet</a></li></ol></li><li class="chapter-item expanded "><a href="guides/moderation.html"><strong aria-hidden="true">8.6.</strong> Moderation</a></li><li class="chapter-item expanded "><a href="guides/reindexing.html"><strong aria-hidden="true">8.7.</strong> Reindexing</a></li><li class="chapter-item expanded "><a href="guides/sat-hunting.html"><strong aria-hidden="true">8.8.</strong> Sat Hunting</a></li><li class="chapter-item expanded "><a href="guides/settings.html"><strong aria-hidden="true">8.9.</strong> Settings</a></li><li class="chapter-item expanded "><a href="guides/teleburning.html"><strong aria-hidden="true">8.10.</strong> Teleburning</a></li><li class="chapter-item expanded "><a href="guides/testing.html"><strong aria-hidden="true">8.11.</strong> Pagsuri</a></li></ol></li><li class="chapter-item expanded "><a href="bounties.html"><strong aria-hidden="true">9.</strong> Bounties</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="bounty/0.html"><strong aria-hidden="true">9.1.</strong> Bounty 0: 100,000 sats Claimed!</a></li><li class="chapter-item expanded "><a href="bounty/1.html"><strong aria-hidden="true">9.2.</strong> Bounty 1: 200,000 sats Claimed!</a></li><li class="chapter-item expanded "><a href="bounty/2.html"><strong aria-hidden="true">9.3.</strong> Bounty 2: 300,000 sats Claimed!</a></li><li class="chapter-item expanded "><a href="bounty/3.html"><strong aria-hidden="true">9.4.</strong> Bounty 3: 400,000 sats</a></li></ol></li></ol>';
(function() {
    let current_page = document.location.href.toString();
    if (current_page.endsWith("/")) {
        current_page += "index.html";
    }
    var links = sidebarScrollbox.querySelectorAll("a");
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
})();

// Track and set sidebar scroll position
sidebarScrollbox.addEventListener('click', function(e) {
    if (e.target.tagName === 'A') {
        sessionStorage.setItem('sidebar-scroll', sidebarScrollbox.scrollTop);
    }
}, { passive: true });
var sidebarScrollTop = sessionStorage.getItem('sidebar-scroll');
sessionStorage.removeItem('sidebar-scroll');
if (sidebarScrollTop) {
    // preserve sidebar scroll position when navigating via links within sidebar
    sidebarScrollbox.scrollTop = sidebarScrollTop;
} else {
    // scroll sidebar to current active section when navigating via "next/previous chapter" buttons
    var activeSection = document.querySelector('#sidebar .active');
    if (activeSection) {
        activeSection.scrollIntoView({ block: 'center' });
    }
}
