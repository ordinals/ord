<div dir="rtl">

<h2 id="recursion"><a class="header" href="#recursion">التكرار Recursion</a></h2>

<p>استثناء مهم من تطبيق البيئة العازلة sandboxing هو التكرار: يُسمح بالوصول إلى نقطة النهاية /content في "ord"، مما يتيح inscription الوصول إلى محتوى inscription اخر من خلال طلب
<code>/content/&lt;INSCRIPTION_ID&gt;</code>.</p>


<p>العديد من الاستخدامات المثيرة للاهتمام:


</p>
<ul>
<li>
<p>إعادة مزج محتوى inscription الحالية</p>
</li>
<li>
<p>نشر مقتطفات من الشفرة، والصور، والصوت، أو ورقات الأنماط كموارد عامة مشتركة</p>
</li>
<li>
<p>مجموعات الفن التوليدية حيث يتم توثيق خوارزمية باستخدام JavaScript، ويتم تفعيلها من خلال inscription متعددة ببذور فريدة.</p>
</li>
<li>
<p>مجموعات صور الملف الشخصي التوليدية حيث يتم تدوين الاكسسوارات والسمات كصور فردية، أو ضمن أطلس للنسيج المشترك، ثم يتم دمجها، على طريقة التراص، في مزيجات فريدة في inscriptions متعددة.</p>
</li>
</ul>
<p>نقاط  أخرى التي يمكن inscriptions الوصول إليها هي كما يلي:
</p>
<ul>
<div dir="ltr">
<li><code>/blockhash</code>: latest block hash.</li>
<li><code>/blockhash/&lt;HEIGHT&gt;</code>: block hash at given block height.</li>
<li><code>/blocktime</code>: UNIX time stamp of latest block.</li>