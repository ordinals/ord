<div dir="rtl">

# دليل Ordinal Inscription


يمكن أن تُرسم العملات الفردية بمحتوى تعسفي، مما يُنشئ قطع فنية رقمية متناسبة مع بيتكوين يمكن الاحتفاظ بها في محفظة بيتكوين ونقلها باستخدام المعاملات المالية في بيتكوين. إن النقوش متينة وثابتة وآمنة ولامركزية تمامًا مثل بيتكوين نفسه.


العمل مع النقوش يتطلب وجود نود بيتكوين كاملة لمنحك نظرة على الحالة الحالية لسلسلة كتل بيتكوين، ومحفظة قادرة على إنشاء النقوش وأداء التحكم في العملات عند بناء المعاملات لإرسال النقوش إلى محفظة أخرى.


توفر Bitcoin Core كل من نود بيتكوين كاملة ومحفظة. ومع ذلك، لا يمكن لمحفظة Bitcoin Core إنشاء النقوش ولا تؤدي إلى التحكم في العملات.


هذا يتطلب [ord](https://github.com/ordinals/ord)، وهو أداة ترتيبية. ord لا تنفذ محفظة خاصة بها، لذا تفاعل أوامر محفظة ord مع محافظ Bitcoin Core.


تشمل هذه الدليل الفقرات التالية:

<div dir="rtl">


1. تثبيت Bitcoin Core
2. مزامنة Bitcoin blockchain
3. إنشاء محفظة Bitcoin Core
4. استخدام `ord wallet receive` لاستقبال العملات
5. إنشاء الinscriptions  باستخدام `ord wallet inscribe`
6. إرسال الinscriptions  باستخدام `ord wallet send`
7. استقبال الinscriptions  باستخدام `ord wallet receive`


<div dir="rtl">


## الحصول على المساعدة


إذا واجهتك مشكلة، جرب أن تطلب المساعدة على خادم <a href="https://discord.com/invite/87cjuz4FYg">Ordinals Discord
Server</a>، أو تفقد موقع [GitHub](https://github.com/ordinals/ord/issues) للقضايا والمناقشات ذات الصلة.


## تهيئة Bitcoin Core


يتوفر Bitcoin Core من موقع [bitcoincore.org](https://bitcoincore.org/) على صفحة [التنزيل](https://bitcoincore.org/en/download/).


إن إجراء Inscribtions يتطلب Bitcoin Core 24 أو الإصدارات الأحدث.


لا يتضمن هذا الدليل تفاصيل تثبيت Bitcoin Core بالتفصيل. بمجرد تثبيت Bitcoin Core، يجب أن تتمكن من تشغيل bitcoind -version بنجاح من سطر الأوامر.


تهيئة Bitcoin Core
تتطلب ord فهرس المعاملات الخاص بـ Bitcoin Core.


لتهيئة نود Bitcoin Core الخاص بك للحفاظ على فهرس المعاملات، أضف ما يلي إلى ملف bitcoin.conf الخاص بك:


<pre><code>bitcoind -txindex</code></pre>


<p>أو ابدأ <code>bitcoind</code> مع <code>-txindex</code>:</p>
<pre><code>bitcoind -txindex
</code></pre>



<h2 id="syncing-the-bitcoin-blockchain"><a class="header" href="#syncing-the-bitcoin-blockchain">مزامنة شبكه البيتكوين </a></h2>
<p> لمزامنه الشبكه استخدم:</p>
<pre><code>bitcoind -txindex
</code></pre>
<p>…واتركها تعمل لحتى <code>getblockcount</code>:</p>
<pre><code>bitcoin-cli getblockcount
</code></pre>

<div dir="rtl">

يتم قبول عدد البلوكات من خلال المتصفح مثل
 <a href="https://mempool.space/">the mempool.space block
explorer</a>.

`ord` يتفاعل مع `bitcoind` لذالك يجب ان تبقي النود فعال بلخلفيه عند استعمالك له



<h2 id="installing-ord"><a class="header" href="#installing-ord">تحميل برنامج <code>ord</code></a></h2>
<p>ادوات <code>ord</code>مكتوبه بلغه الداست ويمكن بنائها من 
<a href="https://github.com/ordinals/ord">المصدر</a>. الثنائيات المبنية مسبقًا متوفرة على
<a href="https://github.com/ordinals/ord/releases">صفحه الاصدار</a>.</p>
<p>يمكنك تحميل الثنائيات المبنية مسبقًا باعطاء الامر التالي:</p>
<pre><code class="language-sh">curl --proto '=https' --tlsv1.2 -fsLS https://ordinals.com/install.sh | bash -s
</code></pre>
<p>بمجرد الانتهاء من تحميل ال <code>ord</code> ، يجب أن تكون قادرًا على تشغيل:</p>
<pre><code>ord --version
</code></pre>
والذي يشير الى الاصدار الحالي.

<h2 id="creating-a-bitcoin-core-wallet"><a class="header" href="#creating-a-bitcoin-core-wallet">إنشاء محفظة بيتكوين كور</a></h2>
<p><code>ord</code> يستخدم بيتكوين كور ليدير المفاتيح الخاصه وتواقيع المعاملات وارسالها الى شبكه البيتكوين 

لإنشاء محفظة Bitcoin Core تحمل اسم <code>ord</code> واستخدامها, اعطي الامر:

<pre><code>ord wallet create
</code></pre>

<h2 id="receiving-sats"><a class="header" href="#receiving-sats">اسقبال السات</a></h2>

ال Inscriptions مؤلفه من وحدات سات فرديه باستعمال عدي لمدواله البيتكوين التي تستعمل السات كرسوم, لذالك ستحتاج محفظتط لبعض السات.

احصل على عنوان محفظه <code> ord </code> الجديد من خلال الأمر التالي:

<pre><code>ord wallet receive
</code></pre>
<p>وأرسل اليها بعض التمويل</p>
<p>يمكنك رؤية المعاملات المعلقة بلامر التالي:</p>
<pre><code>ord wallet transactions
</code></pre>
> بمجرد تأكيد الصفقة، يجب أن تكون قادرًا على رؤية  المعاملات  من خلال <code>ord wallet outputs</code>.


<h2 id="creating-inscription-content"><a class="header" href="#creating-inscription-content">أنشاء محتو ال Inscription </a></h2>

ال Inscriptions مؤلفه من وحدات سات فرديه باستعمال عدي لمدواله البيتكوين التي تستعمل السات كرسوم, لذالك ستحتاج محفظتط لبعض السات.

احصل على عنوان امحفظه  الجديد من خلال الأمر التالي:


ساتوشيات يمكن أن تُنقش (inscribed ) بأي نوع من المحتوى، ولكن محفظة <code>ord</code> تدعم فقط أنواع المحتوى التي يمكن عرضها من قبل متصفح البلوك block explorer <code>ord</code>. بالإضافة إلى ذلك، تُدرج ال Inscriptions في المعاملات، لذا كلما زاد حجم المحتوى، زادت الرسوم التي يجب دفعها لعملية Inscriptions .
يتم تضمين محتوى النقش في شهادات المعاملات، والتي تحصل على الشاهد. لحساب الرسوم التقريبية التي ستدفعها معاملة الInscriptions ، قسم حجم المحتوى على أربعة ثم اضرب الناتج بمعدل الرسوم.
يجب أن تكون المعاملات أقل من 400,000 وحدة وزنية، وإلا فلن يتم نقلها بواسطة بيتكوين كور. بايت واحد من المحتوى يكلف وحدة وزنية واحدة. نظرًا لأن المعاملات لا تشمل فقط المحتوى ، يجب تقييد محتوى Inscriptions  لأقل من 400,000 وحدة وزنية. 390,000 وحدة وزنية يجب أن تكون آمنة وصحيحه.



<h2 id="creating-inscriptions"><a class="header" href="#creating-inscriptions">إنشاء ال Inscriptions</a></h2>
لأنشاء الinscription  مع محتوى ملف , اعطي الأمر:
<pre><code>ord wallet inscribe --fee-rate FEE_RATE FILE
</code></pre>
Ord سيقوم بإخراج رقمين معرّفين للمعاملات، واحد لمعاملة الالتزام، وآخر لمعاملة الكشف، وكذلك مُعرف Inscription . مُعرفات Inscription  على الشكل <code>TXIDiN</code>، حيث <code>TXID</code> هو مُعرّف المعاملة لمعاملة الكشف، و<code>N</code> هو المؤشر Inscription  في معاملة الكشف.

تُؤكد معاملة الالتزام على tapscript يحتوي على محتوى الInscription، وتقوم معاملة الكشف بإنفاق من هذا الـ tapscript، مكشفةً المحتوى على الشبكه وتسجيله على أول ساتوشي في المدخل الذي يحتوي على الـ tapscript المقابل.


انتظر حتى يتم تعدين المعامله. يمكنك التحقق منها من خلال ال <a href="https://mempool.space/">the mempool.space block
explorer</a>

 عند الانتهاء الرقم الخاص ب ال Inscriptions سيكون جاهز  بلأمر التالي:
<pre><code>ord wallet inscriptions
</code></pre>
ويمكنك ايضا الوصول اليه من خلال المتصفح <a href="https://ordinals.com/">the ordinals explorer</a> at
<code>ordinals.com/inscription/INSCRIPTION_ID</code> 



<h2 id="sending-inscriptions"><a class="header" href="#sending-inscriptions">ارسال ال  Inscriptions</a></h2>

اطلب من المستلم إنشاء عنوان جديد عن طريق تشغيل:
<pre><code>ord wallet receive
</code></pre>
ارسل ال inscription من خلال:

<pre><code>ord wallet send --fee-rate &lt;FEE_RATE&gt; &lt;ADDRESS&gt; &lt;INSCRIPTION_ID&gt;
</code></pre>
تفقد حاله المعامله من خلال :

<pre><code>ord wallet transactions
</code></pre>
بمجرد تأكيد المعاملة المرسلة، يمكن للمستلم تأكيد الاستلام عن طريق الامر :

<pre><code>ord wallet inscriptions
</code></pre>


<h2 id="receiving-inscriptions"><a class="header" href="#receiving-inscriptions">استلام ال Inscriptions</a></h2>

قم بإنشاء عنوان استقبال جديد باستخدام:
<pre><code>ord wallet receive
</code></pre>
يمكن للمرسل نقل Inscriptions إلى عنوانك باستخدام:
<pre><code>ord wallet send ADDRESS INSCRIPTION_ID
</code></pre>
تفقد حاله المعامله من خلال :
<pre><code>ord wallet transactions
</code></pre>
بمجرد تأكيد المعاملة المرسلة، يمكن للمستلم تأكيد الاستلام عن طريق الامر :
<pre><code>ord wallet inscriptions
</code></pre>

