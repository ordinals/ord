
<div dir="rtl">

# Sat Hunting صيد الساتوشي

<p dir="rtl"><em>هذا الدليل غير محدّث. منذ كتابته، تم تغيير البرنامج <code>ord</code> الثنائي لبناء فهرس الساتوشي الكامل فقط عندما يتم توفير <code>--index-sats</code>. بالإضافة إلى ذلك، <code>ord</code> الآن يحتوي على محفظة مدمجة تلتف حول محفظة بيتكوين الأساسية. راجع <code>ord wallet --help</code>.</em></p>
<p dir="rtl">صيد الاوردينال أمرٌ صعب ولكنه مكافئ. الشعور بامتلاك محفظة مليئة بـ UTXOs، محملة بالساتوشي النادرة والاستثنائية، لا يمكن وصفه.</p>

<p dir="rtl" 

Ordinals هي أعداد تُستخدم للساتوشي. كل ساتوشي لديه رقم ترتيبي وكل رقم ترتيبي لديه ساتوشي.


<div dir="ltr">

<h2 id="preparation"><a class="header" href="#preparation">Preparation</a></h2>
<p>There are a few things you'll need before you start.</p>
<ol>
<li>
<p>First, you'll need a synced Bitcoin Core node with a transaction index. To
turn on transaction indexing, pass <code>-txindex</code> on the command-line:</p>
<pre><code class="language-sh">bitcoind -txindex
</code></pre>
<p>Or put the following in your <a href="https://github.com/bitcoin/bitcoin/blob/master/doc/bitcoin-conf.md#configuration-file-path">Bitcoin configuration
file</a>:</p>
<pre><code>txindex=1
</code></pre>
<p>Launch it and wait for it to catch up to the chain tip, at which point the
following command should print out the current block height:</p>
<pre><code class="language-sh">bitcoin-cli getblockcount
</code></pre>
</li>
<li>
<p>Second, you'll need a synced <code>ord</code> index.</p>
<ul>
<li>
<p>Get a copy of <code>ord</code> from <a href="https://github.com/ordinals/ord/">the repo</a>.</p>
</li>
<li>
<p>Run <code>RUST_LOG=info ord index</code>. It should connect to your bitcoin core
node and start indexing.</p>
</li>
<li>
<p>Wait for it to finish indexing.</p>
</li>
</ul>
</li>
<li>
<p>Third, you'll need a wallet with UTXOs that you want to search.</p>
</li>
</ol>
<h2 id="searching-for-rare-ordinals"><a class="header" href="#searching-for-rare-ordinals">Searching for Rare Ordinals</a></h2>
<h3 id="searching-for-rare-ordinals-in-a-bitcoin-core-wallet"><a class="header" href="#searching-for-rare-ordinals-in-a-bitcoin-core-wallet">Searching for Rare Ordinals in a Bitcoin Core Wallet</a></h3>
<p>The <code>ord wallet</code> command is just a wrapper around Bitcoin Core's RPC API, so
searching for rare ordinals in a Bitcoin Core wallet is Easy. Assuming your
wallet is named <code>foo</code>:</p>
<ol>
<li>
<p>Load your wallet:</p>
<pre><code class="language-sh">bitcoin-cli loadwallet foo
</code></pre>
</li>
<li>
<p>Display any rare ordinals wallet <code>foo</code>'s UTXOs:</p>
<pre><code class="language-sh">ord wallet sats
</code></pre>
</li>
</ol>
<h3 id="searching-for-rare-ordinals-in-a-non-bitcoin-core-wallet"><a class="header" href="#searching-for-rare-ordinals-in-a-non-bitcoin-core-wallet">Searching for Rare Ordinals in a Non-Bitcoin Core Wallet</a></h3>
<p>The <code>ord wallet</code> command is just a wrapper around Bitcoin Core's RPC API, so to
search for rare ordinals in a non-Bitcoin Core wallet, you'll need to import
your wallet's descriptors into Bitcoin Core.</p>
<p><a href="https://github.com/bitcoin/bitcoin/blob/master/doc/descriptors.md">Descriptors</a>
describe the ways that wallets generate private keys and public keys.</p>
<p>You should only import descriptors into Bitcoin Core for your wallet's public
keys, not its private keys.</p>
<p>If your wallet's public key descriptor is compromised, an attacker will be able
to see your wallet's addresses, but your funds will be safe.</p>
<p>If your wallet's private key descriptor is compromised, an attacker can drain
your wallet of funds.</p>
<ol>
<li>
<p>Get the wallet descriptor from the wallet whose UTXOs you want to search for
rare ordinals. It will look something like this:</p>
<pre><code>wpkh([bf1dd55e/84'/0'/0']xpub6CcJtWcvFQaMo39ANFi1MyXkEXM8T8ZhnxMtSjQAdPmVSTHYnc8Hwoc11VpuP8cb8JUTboZB5A7YYGDonYySij4XTawL6iNZvmZwdnSEEep/0/*)#csvefu29
</code></pre>
</li>
<li>
<p>Create a watch-only wallet named <code>foo-watch-only</code>:</p>
<pre><code class="language-sh">bitcoin-cli createwallet foo-watch-only true true
</code></pre>
<p>Feel free to give it a better name than <code>foo-watch-only</code>!</p>
</li>
<li>
<p>Load the <code>foo-watch-only</code> wallet:</p>
<pre><code class="language-sh">bitcoin-cli loadwallet foo-watch-only
</code></pre>
</li>
<li>
<p>Import your wallet descriptors into <code>foo-watch-only</code>:</p>
<pre><code class="language-sh">bitcoin-cli importdescriptors \
  '[{ &quot;desc&quot;: &quot;wpkh([bf1dd55e/84h/0h/0h]xpub6CcJtWcvFQaMo39ANFi1MyXkEXM8T8ZhnxMtSjQAdPmVSTHYnc8Hwoc11VpuP8cb8JUTboZB5A7YYGDonYySij4XTawL6iNZvmZwdnSEEep/0/*)#tpnxnxax&quot;, &quot;timestamp&quot;:0 }]'
</code></pre>
<p>If you know the Unix timestamp when your wallet first started receive
transactions, you may use it for the value of <code>&quot;timestamp&quot;</code> instead of <code>0</code>.
This will reduce the time it takes for Bitcoin Core to search for your
wallet's UTXOs.</p>
</li>
<li>
<p>Check that everything worked:</p>
<pre><code class="language-sh">bitcoin-cli getwalletinfo
</code></pre>
</li>
<li>
<p>Display your wallet's rare ordinals:</p>
<pre><code class="language-sh">ord wallet sats
</code></pre>
</li>
</ol>
<h3 id="searching-for-rare-ordinals-in-a-wallet-that-exports-multi-path-descriptors"><a class="header" href="#searching-for-rare-ordinals-in-a-wallet-that-exports-multi-path-descriptors">Searching for Rare Ordinals in a Wallet that Exports Multi-path Descriptors</a></h3>
<p>Some descriptors describe multiple paths in one descriptor using angle brackets,
e.g., <code>&lt;0;1&gt;</code>. Multi-path descriptors are not yet supported by Bitcoin Core, so
you'll first need to convert them into multiple descriptors, and then import
those multiple descriptors into Bitcoin Core.</p>
<ol>
<li>
<p>First get the multi-path descriptor from your wallet. It will look something
like this:</p>
<pre><code>wpkh([bf1dd55e/84h/0h/0h]xpub6CcJtWcvFQaMo39ANFi1MyXkEXM8T8ZhnxMtSjQAdPmVSTHYnc8Hwoc11VpuP8cb8JUTboZB5A7YYGDonYySij4XTawL6iNZvmZwdnSEEep/&lt;0;1&gt;/*)#fw76ulgt
</code></pre>
</li>
<li>
<p>Create a descriptor for the receive address path:</p>
<pre><code>wpkh([bf1dd55e/84'/0'/0']xpub6CcJtWcvFQaMo39ANFi1MyXkEXM8T8ZhnxMtSjQAdPmVSTHYnc8Hwoc11VpuP8cb8JUTboZB5A7YYGDonYySij4XTawL6iNZvmZwdnSEEep/0/*)
</code></pre>
<p>And the change address path:</p>
<pre><code>wpkh([bf1dd55e/84'/0'/0']xpub6CcJtWcvFQaMo39ANFi1MyXkEXM8T8ZhnxMtSjQAdPmVSTHYnc8Hwoc11VpuP8cb8JUTboZB5A7YYGDonYySij4XTawL6iNZvmZwdnSEEep/1/*)
</code></pre>
</li>
<li>
<p>Get and note the checksum for the receive address descriptor, in this case
<code>tpnxnxax</code>:</p>
<pre><code class="language-sh">bitcoin-cli getdescriptorinfo \
  'wpkh([bf1dd55e/84h/0h/0h]xpub6CcJtWcvFQaMo39ANFi1MyXkEXM8T8ZhnxMtSjQAdPmVSTHYnc8Hwoc11VpuP8cb8JUTboZB5A7YYGDonYySij4XTawL6iNZvmZwdnSEEep/0/*)'
</code></pre>
<pre><code class="language-json">{  &quot;descriptor&quot;: &quot;wpkh([bf1dd55e/84'/0'/0']xpub6CcJtWcvFQaMo39ANFi1MyXkEXM8T8ZhnxMtSjQAdPmVSTHYnc8Hwoc11VpuP8cb8JUTboZB5A7YYGDonYySij4XTawL6iNZvmZwdnSEEep/0/*)#csvefu29&quot;,
  &quot;checksum&quot;: &quot;tpnxnxax&quot;,
  &quot;isrange&quot;: true,
  &quot;issolvable&quot;: true,
  &quot;hasprivatekeys&quot;: false
}
</code></pre>
<p>And for the change address descriptor, in this case <code>64k8wnd7</code>:</p>
<pre><code class="language-sh">bitcoin-cli getdescriptorinfo \
  'wpkh([bf1dd55e/84h/0h/0h]xpub6CcJtWcvFQaMo39ANFi1MyXkEXM8T8ZhnxMtSjQAdPmVSTHYnc8Hwoc11VpuP8cb8JUTboZB5A7YYGDonYySij4XTawL6iNZvmZwdnSEEep/1/*)'
</code></pre>
<pre><code class="language-json">{
  &quot;descriptor&quot;: &quot;wpkh([bf1dd55e/84'/0'/0']xpub6CcJtWcvFQaMo39ANFi1MyXkEXM8T8ZhnxMtSjQAdPmVSTHYnc8Hwoc11VpuP8cb8JUTboZB5A7YYGDonYySij4XTawL6iNZvmZwdnSEEep/1/*)#fyfc5f6a&quot;,
  &quot;checksum&quot;: &quot;64k8wnd7&quot;,
  &quot;isrange&quot;: true,
  &quot;issolvable&quot;: true,
  &quot;hasprivatekeys&quot;: false
}
</code></pre>
</li>
<li>
<p>Load the wallet you want to import the descriptors into:</p>
<pre><code class="language-sh">bitcoin-cli loadwallet foo-watch-only
</code></pre>
</li>
<li>
<p>Now import the descriptors, with the correct checksums, into Bitcoin Core.</p>
<pre><code class="language-sh">bitcoin-cli \
 importdescriptors \
 '[
   {
     &quot;desc&quot;: &quot;wpkh([bf1dd55e/84h/0h/0h]xpub6CcJtWcvFQaMo39ANFi1MyXkEXM8T8ZhnxMtSjQAdPmVSTHYnc8Hwoc11VpuP8cb8JUTboZB5A7YYGDonYySij4XTawL6iNZvmZwdnSEEep/0/*)#tpnxnxax&quot;
     &quot;timestamp&quot;:0
   },
   {
     &quot;desc&quot;: &quot;wpkh([bf1dd55e/84h/0h/0h]xpub6CcJtWcvFQaMo39ANFi1MyXkEXM8T8ZhnxMtSjQAdPmVSTHYnc8Hwoc11VpuP8cb8JUTboZB5A7YYGDonYySij4XTawL6iNZvmZwdnSEEep/1/*)#64k8wnd7&quot;,
     &quot;timestamp&quot;:0
   }
 ]'
</code></pre>
<p>If you know the Unix timestamp when your wallet first started receive
transactions, you may use it for the value of the <code>&quot;timestamp&quot;</code> fields
instead of <code>0</code>. This will reduce the time it takes for Bitcoin Core to
search for your wallet's UTXOs.</p>
</li>
<li>
<p>Check that everything worked:</p>
<pre><code class="language-sh">bitcoin-cli getwalletinfo
</code></pre>
</li>
<li>
<p>Display your wallet's rare ordinals:</p>
<pre><code class="language-sh">ord wallet sats
</code></pre>
</li>
</ol>
<h3 id="exporting-descriptors"><a class="header" href="#exporting-descriptors">Exporting Descriptors</a></h3>
<h4 id="sparrow-wallet"><a class="header" href="#sparrow-wallet">Sparrow Wallet</a></h4>
<p>Navigate to the <code>Settings</code> tab, then to <code>Script Policy</code>, and press the edit
button to display the descriptor.</p>
<h3 id="transferring-ordinals"><a class="header" href="#transferring-ordinals">Transferring Ordinals</a></h3>
<p>The <code>ord</code> wallet supports transferring specific satoshis. You can also use
<code>bitcoin-cli</code> commands <code>createrawtransaction</code>, <code>signrawtransactionwithwallet</code>,
and <code>sendrawtransaction</code>, how to do so is complex and outside the scope of this
guide.

