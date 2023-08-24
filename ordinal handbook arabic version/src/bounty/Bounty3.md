

<div dir="rtl">

# مكافأة 3: 400,000 ساتوشي

<h2 id="criteria"><a class="header" href="#criteria">الشروط</a></h2>
<p>لديها جزأين، يعتمد كل منهما على "أسماء الترتيب" (<em>ordinal names</em>).
أسماء الترتيب هي تشفير معدل من 26 على أساس الترتيب المعدل للأعداد الترتيبية. لتجنب
تأمين الأسماء القصيرة داخل مكافأة coinbase reward الغير صالحة للاستخدام في كتلة النشوء (genesis block coinbase reward)
، تصبح أسماء الترتيب <em>أقصر</em> كلما زادت الأعداد الترتيبية. اسم الساتوشي 0، أول ساتوشي يتم تعدينه هو <code>nvtdijuwxlp</code>
واسم الساتوشي 2,099,999,997,689,999، أخر ساتوشي يتم تعدينه هو <code>a</code>.</p>
<p>المكافأة متاحة للتقديم حتى الكتلة 840000 - أول كتلة بعد الانقسام الرابع. التقديمات المتضمنة في الكتلة 840000 أو بعد ذلك لن يتم النظر فيها.</p>
<p>كلا الجزأين يستخدمان <a href="frequency.tsv">frequency.tsv</a>، وهي قائمة بالكلمات وعدد مرات ظهورها في مجموعة بيانات <a href="http://storage.googleapis.com/books/ngrams/books/datasetsv2.html">جوجل بوكس نغرام</a>
تم تصفية هذه القائمة لتتضمن فقط أسماء الساتوشي التي سيتم تعدينها بحلول نهاية فترة التقديم، والتي تظهر ما لا يقل عن 5000 مرة في النص.</p>
<p>ملف <code>frequency.tsv</code> هو ملف يحتوي على قيم مفصولة بعلامات التبويب. العمود الأول هو الكلمة، والعمود الثاني هو عدد مرات ظهورها في النص. القيم مرتبة من أقل تكرار إلى أعلى تكرار.</p>
<p>تم تجميع <code>frequency.tsv</code> باستخدام <a href="https://github.com/casey/onegrams">هذا البرنامج</a>.</p>
<p>للبحث في محفظة <code>ord</code> عن ساتوشي بأسماء موجودة في <code>frequency.tsv</code>، استخدم الأمر التالي من <a href="https://github.com/ordinals/ord"><code>ord</code></a>:</p>
<pre><code>ord wallet sats --tsv frequency.tsv
</code></pre>
<p>هذا الأمر يتطلب فهرسة الساتوشي، لذا يجب تمرير <code>--index-sats</code> إلى <code>ord</code> عند إنشاء الفهرس للمرة الأولى.</p>
<h3 id="part-0"><a class="header" href="#part-0">الجزء 0</a></h3>
<p><em>الساتوشي نادر يتناسب مع الكلمات النادرة.</em></p>
<p>سيكون الفائز في الجزء 0 هو التقديم الذي يحتوي على ساتوشي مع اسم يظهر بأدنى عدد من المرات في <code>frequency.tsv</code>.</p>
<h3 id="part-1"><a class="header" href="#part-1">الجزء 1</a></h3>
<p><em>الشهرة هي مصدر القيمة.</em></p>
<p>سيكون الفائز في الجزء 1 هو التقديم الذي يحتوي على ساتوشي مع اسم يظهر بأعلى عدد من المرات في <code>frequency.tsv</code>.</p>
<h3 id="tie-breaking"><a class="header" href="#tie-breaking">كسر التعادل</a></h3>
<p>في حالة التعادل، حيث يتم التقديم بنفس التكرار، سيتم اعتبار التقديم الأقدم كفائز.</p>
<h2 id="reward"><a class="header" href="#reward">المكافأة</a></h2>
<ul>
<li>الجزء 0: 200,000 ساتوش

ي</li>
<li>الجزء 1: 200,000 ساتوشي</li>
<li>الإجمالي: 400,000 ساتوشي</li>
</ul>
<h2 id="submission-address"><a class="header" href="#submission-address">عنوان التقديم</a></h2>
<p><a href="https://mempool.space/address/17m5rvMpi78zG8RUpCRd6NWWMJtWmu65kg"><code>17m5rvMpi78zG8RUpCRd6NWWMJtWmu65kg</code></a></p>
<h2 id="status"><a class="header" href="#status">الحالة</a></h2>
<p>لم يتم الاستحواذ عليها بعد!</p>